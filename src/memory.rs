use crate::{MemoryAddresses, MemoryValues};
use asr::{itoa, ryu, signature::Signature, string::ArrayCString, watcher::Pair, Address, Process};
use itoa::Integer;

// the array with all the room names
const ROOM_ID_ARRAY_SIG: Signature<13> = Signature::new("74 0C 48 8B 05 ?? ?? ?? ?? 48 8B 04 D0");
// the id of the current room the player is on (i32)
const ROOM_ID_SIG: Signature<9> = Signature::new("89 3D ?? ?? ?? ?? 48 3B 1D");

// the magic numbers to find for the buffer, using 16 of the 32, good enough
const BUFFER_MAGIC_NUMBER: Signature<16> =
    Signature::new("C2 5A 17 65 BE 4D DF D6 F2 1C D1 3B A7 A6 1F C3");

pub const ROOM_NAME_SIZE_CAP: usize = 0x40;

/**
 * update an f64 pair and display it in the variable view of livesplit
 */
fn update_pair_f64(variable_name: &str, new_value: f64, pair: &mut Pair<f64>) {
    let mut buffer = ryu::Buffer::new();
    asr::timer::set_variable(variable_name, buffer.format(new_value));
    pair.old = pair.current;
    pair.current = new_value;
}

/**
 * update an integer pair and display it in the variable view of livesplit
 */
fn update_pair_int<T: Integer>(variable_name: &str, new_value: T, pair: &mut Pair<T>) {
    let mut buffer = itoa::Buffer::new();
    asr::timer::set_variable(variable_name, buffer.format(new_value));
    pair.old = pair.current;
    pair.current = new_value;
}

/**
 * update a cstring pair and display it in the variable view of livesplit
 */
fn update_pair_cstring(
    variable_name: &str,
    new_value: ArrayCString<ROOM_NAME_SIZE_CAP>,
    pair: &mut Pair<ArrayCString<ROOM_NAME_SIZE_CAP>>,
) {
    asr::timer::set_variable(
        variable_name,
        new_value
            .validate_utf8()
            .unwrap_or("(invalid utf-8 string)"),
    );
    pair.old = pair.current;
    pair.current = new_value;
}

pub fn room_id_sigscan_start(
    process: &asr::Process,
    addresses: &MemoryAddresses,
) -> Result<asr::Address, ()> {
    let main_address = addresses.main_address.unwrap_or(Address::new(0));

    // room id sigscan
    asr::print_message("Starting the room id signature scan...");
    let mut room_id_address: Option<Address> = None;
    for range in process.memory_ranges().rev() {
        let address = range.address().unwrap().value();
        let size = range.size().unwrap_or_default();

        if let Some(add) = ROOM_ID_SIG.scan_process_range(process, (address, size)) {
            let offset = match process.read::<u32>(Address::new(add.value() + 0x2)) {
                Ok(offset) => offset,
                Err(_) => {
                    asr::print_message("Could not find offset for room id");
                    return Err(());
                }
            };
            room_id_address = Some(Address::new(
                add.value() + 0x6 + offset as u64 - main_address.value(),
            ));
            break;
        }
    }

    match room_id_address {
        Some(address) => {
            let mut buffer = itoa::Buffer::new();
            asr::timer::set_variable(
                "Room Id Address",
                buffer.format(room_id_address.unwrap().value()),
            );
            asr::print_message("Room ID signature scan complete.");
            Ok(address)
        }
        None => {
            asr::print_message("Could NOT complete the room ID scan.");
            Err(())
        }
    }
}

pub fn buffer_helper_sigscan_init(process: &asr::Process) -> Result<asr::Address, ()> {
    asr::print_message("Starting the helper buffer signature scan...");

    let mut helper_address: Option<Address> = None;

    for range in process.memory_ranges() {
        let address = range.address().unwrap_or_default().value();
        let size = range.size().unwrap_or_default();
        if let Some(address) = BUFFER_MAGIC_NUMBER.scan_process_range(process, (address, size)) {
            helper_address = Some(address);
            break;
        }
    }

    // this is a direct reference to the speedrun data, finding the scanned address is enough
    if let Some(add) = helper_address {
        let mut buffer = itoa::Buffer::new();
        asr::timer::set_variable(
            "Buffer address",
            buffer.format(helper_address.unwrap_or(Address::new(0)).value()),
        );
        asr::print_message("Buffer sigscan complete");
        Ok(add)
    } else {
        asr::print_message("Could not complete the buffer helper sigscan. Is the \"-livesplit\" launch option set?");
        asr::print_message("Continuing with the basic real time and split features.");
        Err(())
    }
}

