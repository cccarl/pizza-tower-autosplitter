#![no_std]

use asr::print_message;
use asr::string::ArrayCString;
use asr::{future::next_tick, settings::Gui, watcher::Pair, Process};
use memory::{refresh_mem_values, ROOM_NAME_SIZE_CAP};

mod memory;
mod room_names;
mod settings;

asr::async_main!(stable);
asr::panic_handler!();

const MAIN_MODULE: &str = "PizzaTower.exe";

#[derive(Default, Clone)]
struct MemoryAddresses {
    main_address: Option<asr::Address>,
    room_id: Option<asr::Address>,
    room_names: Option<asr::Address>,
    buffer_helper: Option<asr::Address>,
}

#[derive(Default)]
struct MemoryValues {
    game_version: Pair<ArrayCString<0x40>>,
    room_id: Pair<i32>,
    room_name: Pair<ArrayCString<ROOM_NAME_SIZE_CAP>>,
    file_seconds: Pair<f64>,
    file_minutes: Pair<f64>,
    level_seconds: Pair<f64>,
    level_minutes: Pair<f64>,
    end_of_level: Pair<u8>,
    boss_hp: Pair<u8>,
}

async fn main() {
    let mut settings = settings::Settings::register();
    let mut mem_addresses = MemoryAddresses::default();
    let mut mem_values = MemoryValues::default();
    asr::set_tick_rate(240.0);

    loop {
        // check if settings GUI changes
        settings.update();
        if settings.timer_mode.changed() {
            settings.load_default_settings_for_mode();
        }

        let process_option = Process::attach(MAIN_MODULE);

        let process;
        match process_option {
            Some(process_found) => {
                process = process_found;
                mem_addresses.main_address = Some(process.get_module_address(MAIN_MODULE).unwrap());
                print_message("Connected to Pizza Tower the pizzapasta game!!!");
            }
            None => {
                next_tick().await;
                continue;
            }
        }

        process
            .until_closes(async {

                // init
                if let Ok(address) = memory::room_id_sigscan_start(&process, mem_addresses.clone()) {
                    mem_addresses.room_id = Some(address);
                } else {
                    mem_addresses.room_id = None;
                }

                if mem_addresses.room_id.is_some() {
                    if let Ok(room_id_result) = process.read(mem_addresses.main_address.unwrap_or(asr::Address::default()).value() + mem_addresses.room_id.unwrap().value()) {
                        mem_values.room_id.current = room_id_result
                    } else {
                        mem_values.room_id.current = 0;
                        print_message("Could not read room ID before stall that waits for the game opening. Using 0");
                    }
                    if mem_values.room_id.current == 0 {
                        print_message("Waiting for the game to start...");
                    }
                    while mem_values.room_id.current == 0 {
                        if mem_values.room_id.current == 0 {
                            if let Ok(value) = process.read::<i32>(mem_addresses.main_address.unwrap_or(asr::Address::default()).value() + mem_addresses.room_id.unwrap().value()) {
                                mem_values.room_id.current = value;
                            } else {
                                break;
                            }
                        }
                    }
                }

                mem_addresses.buffer_helper = match memory::buffer_helper_sigscan_init(&process) {
                    Ok(address) => Some(address),
                    Err(_) => None,
                };

                // not needed if helper was found
                if mem_addresses.buffer_helper.is_none() {
                    mem_addresses.room_names = match memory::room_name_array_sigscan_start(&process) {
                        Ok(address) => Some(address),
                    Err(_) => None,
                    };
                }

                // ready for main loop
                if mem_addresses.room_names.is_some() || mem_addresses.buffer_helper.is_some() {

                    // variables declaration for the main loop
                    let mut current_level = room_names::Level::Unknown;
                    let mut igt_file_secs_calculated: Pair<f64> = Pair::default();
                    let mut igt_level_secs_calculated: Pair<f64> = Pair::default();

                    let mut ng_plus_offset_seconds: Option<f64> = None;
                    let mut iw_offset_seconds: Option<f64> = None;

                    let mut enable_full_game_split = false;
                    let mut ctop_oob_split = false; // should only happen once per run

                    let mut last_room_split_name = ArrayCString::<ROOM_NAME_SIZE_CAP>::new();
                    let mut last_room_split_time = 0.0;

                    loop {
                        settings.update();
                        if settings.timer_mode.changed() {
                            settings.load_default_settings_for_mode();
                        }

                        if let Err(text) = refresh_mem_values(&process, &mem_addresses, &mut mem_values) {
                            print_message(text);
                            print_message("Exiting main loop and retrying...");
                            break;
                        }

                        // TODO: Do something on every tick.
                        next_tick().await;
                    }
                }
            })
            .await;
    }
}
