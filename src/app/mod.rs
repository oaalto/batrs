mod dialogs;
mod input_state;
mod output_buffer;
mod player_logger;
mod session_state;
mod telnet_buffer;
mod util;

use crate::ansi::StyledLine;
use crate::automation::{Action, Automation};
use crate::config::{ConfigManager, GenericCommandsConfig, SettingEntry, UserSettings};
use crate::generic_commands::GenericCommands;
use crate::guilds::{Guild, build_guilds, guild_definitions};
use crate::stats::Stats;
use crate::ui::{Renderer, ViewModel};
use crate::{command, triggers};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use dialogs::{GenericCommandsDialog, GuildDialog, SettingsDialog, apply_guild_dialog_keystroke};
use input_state::InputState;
use libmudtelnet::events::TelnetEvents;
use player_logger::PlayerLogger;
use ratatui::Frame;
use ratatui::text::Line;
use std::sync::mpsc::{Receiver, Sender};
use telnet_buffer::TelnetBuffer;
use util::{filter_known_guilds, show_clock};

use output_buffer::OutputBuffer;
use session_state::{LoginState, SessionState};

pub struct BatApp {
    output: OutputBuffer,
    input: InputState,
    session: SessionState,
    stats: Stats,
    event_receiver: Receiver<TelnetEvents>,
    command_sender: Sender<String>,
    telnet_buffer: TelnetBuffer,
    selected_guilds: Vec<Box<dyn Guild>>,
    selected_guild_keys: Vec<String>,
    should_quit: bool,
    automation: Automation,
    config_manager: Option<ConfigManager>,
    user_config_loaded: bool,
    user_rig: Option<String>,
    guild_dialog: Option<GuildDialog>,
    generic_commands: GenericCommands,
    generic_commands_dialog: Option<GenericCommandsDialog>,
    settings_dialog: Option<SettingsDialog>,
    player_logger: Option<PlayerLogger>,
}

impl BatApp {
    pub fn new(event_receiver: Receiver<TelnetEvents>, command_sender: Sender<String>) -> Self {
        let config_manager = match ConfigManager::new() {
            Ok(mut manager) => {
                if let Err(e) = manager.init_base() {
                    eprintln!("failed to initialize base config: {e}");
                }
                Some(manager)
            }
            Err(e) => {
                eprintln!("failed to initialize config manager: {e}");
                None
            }
        };
        let mut app = BatApp {
            output: OutputBuffer::new(),
            input: InputState::new(),
            session: SessionState::new(),
            stats: Default::default(),
            event_receiver,
            command_sender,
            telnet_buffer: TelnetBuffer::new(),
            selected_guilds: Vec::new(),
            selected_guild_keys: Vec::new(),
            should_quit: false,
            automation: Automation::new(),
            config_manager,
            user_config_loaded: false,
            user_rig: None,
            guild_dialog: None,
            generic_commands: GenericCommands::default(),
            generic_commands_dialog: None,
            settings_dialog: None,
            player_logger: PlayerLogger::new().ok(),
        };

        app.apply_selected_guilds(app.selected_guild_keys.clone());

        app
    }

    fn process_input_lines(&mut self, lines: Vec<String>) {
        let mut output_lines = Vec::new();

        for line in lines {
            let mut styled_line = StyledLine::new(&line);
            let was_logged_in = self.session.is_logged_in();
            if self.session.update_login_state(&styled_line.plain_line) {
                self.input.clear_all();
            }
            if !was_logged_in && self.session.is_logged_in() {
                self.load_user_config();
            }
            if let Some(player_name) = self.session.login_name()
                && let Some(logger) = self.player_logger.as_mut()
            {
                logger.set_player_name(player_name);
            }
            if let Some(logger) = self.player_logger.as_mut()
                && let Err(e) = logger.log_line(&styled_line.plain_line)
            {
                eprintln!("failed to log line: {e}");
            }

            if self.session.is_logged_in() {
                let mut ctx = triggers::TriggerContext {
                    stats: &mut self.stats,
                    automation: &mut self.automation,
                    rig: self.user_rig.as_deref(),
                };
                let result = triggers::process(&mut ctx, &self.selected_guilds, &mut styled_line);
                self.apply_automation_actions(result.actions);
                let mut new_lines = result.lines;
                new_lines.push(styled_line);
                output_lines.extend(new_lines);
            } else {
                output_lines.push(styled_line);
            }
        }

        if self.session.is_logged_in() {
            for line in &output_lines {
                self.run_automation(&line.plain_line);
            }
        }

        self.output.append_lines(output_lines);
    }