pub fn room_name_array_sigscan_start(process: &asr::Process) -> Result<asr::Address, &str> {
    asr::print_message("Starting the name array signature scan...");
    let mut pointer_to_rooms_array: Option<Address> = None;
    // get pointer scan add -> read u32 5 bytes after the result to find offset -> result is add scanned + 9 + offset
    for range in process.memory_ranges().rev() {
        let address = range.address().unwrap_or_default().value();
        let size = range.size().unwrap_or_default();

        if let Some(add) = ROOM_ID_ARRAY_SIG.scan_process_range(process, (address, size)) {
            let offset = match process.read::<u32>(Address::new(add.value() + 0x5)) {
                Ok(pointer) => pointer,
                Err(_) => return Err("Could not read offset to find the room names array"),
            };
            pointer_to_rooms_array = Some(Address::new(add.value() + 0x9 + offset as u64));
            break;
        };
    }

    match pointer_to_rooms_array {
        Some(address) => match process.read::<u64>(address) {
            Ok(add) => {
                asr::print_message("Room name array signature scan complete.");
                let mut buffer = itoa::Buffer::new();
                asr::timer::set_variable("Room names array", buffer.format(address.value()));
                Ok(Address::new(add))
            }
            Err(_) => Err("Could not read the array address"),
        },
        None => Err("Could not find signature for room names array"),
    }
}

pub fn refresh_mem_values<'a>(
    process: &'a Process,
    memory_addresses: &'a MemoryAddresses,
    memory_values: &mut MemoryValues,
) -> Result<(), &'a str> {
    let main_address;
    if let Some(address) = memory_addresses.main_address {
        main_address = address;
    } else {
        return Err("Main Address is None in refresh mem values function");
    }

    if let Ok(value) = process.read::<i32>(Address::new(
        memory_addresses.room_id.unwrap_or(Address::new(0)).value() + main_address.value(),
    )) {
        update_pair_int("Room ID", value, &mut memory_values.room_id);
    } else {
        return Err("Could not read the room ID from memory");
    }

    // only update if buffer helper was found
    if memory_addresses.buffer_helper.is_some() {
        /*
        Buffer documentation:
        0x00: magic numbers
        0x40: game version (string)
        0x80: file minutes (f64)
        0x88: file seconds (f64)
        0x90: level minute (f64)
        0x98: level seconds (f64)
        0xA0: current room (string)
        0xE0: end of level fade exists (u8 (bool))
        0xE1: boss HP (u8)
        */

        let buffer_helper_add = memory_addresses
            .buffer_helper
            .unwrap_or(Address::new(0))
            .value();

        // game version doesn't need to be updated more tha once...
        if memory_values.game_version.current == ArrayCString::default() {
            let game_version = buffer_helper_add + 0x40;

            if let Ok(value) = process.read::<ArrayCString<ROOM_NAME_SIZE_CAP>>(game_version) {
                update_pair_cstring("Game Version", value, &mut memory_values.game_version);
            }
        }

        let file_minutes_add = buffer_helper_add + 0x80;
        let file_seconds_add = buffer_helper_add + 0x88;
        let level_minutes_add = buffer_helper_add + 0x90;
        let level_seconds_add = buffer_helper_add + 0x98;
        let room_add = buffer_helper_add + 0xA0;
        let end_level_fade_add = buffer_helper_add + 0xE0;
        let boss_hp_add = buffer_helper_add + 0xE1;

        if let Ok(value) = process.read::<f64>(Address::new(file_seconds_add)) {
            update_pair_f64("File Seconds", value, &mut memory_values.file_seconds);
        };

        if let Ok(value) = process.read::<f64>(Address::new(file_minutes_add)) {
            update_pair_f64("File Minutes", value, &mut memory_values.file_minutes);
        };

        if let Ok(value) = process.read::<f64>(Address::new(level_seconds_add)) {
            update_pair_f64("Level Seconds", value, &mut memory_values.level_seconds);
        };

        if let Ok(value) = process.read::<f64>(Address::new(level_minutes_add)) {
            update_pair_f64("Level Minutes", value, &mut memory_values.level_minutes);
        };

        if let Ok(value) = process.read::<ArrayCString<ROOM_NAME_SIZE_CAP>>(room_add) {
            update_pair_cstring("Room Name (Buffer)", value, &mut memory_values.room_name);
        }

        if let Ok(value) = process.read::<u8>(Address::new(end_level_fade_add)) {
            update_pair_int("End Fade Exists", value, &mut memory_values.end_of_level);
        };

        if let Ok(value) = process.read::<u8>(Address::new(boss_hp_add)) {
            update_pair_int("Boss HP", value, &mut memory_values.boss_hp);
        };
    } else {
        // with the current room id value as an offset, find its name in the array
        let curr_room_name_add = process.read::<u64>(Address::new(
            memory_addresses
                .room_names
                .unwrap_or(Address::new(0))
                .value()
                + memory_values.room_id.current as u64 * 0x8,
        ));

        match curr_room_name_add {
            Ok(add) => {
                if let Ok(value) = process.read::<ArrayCString<ROOM_NAME_SIZE_CAP>>(add) {
                    update_pair_cstring(
                        "Room Name (GM Array)",
                        value,
                        &mut memory_values.room_name,
                    );
                }
            }
            Err(_) => return Err("Could not read the room address, retrying signature scan..."),
        };
    }

    Ok(())
}
