use asr::settings::gui::Title;
use asr::settings::Gui;
use asr::watcher::Pair;

#[derive(Gui, Clone, Copy, PartialEq, core::cmp::Eq)]
pub enum TimerMode {
    /// Full Game
    #[default]
    FullGame,
    /// Individual Level
    IL,
    /// New Game+
    NewGamePlus,
    /// Individual World
    IW,
}

#[derive(Gui)]
pub struct Settings {
    /// NOTE: Use "-livesplit" as a launch options to use game time.
    _message: Title,

    /// LiveSplit Timer Mode
    _igt_mode: Title,

    /// Pick a Mode
    pub timer_mode: Pair<TimerMode>,

    #[default = true]
    /// Load recommended settings when switching mode
    pub timer_mode_load_defaults: bool,

    /// Start Options
    _timer_mode_title: Title,

    #[default = true]
    /// Enable
    pub start_enable: bool,

    #[default = true]
    /// On opening a new file
    pub start_new_file: bool,

    #[default = false]
    /// On opening any file
    pub start_any_file: bool,

    #[default = false]
    /// On starting a level
    pub start_new_il: bool,

    #[default = false]
    /// On exiting a level
    ///
    /// Useful for individual world runs
    pub start_exit_level: bool,

    /// Split Options
    _splits_title: Title,

    #[default = true]
    /// Enable
    pub splits_enable: bool,

    #[default = true]
    /// On ending a level
    ///
    /// All full game splits, including pizza face
    pub splits_level_end: bool,

    #[default = false]
    /// On room change
    pub splits_rooms: bool,

    /// Reset Options
    _reset_title: Title,

    #[default = true]
    /// Enable
    pub reset_enable: bool,

    #[default = true]
    /// On opening a new file
    pub reset_new_file: bool,

    #[default = true]
    /// On opening any file
    ///
    /// Careful with accidentally exiting to main menu!
    pub reset_any_file: bool,

    #[default = true]
    /// On restarting a level
    pub reset_new_level: bool,
}

impl Settings {
    pub fn load_default_settings_for_mode(&mut self) {
        if !self.timer_mode_load_defaults {
            return;
        }

        let settings_map = asr::settings::Map::load();

        match self.timer_mode.current {
            TimerMode::FullGame => {
                settings_map.insert("start_new_file", true);
                settings_map.insert("start_any_file", false);
                settings_map.insert("start_new_il", false);
                settings_map.insert("start_exit_level", false);

                settings_map.insert("splits_level_end", true);
                settings_map.insert("splits_rooms", false);

                settings_map.insert("reset_new_file", true);
                settings_map.insert("reset_any_file", false);
                settings_map.insert("reset_new_level", false);
            }
            TimerMode::IL => {
                settings_map.insert("start_new_file", false);
                settings_map.insert("start_any_file", false);
                settings_map.insert("start_new_il", true);
                settings_map.insert("start_exit_level", false);

                settings_map.insert("splits_level_end", true);
                settings_map.insert("splits_rooms", true);

                settings_map.insert("reset_new_file", true);
                settings_map.insert("reset_any_file", true);
                settings_map.insert("reset_new_level", true);
            }
            TimerMode::NewGamePlus => {
                settings_map.insert("start_new_file", false);
                settings_map.insert("start_any_file", true);
                settings_map.insert("start_new_il", false);
                settings_map.insert("start_exit_level", false);

                settings_map.insert("splits_level_end", true);
                settings_map.insert("splits_rooms", false);

                settings_map.insert("reset_new_file", true);
                settings_map.insert("reset_any_file", true);
                settings_map.insert("reset_new_level", false);
            }
            TimerMode::IW => {
                settings_map.insert("start_new_file", true);
                settings_map.insert("start_any_file", false);
                settings_map.insert("start_new_il", false);
                settings_map.insert("start_exit_level", true);

                settings_map.insert("splits_level_end", true);
                settings_map.insert("splits_rooms", false);

                settings_map.insert("reset_new_file", true);
                settings_map.insert("reset_any_file", true);
                settings_map.insert("reset_new_level", false);
            }
        }

        settings_map.store();
    }
}
