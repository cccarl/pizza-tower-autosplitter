#![no_std]

use asr::string::ArrayCString;
use asr::time::Duration;
use asr::timer::TimerState;
use asr::{future::next_tick, settings::Gui, watcher::Pair, Process};
use asr::{print_message, timer};
use memory::{refresh_mem_values, ROOM_NAME_SIZE_CAP};
use room_names::Level;
use settings::TimerMode;

mod memory;
mod room_names;
mod settings;

asr::async_main!(stable);
asr::panic_handler!();

const MAIN_MODULE: &str = "PizzaTower.exe";
const TICK_RATE_MAIN_LOOP: f64 = 120.0;
const TICK_RATE_RETRY_ATTACH: f64 = 1.0;

#[derive(Default)]
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
    if settings.timer_mode_load_defaults {
        settings.load_default_settings_for_mode();
    }
    let mut mem_addresses = MemoryAddresses::default();
    let mut mem_values = MemoryValues::default();

    asr::set_tick_rate(TICK_RATE_MAIN_LOOP);

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
                mem_addresses.main_address = match process.get_module_address(MAIN_MODULE) {
                    Ok(address) => Some(address),
                    Err(_) => None,
                };
                print_message("Connected to Pizza Tower the pizzapasta game!!!");
            }
            None => {
                next_tick().await;
                continue;
            }
        }

        process
            .until_closes(async {

                asr::set_tick_rate(TICK_RATE_RETRY_ATTACH);

                // init
                if let Ok(address) = memory::room_id_sigscan_start(&process, &mem_addresses) {
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
                                asr::set_tick_rate(TICK_RATE_MAIN_LOOP);
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

                    asr::set_tick_rate(TICK_RATE_MAIN_LOOP);

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

                        let room_name_parsed_current = mem_values.room_name.current.validate_utf8().unwrap_or("(invalid utf8 string)");
                        let room_name_parsed_old = mem_values.room_name.old.validate_utf8().unwrap_or("(invalid utf8 string)");

                        // update current level and enable full game splits
                        if mem_values.room_name.changed() {
                            current_level = room_names::get_current_level(room_name_parsed_current, current_level);
                            if !enable_full_game_split {
                                enable_full_game_split = room_names::full_game_split_unlock_rooms(room_name_parsed_current);
                            }

                        }

                        timer::set_variable("Current Level", room_names::get_full_level_name(&current_level));

                        // game time set
                        if mem_addresses.buffer_helper.is_some() {

                            igt_file_secs_calculated.old = igt_file_secs_calculated.current;
                            igt_file_secs_calculated.current = mem_values.file_minutes.current * 60.0 + mem_values.file_seconds.current;
                            igt_level_secs_calculated.old =  igt_level_secs_calculated.current;
                            igt_level_secs_calculated.current = mem_values.level_minutes.current * 60.0 + mem_values.level_seconds.current;

                            // offsets for ng+ and iw
                            if timer::state() == TimerState::NotRunning {
                                // ng+ offset update
                                if ng_plus_offset_seconds.is_none() && room_name_parsed_current == "tower_entrancehall" && mem_values.level_minutes.current == 0.0 && mem_values.level_seconds.current < 1.0 {
                                    ng_plus_offset_seconds = Some(igt_file_secs_calculated.current);
                                }
                                if ng_plus_offset_seconds.is_some() && (room_name_parsed_current == "hub_loadingscreen" || room_name_parsed_current == "Finalintro") {
                                    ng_plus_offset_seconds = None;
                                }

                                // iw offset update
                                if iw_offset_seconds.is_none() && current_level == room_names::Level::Hub {
                                    iw_offset_seconds = Some(igt_file_secs_calculated.current);
                                }
                                if iw_offset_seconds.is_some() && current_level != room_names::Level::Hub {
                                    iw_offset_seconds = None;
                                }
                            }

                            // makes the livesplit game time frozen, if not used it stutters when the igt stops advancing
                            timer::pause_game_time();


                            let game_time_livesplit = match settings.timer_mode.current {
                                TimerMode::FullGame => igt_file_secs_calculated.current,
                                TimerMode::IL => igt_level_secs_calculated.current,
                                TimerMode::NewGamePlus => igt_file_secs_calculated.current - ng_plus_offset_seconds.unwrap_or(0.0),
                                TimerMode::IW => igt_file_secs_calculated.current - iw_offset_seconds.unwrap_or(0.0),
                            };
                            timer::set_game_time(Duration::seconds_f64(game_time_livesplit));
                        }

                        // start
                        if settings.start_enable {
                            if settings.start_new_file && room_name_parsed_current == "tower_entrancehall" && room_name_parsed_old == "Finalintro" {
                                timer::start();
                            }
                            if settings.start_any_file && room_name_parsed_current == "tower_entrancehall" && room_name_parsed_old == "hub_loadingscreen" {
                                timer::start();
                            }
                            if settings.start_new_il && room_names::get_starting_room(&current_level) == room_name_parsed_current && igt_level_secs_calculated.current > 0.07 && igt_level_secs_calculated.current <= 0.1 {
                                timer::start();
                            }
                            if settings.start_exit_level && mem_values.room_name.changed() && room_names::full_game_split_rooms(room_name_parsed_old) && current_level == Level::Hub {
                                timer::start();
                            }
                        }

                        // reset
                        if settings.reset_enable {
                            if settings.reset_new_file && room_name_parsed_current == "Finalintro" && room_name_parsed_old != "Finalintro" {
                                timer::reset();
                            }
                            if settings.reset_any_file && mem_values.room_name.changed() && room_name_parsed_current == "hub_loadingscreen" {
                                timer::reset();
                            }
                            if settings.reset_new_level && igt_level_secs_calculated.decreased() && current_level != Level::Hub {
                                last_room_split_time = 0.0;
                                timer::reset();
                            }
                        }

                        // split
                        if settings.splits_enable {

                            // covers any full game split
                            if settings.splits_level_end {

                                // standard level / boss end
                                // got lazy and hardcoded the noise pizzaface split here :)
                                if mem_values.room_name.changed()
                                && room_names::full_game_split_rooms(room_name_parsed_old)
                                && (current_level == Level::Hub || current_level == Level::ResultsScreen)
                                && enable_full_game_split
                                && (mem_values.boss_hp.old == 0 || (room_name_parsed_current == "boss_pizzafacehub" && room_name_parsed_old == "boss_pizzaface")) {
                                    timer::split();
                                    enable_full_game_split = false;
                                }

                                // end of the run frame perfect split, technically the prev "if" could cover this too but frame perfectly splitting at the end is cooler
                                if mem_values.end_of_level.current == 1 && mem_values.end_of_level.old == 0 && room_name_parsed_current == "tower_entrancehall" {
                                    timer::split();
                                }

                                // ctop entering from oob
                                if timer::state() == TimerState::NotRunning && ctop_oob_split {
                                    ctop_oob_split = false;
                                }
                                if room_name_parsed_current == "tower_finalhallway" && room_name_parsed_old == "tower_5" && !ctop_oob_split {
                                    ctop_oob_split = true;
                                    timer::split();
                                }
                            }

                            if settings.splits_rooms
                            && (igt_level_secs_calculated.current - last_room_split_time > 2.0 || mem_values.room_name.current != last_room_split_name)
                            && (mem_values.room_name.changed() || mem_values.end_of_level.current == 1 && mem_values.end_of_level.old == 1) {
                                last_room_split_time = igt_level_secs_calculated.current;
                                last_room_split_name = mem_values.room_name.old;

                                asr::timer::split();
                            }

                        }

                        next_tick().await;
                    }
                } else {
                    next_tick().await;
                    asr::set_tick_rate(TICK_RATE_MAIN_LOOP);
                }
            })
            .await;
    }
}
