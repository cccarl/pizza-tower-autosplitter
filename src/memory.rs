use crate::{MemoryAddresses, MemoryValues};
use asr::{signature::Signature, watcher::Pair, Address, Process};

// the array with all the room names
const ROOM_ID_ARRAY_SIG: Signature<13> = Signature::new("74 0C 48 8B 05 ?? ?? ?? ?? 48 8B 04 D0");
// the id of the current room the player is on (i32)
const ROOM_ID_SIG: Signature<9> = Signature::new("89 3D ?? ?? ?? ?? 48 3B 1D");

// the magic numbers to find for the buffer, using 16 of the 32, good enough
const BUFFER_MAGIC_NUMBER: Signature<16> =
    Signature::new("C2 5A 17 65 BE 4D DF D6 F2 1C D1 3B A7 A6 1F C3");

pub fn room_id_sigscan_start(
    process: &asr::Process,
    addresses: MemoryAddresses,
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