    pub fn read_input(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            let lines = self.telnet_buffer.handle_event(&event);
            if !lines.is_empty() {
                self.process_input_lines(lines);
            }
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        if self.guild_dialog.is_some() {
            self.handle_guild_dialog_event(event);
            return;
        }
        if self.generic_commands_dialog.is_some() {
            self.handle_generic_commands_dialog_event(event);
            return;
        }
        if self.settings_dialog.is_some() {
            self.handle_settings_dialog_event(event);
            return;
        }
        match event.code {
            KeyCode::Enter => self.submit_input(),
            KeyCode::Backspace => self.input.backspace(),
            KeyCode::Up if self.session.is_logged_in() => {
                self.input.move_history(-1);
            }
            KeyCode::Down if self.session.is_logged_in() => {
                self.input.move_history(1);
            }
            KeyCode::Char(c)
                if !event.modifiers.contains(KeyModifiers::CONTROL)
                    && !event.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.input.insert_char(c);
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        let show_stats = self.session.is_logged_in();
        let soul_supported = self
            .selected_guild_keys
            .iter()
            .any(|key| key.as_str() == "animist")
            || self.stats.has_soul_companion_status();
        let show_soul_stats = show_stats && soul_supported;
        let reserved_rows = if show_soul_stats { 3 } else { 2 };
        let output_area_height = frame.area().height.saturating_sub(reserved_rows);
        let output_area_width = frame.area().width;
        let visible_height = output_area_height.saturating_sub(1) as usize;
        let output_lines: Vec<Line<'_>> = self.output.wrapped_lines(output_area_width);
        let scroll_offset = output_lines.len().saturating_sub(visible_height);
        let scroll_offset = scroll_offset.min(u16::MAX as usize) as u16;
        let stats_line = if show_stats {
            self.stats.render_inline()
        } else {
            Line::from("")
        };
        let soul_stats_line = if show_soul_stats {
            self.stats.render_soul_inline()
        } else {
            Line::from("")
        };
        let hide_input = self.session.login_state() == LoginState::Password;
        let input_text = format!("> {}", self.input.displayed_text(hide_input));
        let view = ViewModel {
            output_lines,
            scroll_offset,
            show_stats,
            stats_line,
            show_soul_stats,
            soul_stats_line,
            clock: show_clock(),
            input_text,
            cursor_offset: self.input.cursor_offset(hide_input),
            show_cursor: self.guild_dialog.is_none()
                && self.generic_commands_dialog.is_none()
                && self.settings_dialog.is_none(),
            guild_dialog: self.guild_dialog.as_ref().map(|dialog| dialog.view_model()),
            generic_commands_dialog: self
                .generic_commands_dialog
                .as_ref()
                .map(|dialog| dialog.view_model()),
            settings_dialog: self
                .settings_dialog
                .as_ref()
                .map(|dialog| dialog.view_model()),
        };

        Renderer::render(frame, &view);
    }

    fn submit_input(&mut self) {
        if !self.session.is_logged_in() {
            let input = self.input.take_displayed_input();
            if input.is_empty() {
                return;
            }

            if input.starts_with('/') {
                let mut ctx = command::CommandContext::new(self.automation.snapshot_flags(), false);
                let outcome = command::process(
                    &input,
                    &mut ctx,
                    &self.selected_guilds,
                    &self.generic_commands,
                );
                if outcome.should_quit {
                    self.should_quit = true;
                    return;
                }
                if outcome.open_guilds_dialog {
                    self.open_guilds_dialog();
                }
                if outcome.open_generic_commands_dialog {
                    self.open_generic_commands_dialog();
                }
                if outcome.open_settings_dialog {
                    self.open_settings_dialog();
                }
                if let Some(rig) = outcome.set_rig {
                    self.update_user_rig(rig);
                }
                if !outcome.output_lines.is_empty() {
                    self.output.append_lines(outcome.output_lines);
                }
                if let Some(s) = outcome.send {
                    self.send_command(s);
                }
            } else {
                if self.session.login_state() == LoginState::Name {
                    self.session.set_login_name(input.clone());
                }
                if self.session.login_state() == LoginState::Choice {
                    self.session.set_last_login_input(input.clone());
                }
                self.send_command(input);
            }

            self.input.clear_current_typed_input();
            return;
        }

        let mut ctx = command::CommandContext::new(self.automation.snapshot_flags(), true);
        let outcome = command::process(
            self.input.displayed_input(),
            &mut ctx,
            &self.selected_guilds,
            &self.generic_commands,
        );

        if outcome.should_quit {
            self.should_quit = true;
            return;
        }

        if outcome.open_guilds_dialog {
            self.open_guilds_dialog();
        }
        if outcome.open_generic_commands_dialog {
            self.open_generic_commands_dialog();
        }
        if outcome.open_settings_dialog {
            self.open_settings_dialog();
        }

        self.apply_automation_actions(outcome.automation_actions);
        if let Some(rig) = outcome.set_rig {
            self.update_user_rig(rig);
        }
        if !outcome.output_lines.is_empty() {
            self.output.append_lines(outcome.output_lines);
        }

        if let Some(s) = outcome.send {
            self.send_command(s);
        }

        let input = self.input.take_displayed_input();
        self.input.push_history(input);
    }

    fn run_automation(&mut self, line: &str) {
        for cmd in self.automation.process_line(line) {
            self.send_command(cmd);
        }
    }

    fn apply_automation_actions(&mut self, actions: Vec<Action>) {
        for cmd in self.automation.apply_actions(actions) {
            self.send_command(cmd);
        }
    }

    fn update_user_rig(&mut self, rig: String) {
        if !self.user_config_loaded {
            self.load_user_config();
        }
        let entries = if let Some(manager) = self.config_manager.as_mut() {
            match manager.user_settings() {
                Ok(settings) => {
                    let mut entries = settings.entries;
                    if let Some(entry) = entries.iter_mut().find(|entry| entry.key == "rig") {
                        entry.value = rig.clone();
                    } else {
                        entries.push(SettingEntry {
                            key: "rig".to_string(),
                            value: rig.clone(),
                        });
                    }
                    entries
                }
                Err(e) => {
                    eprintln!("invalid settings config: {e}");
                    std::process::exit(1);
                }
            }
        } else {
            vec![SettingEntry {
                key: "rig".to_string(),
                value: rig.clone(),
            }]
        };
        self.apply_user_settings(UserSettings { entries });
    }

    fn apply_user_settings(&mut self, settings: UserSettings) {
        let rig = settings.get("rig").unwrap_or_default().to_string();
        self.user_rig = Some(rig.clone());
        self.automation.set_var("rig", rig);
        let Some(manager) = self.config_manager.as_mut() else {
            return;
        };
        if let Err(e) = manager.save_user_settings(&settings) {
            eprintln!("failed to save user settings: {e}");
        }
    }

    fn apply_selected_guilds(&mut self, keys: Vec<String>) {
        self.selected_guild_keys = keys;
        self.selected_guilds = build_guilds(&self.selected_guild_keys);
        self.automation = Automation::new();
        for guild in &self.selected_guilds {
            guild.register_automation(&mut self.automation);
        }
        self.automation.set_flag("in_battle", false);
        if let Some(rig) = self.user_rig.as_ref()
            && !rig.is_empty()
        {
            self.automation.set_var("rig", rig.clone());
        }

        // Set tzarakk mount variable if configured
        if let Some(manager) = self.config_manager.as_mut()
            && let Ok(settings) = manager.user_settings()
            && let Some(mount) = settings.get("tzarakk_mount")
            && !mount.is_empty()
        {
            self.automation.set_var("tzarakk_mount", mount.to_string());
        }
    }

    fn open_guilds_dialog(&mut self) {
        if !self.session.is_logged_in() {
            return;
        }
        let definitions = guild_definitions();
        let selected = definitions
            .iter()
            .map(|def| self.selected_guild_keys.iter().any(|key| key == def.key))
            .collect();

        // Load mount name from config
        let mount_name = if self.user_config_loaded {
            if let Some(manager) = self.config_manager.as_mut() {
                if let Ok(settings) = manager.user_settings() {
                    settings.get("tzarakk_mount").unwrap_or("").to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        self.guild_dialog = Some(GuildDialog::new(definitions, selected, mount_name));
    }

    fn open_settings_dialog(&mut self) {
        if !self.session.is_logged_in() {
            return;
        }
        if !self.user_config_loaded {
            self.load_user_config();
        }
        let entries = {
            let Some(manager) = self.config_manager.as_mut() else {
                return;
            };
            match manager.user_settings() {
                Ok(settings) => settings.entries,
                Err(e) => {
                    eprintln!("invalid settings config: {e}");
                    std::process::exit(1);
                }
            }
        };
        self.settings_dialog = Some(SettingsDialog::new(entries));
    }

    fn handle_guild_dialog_event(&mut self, event: KeyEvent) {
        let Some(dialog) = self.guild_dialog.as_mut() else {
            return;
        };
        match event.code {
            KeyCode::Esc => {
                self.guild_dialog = None;
            }
            KeyCode::Enter => {
                let keys = dialog.selected_keys();
                let mount_name = dialog.mount_name();
                self.guild_dialog = None;
                self.apply_selected_guilds(keys.clone());
                self.save_selected_guilds_with_mount(keys, mount_name);
            }
            _ => {
                apply_guild_dialog_keystroke(dialog, event);
            }
        }
    }

    fn handle_settings_dialog_event(&mut self, event: KeyEvent) {
        let Some(dialog) = self.settings_dialog.as_mut() else {
            return;
        };
        match event.code {
            KeyCode::Up => dialog.move_cursor(-1),
            KeyCode::Down => dialog.move_cursor(1),
            KeyCode::Backspace => dialog.backspace(),
            KeyCode::Char(c)
                if !event.modifiers.contains(KeyModifiers::CONTROL)
                    && !event.modifiers.contains(KeyModifiers::ALT) =>
            {
                dialog.insert_char(c);
            }
            KeyCode::Esc => {
                self.settings_dialog = None;
            }
            KeyCode::Enter => {
                let entries = dialog.entries();
                self.settings_dialog = None;
                self.apply_user_settings(UserSettings { entries });
            }
            _ => {}
        }
    }

    fn open_generic_commands_dialog(&mut self) {
        if !self.session.is_logged_in() {
            return;
        }
        if !self.user_config_loaded {
            self.load_user_config();
        }

        // Load saved configuration
        let config = self
            .config_manager
            .as_ref()
            .map(|m| m.generic_commands_config())
            .unwrap_or_default();

        // Apply configuration to generic commands
        self.generic_commands
            .apply_config(&config.enabled_groups, &config.disabled_commands);

        self.generic_commands_dialog = Some(GenericCommandsDialog::new(&self.generic_commands));
    }

    fn handle_generic_commands_dialog_event(&mut self, event: KeyEvent) {
        let Some(dialog) = self.generic_commands_dialog.as_mut() else {
            return;
        };
        match event.code {
            KeyCode::Up => dialog.move_cursor(-1),
            KeyCode::Down => dialog.move_cursor(1),
            KeyCode::Char(' ') => dialog.toggle_selected(),
            KeyCode::Esc => {
                self.generic_commands_dialog = None;
            }
            KeyCode::Enter => {
                let (enabled_groups, disabled_commands) = dialog.to_config();
                self.generic_commands_dialog = None;
                self.save_generic_commands(enabled_groups, disabled_commands);
            }
            _ => {}
        }
    }

    fn save_generic_commands(
        &mut self,
        enabled_groups: Vec<String>,
        disabled_commands: Vec<String>,
    ) {
        if !self.user_config_loaded {
            self.load_user_config();
        }
        let Some(manager) = self.config_manager.as_mut() else {
            return;
        };

        let config = GenericCommandsConfig {
            enabled_groups,
            disabled_commands,
        };

        // Update in-memory generic commands
        self.generic_commands
            .apply_config(&config.enabled_groups, &config.disabled_commands);

        if let Err(e) = manager.save_generic_commands(&config) {
            eprintln!("failed to save generic commands config: {e}");
        }
    }

    fn save_selected_guilds_with_mount(&mut self, keys: Vec<String>, mount_name: String) {
        if !self.user_config_loaded {
            self.load_user_config();
        }
        let Some(manager) = self.config_manager.as_mut() else {
            return;
        };

        // Save guilds
        if let Err(e) = manager.save_user_guilds(&keys) {
            eprintln!("failed to save user guilds: {e}");
        }

        // Save mount name
        if let Err(e) = manager.save_user_setting("tzarakk_mount", &mount_name) {
            eprintln!("failed to save tzarakk mount name: {e}");
        }
    }

    fn send_command(&mut self, command: String) {
        if let Err(e) = self.command_sender.send(command) {
            eprintln!("failed to send data: {e}");
        }
    }

    fn load_user_config(&mut self) {
        if self.user_config_loaded {
            return;
        }
        self.user_config_loaded = true;
        let Some(player_name) = self.session.login_name() else {
            eprintln!("logged in without a known player name; skipping user config");
            return;
        };
        let (guild_keys, settings, generic_config) = {
            let Some(manager) = self.config_manager.as_mut() else {
                return;
            };
            if let Err(e) = manager.load_user(player_name) {
                eprintln!("failed to load user config for {player_name}: {e}");
                return;
            }
            let settings = match manager.user_settings() {
                Ok(settings) => Some(settings),
                Err(e) => {
                    eprintln!("invalid settings config: {e}");
                    std::process::exit(1);
                }
            };
            let generic_config = manager.generic_commands_config();
            (manager.user_guilds(), settings, generic_config)
        };

        if let Some(keys) = guild_keys {
            self.apply_selected_guilds(filter_known_guilds(keys));
        }

        if let Some(settings) = settings {
            self.user_rig = settings.get("rig").map(|rig| rig.to_string());
            if let Some(rig) = self.user_rig.as_ref()
                && !rig.is_empty()
            {
                self.automation.set_var("rig", rig.clone());
            }

            // Set tzarakk mount variable if configured
            if let Some(mount) = settings.get("tzarakk_mount")
                && !mount.is_empty()
            {
                self.automation.set_var("tzarakk_mount", mount.to_string());
            }
        }

        // Apply generic commands configuration
        self.generic_commands.apply_config(
            &generic_config.enabled_groups,
            &generic_config.disabled_commands,
        );
    }
}
