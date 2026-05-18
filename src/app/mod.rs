mod dialogs;
mod input_state;
mod output_buffer;
mod player_logger;
mod raw_logger;
mod scrollback;
mod session_state;
mod telnet_buffer;
mod util;

use crate::ansi::StyledLine;
use crate::automation::{Action, Automation};
use crate::config::{ConfigManager, GenericCommandsConfig, UserSettings};
use crate::generic_commands::GenericCommands;
use crate::guilds::{
    Guild,
    catalog::{self, GuildKey, GuildSelection},
    grouping::{DEFAULT_GUILD_PRIMARY_KEYWORD, thematic_index_for_keyword},
};
use crate::player_profile::{self, PlayerRuntimeProfile};
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
use raw_logger::RawLogger;
use std::sync::mpsc::{Receiver, Sender};
use telnet_buffer::TelnetBuffer;
use util::show_clock;

use output_buffer::OutputBuffer;
use scrollback::Scrollback;
use session_state::{LoginState, SessionState};

pub enum AppEvent {
    RawInput(Vec<u8>),
    Telnet(TelnetEvents),
}

pub struct BatApp {
    output: OutputBuffer,
    input: InputState,
    session: SessionState,
    stats: Stats,
    event_receiver: Receiver<AppEvent>,
    command_sender: Sender<String>,
    telnet_buffer: TelnetBuffer,
    selected_guilds: Vec<Box<dyn Guild>>,
    guild_selection: GuildSelection,
    should_quit: bool,
    automation: Automation,
    config_manager: Option<ConfigManager>,
    user_config_loaded: bool,
    player_profile: PlayerRuntimeProfile,
    guild_dialog: Option<GuildDialog>,
    generic_commands: GenericCommands,
    generic_commands_dialog: Option<GenericCommandsDialog>,
    settings_dialog: Option<SettingsDialog>,
    player_logger: Option<PlayerLogger>,
    raw_logger: Option<RawLogger>,
    scrollback: Scrollback,
}

impl BatApp {
    pub fn new(event_receiver: Receiver<AppEvent>, command_sender: Sender<String>) -> Self {
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
            guild_selection: GuildSelection::default(),
            should_quit: false,
            automation: Automation::new(),
            config_manager,
            user_config_loaded: false,
            player_profile: PlayerRuntimeProfile::default(),
            guild_dialog: None,
            generic_commands: GenericCommands::default(),
            generic_commands_dialog: None,
            settings_dialog: None,
            player_logger: PlayerLogger::new().ok(),
            raw_logger: RawLogger::new().ok(),
            scrollback: Scrollback::new(),
        };

        app.apply_guild_selection(app.guild_selection.clone());

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
                self.stats.set_recovery_bracket_defaults_for_login();
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
                let facts = triggers::TriggerFacts::new(
                    self.automation.snapshot_flags(),
                    self.automation.snapshot_vars(),
                    self.player_profile.settings.rig_for_triggers(),
                    self.session.login_name(),
                );
                let result =
                    triggers::process(&facts, &self.selected_guilds, &styled_line.plain_line);
                self.apply_stats_effects(result.stats);
                self.apply_automation_actions(result.actions);
                let mut new_lines = result.lines;
                self.apply_original_line_effects(&mut styled_line, result.original);
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
            match event {
                AppEvent::RawInput(bytes) => {
                    if let Some(logger) = self.raw_logger.as_mut()
                        && let Err(e) = logger.write_bytes(&bytes)
                    {
                        eprintln!("failed to write raw log: {e}");
                    }
                }
                AppEvent::Telnet(telnet_event) => {
                    let lines = self.telnet_buffer.handle_event(&telnet_event);
                    if !lines.is_empty() {
                        self.process_input_lines(lines);
                    }
                }
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
            KeyCode::PageUp => self.scrollback.page_up(),
            KeyCode::PageDown => self.scrollback.page_down(),
            KeyCode::Backspace => self.input.backspace(),
            KeyCode::Home => self.input.move_cursor_to_start(),
            KeyCode::End => self.input.move_cursor_to_end(),
            KeyCode::Left if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input.move_cursor_word_left();
            }
            KeyCode::Left => self.input.move_cursor_left(),
            KeyCode::Right if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input.move_cursor_word_right();
            }
            KeyCode::Right => self.input.move_cursor_right(),
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
        let soul_supported = self.guild_selection.is_selected(GuildKey::Animist)
            || self.stats.has_soul_companion_status();
        let riftwalker_supported = self.guild_selection.is_selected(GuildKey::Riftwalker)
            || self.stats.has_riftwalker_entity_status();
        let nergal_supported =
            self.guild_selection.is_selected(GuildKey::Nergal) || self.stats.has_nergal_minions();
        let tzarakk_supported = self.guild_selection.is_selected(GuildKey::Tzarakk)
            || self.stats.has_tzarakk_mount_status();

        let mut secondary_status_lines: Vec<Line<'static>> = Vec::new();
        if show_stats && soul_supported {
            secondary_status_lines.push(self.stats.render_soul_inline());
        }
        if show_stats && riftwalker_supported {
            secondary_status_lines.push(self.stats.render_riftwalker_entity_inline());
        }
        if show_stats && tzarakk_supported {
            secondary_status_lines.push(self.stats.render_tzarakk_mount_inline());
        }
        if show_stats && nergal_supported {
            secondary_status_lines
                .extend(self.stats.render_nergal_minion_lines(frame.area().width));
        }

        let reserved_rows = 2 + secondary_status_lines.len() as u16;
        let output_area_height = frame.area().height.saturating_sub(reserved_rows);
        let output_area_width = frame.area().width;
        let visible_height = output_area_height as usize;
        let output_lines: Vec<Line<'_>> = self.output.wrapped_lines(output_area_width);
        self.scrollback
            .update_viewport(output_lines.len(), visible_height);
        let scroll_offset = self.scrollback.offset();
        let stats_line = if show_stats {
            self.stats.render_inline()
        } else {
            Line::from("")
        };
        let hide_input = self.session.login_state() == LoginState::Password;
        let input_text = format!(">{}", self.input.displayed_text(hide_input));
        let view = ViewModel {
            output_lines,
            scroll_offset,
            show_stats,
            stats_line,
            secondary_status_lines,
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
                let effects = command::dispatch(
                    command::CommandDispatchInput::new(
                        &input,
                        false,
                        self.automation.snapshot_flags(),
                        self.automation.snapshot_vars(),
                    ),
                    &self.selected_guilds,
                    &self.generic_commands,
                );
                if self.apply_command_effects(effects) {
                    self.scrollback.follow_latest();
                }
            } else {
                if self.session.login_state() == LoginState::Name {
                    self.session.set_login_name(input.clone());
                }
                if self.session.login_state() == LoginState::Choice {
                    self.session.set_last_login_input(input.clone());
                }
                if self.send_command(input) {
                    self.scrollback.follow_latest();
                }
            }

            self.input.clear_current_typed_input();
            return;
        }

        let effects = command::dispatch(
            command::CommandDispatchInput::new(
                self.input.displayed_input(),
                true,
                self.automation.snapshot_flags(),
                self.automation.snapshot_vars(),
            ),
            &self.selected_guilds,
            &self.generic_commands,
        );

        if self.apply_command_effects(effects) {
            self.scrollback.follow_latest();
        }

        let input = self.input.take_displayed_input();
        self.input.push_history(input);
    }

    fn run_automation(&mut self, line: &str) {
        for cmd in self.automation.process_line(line) {
            self.send_command(cmd);
        }
    }

    fn apply_command_effects(&mut self, effects: Vec<command::CommandEffect>) -> bool {
        let mut sent_command = false;
        for effect in effects {
            match effect {
                command::CommandEffect::Send(command) => {
                    sent_command |= self.send_command(command);
                }
                command::CommandEffect::Automation(action) => {
                    sent_command |= self.apply_automation_actions(vec![action]);
                }
                command::CommandEffect::Output(line) => {
                    self.output.append_lines(vec![line]);
                }
                command::CommandEffect::OpenDialog(kind) => match kind {
                    command::DialogKind::Guilds => self.open_guilds_dialog(),
                    command::DialogKind::GenericCommands => self.open_generic_commands_dialog(),
                    command::DialogKind::Settings => self.open_settings_dialog(),
                },
                command::CommandEffect::ToggleRawLogs => self.toggle_raw_logs(),
                command::CommandEffect::Quit => self.should_quit = true,
            }
        }
        sent_command
    }

    fn apply_automation_actions(&mut self, actions: Vec<Action>) -> bool {
        let mut sent_command = false;
        for cmd in self.automation.apply_actions(actions) {
            sent_command |= self.send_command(cmd);
        }
        sent_command
    }

    fn apply_stats_effects(&mut self, effects: Vec<crate::stats::StatsEffect>) {
        for effect in effects {
            self.stats.apply_effect(effect);
        }
    }

    fn apply_original_line_effects(
        &mut self,
        styled_line: &mut StyledLine,
        effects: triggers::OriginalLineEffects,
    ) {
        for edit in effects.edits {
            edit.apply_to(styled_line);
        }
        if effects.gag {
            styled_line.gag = true;
        }
    }

    fn apply_player_runtime_profile(
        &mut self,
        profile: PlayerRuntimeProfile,
        apply_selected_guilds: bool,
    ) {
        let guild_selection = profile.guild_selection.clone();
        self.player_profile = profile;
        if apply_selected_guilds {
            self.apply_guild_selection(guild_selection);
        } else {
            self.apply_player_profile_to_automation();
        }
        self.apply_player_profile_to_generic_commands();
    }

    fn apply_player_profile_to_automation(&mut self) {
        for (key, value) in &self.player_profile.automation_vars {
            self.automation.set_var(key, value.clone());
        }
        for (key, value) in &self.player_profile.automation_flags {
            self.automation.set_flag(key, *value);
        }
    }

    fn apply_player_profile_to_generic_commands(&mut self) {
        let config = &self.player_profile.generic_commands_config;
        self.generic_commands
            .apply_config(&config.enabled_groups, &config.disabled_commands);
    }

    fn player_profile_from_config(&mut self) -> Option<PlayerRuntimeProfile> {
        let manager = self.config_manager.as_mut()?;
        match manager.player_runtime_profile() {
            Ok(profile) => Some(profile),
            Err(e) => {
                eprintln!("failed to normalize player config: {e}");
                std::process::exit(1);
            }
        }
    }

    fn refresh_player_profile_from_config(&mut self, apply_selected_guilds: bool) {
        if let Some(profile) = self.player_profile_from_config() {
            self.apply_player_runtime_profile(profile, apply_selected_guilds);
        }
    }

    fn apply_user_settings(&mut self, settings: UserSettings) {
        let Some(manager) = self.config_manager.as_mut() else {
            return;
        };
        if let Err(e) = manager.save_user_settings(&settings) {
            eprintln!("failed to save user settings: {e}");
            return;
        }
        self.refresh_player_profile_from_config(false);
    }

    fn apply_guild_selection(&mut self, selection: GuildSelection) {
        self.selected_guilds = selection.build_guilds();
        self.guild_selection = selection;
        self.automation = Automation::new();
        for guild in &self.selected_guilds {
            guild.register_automation(&mut self.automation);
        }
        self.automation.set_flag("in_battle", false);
        self.apply_player_profile_to_automation();
    }

    fn open_guilds_dialog(&mut self) {
        if !self.session.is_logged_in() {
            return;
        }
        if !self.user_config_loaded {
            self.load_user_config();
        }

        let defaults = &self.player_profile.guild_dialog_defaults;
        let primary_kw = if thematic_index_for_keyword(&defaults.primary_background).is_some() {
            defaults.primary_background.as_str()
        } else {
            DEFAULT_GUILD_PRIMARY_KEYWORD
        };

        let entries = catalog::playable_entries_list();
        let selected = entries
            .iter()
            .map(|entry| self.guild_selection.is_selected(entry.key))
            .collect();

        self.guild_dialog = Some(GuildDialog::new(
            entries,
            selected,
            primary_kw,
            defaults.tzarakk_mount.clone(),
            defaults.sabre_weapon.clone(),
            defaults.riftwalker_entity_labels.clone(),
        ));
    }

    fn open_settings_dialog(&mut self) {
        if !self.session.is_logged_in() {
            return;
        }
        if !self.user_config_loaded {
            self.load_user_config();
        }
        let Some(manager) = self.config_manager.as_mut() else {
            return;
        };
        let entries = match manager.user_settings_entries() {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("failed to load settings entries: {e}");
                return;
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
                if dialog.is_browsing_backgrounds() {
                    self.guild_dialog = None;
                } else {
                    dialog.back_to_browse();
                }
            }
            KeyCode::Enter => {
                if dialog.is_browsing_backgrounds() {
                    dialog.open_drill_from_browse_cursor();
                    return;
                }
                let guild_selection = dialog.guild_selection();
                let mount_name = dialog.mount_name();
                let sabre_weapon = dialog.sabre_weapon();
                let riftwalker_entities = dialog.riftwalker_entity_labels();
                self.guild_dialog = None;
                self.apply_guild_selection(guild_selection.clone());
                self.save_selected_guilds_with_auxiliary(
                    guild_selection,
                    mount_name,
                    sabre_weapon,
                    riftwalker_entities,
                );
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

        if let Err(e) = manager.save_generic_commands(&config) {
            eprintln!("failed to save generic commands config: {e}");
        }
        self.refresh_player_profile_from_config(false);
    }

    fn save_selected_guilds_with_auxiliary(
        &mut self,
        guild_selection: GuildSelection,
        mount_name: String,
        sabre_weapon: String,
        riftwalker_entity_labels: [String; 4],
    ) {
        if !self.user_config_loaded {
            self.load_user_config();
        }
        let Some(manager) = self.config_manager.as_mut() else {
            return;
        };

        // Save guilds
        let keys = guild_selection.persisted_keys();
        if let Err(e) =
            manager.save_user_guilds(&keys, guild_selection.primary_background_keyword())
        {
            eprintln!("failed to save user guilds: {e}");
        }

        // Save mount name
        if let Err(e) = manager.save_user_setting(player_profile::TZARAKK_MOUNT_KEY, &mount_name) {
            eprintln!("failed to save tzarakk mount name: {e}");
        }

        if let Err(e) = manager.save_user_setting(player_profile::SABRE_WEAPON_KEY, &sabre_weapon) {
            eprintln!("failed to save sabre_weapon: {e}");
        }

        for (label_key, raw) in player_profile::RIFTWALKER_ENTITY_LABEL_KEYS
            .iter()
            .zip(riftwalker_entity_labels.iter())
        {
            if let Err(err) = manager.save_user_setting(label_key, raw) {
                eprintln!("failed to save {label_key}: {err}");
            }
        }

        self.refresh_player_profile_from_config(true);
    }

    fn send_command(&mut self, command: String) -> bool {
        match self.command_sender.send(command) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("failed to send data: {e}");
                false
            }
        }
    }

    fn toggle_raw_logs(&mut self) {
        let Some(logger) = self.raw_logger.as_mut() else {
            self.output.append_lines(vec![StyledLine::new(
                "Raw logging unavailable: HOME is not set.",
            )]);
            return;
        };

        if logger.is_enabled() {
            logger.disable();
            self.output
                .append_lines(vec![StyledLine::new("Raw logging disabled.")]);
            return;
        }

        match logger.enable(self.session.login_name()) {
            Ok(path) => self.output.append_lines(vec![StyledLine::new(&format!(
                "Raw logging enabled: {}",
                path.display()
            ))]),
            Err(e) => self
                .output
                .append_lines(vec![StyledLine::new(&format!("Raw logging failed: {e}"))]),
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
        let Some(manager) = self.config_manager.as_mut() else {
            return;
        };
        if let Err(e) = manager.load_user(player_name) {
            eprintln!("failed to load user config for {player_name}: {e}");
            return;
        }

        self.refresh_player_profile_from_config(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn test_app() -> (BatApp, mpsc::Receiver<String>) {
        let (_event_sender, event_receiver) = mpsc::channel();
        let (command_sender, command_receiver) = mpsc::channel();
        let app = BatApp {
            output: OutputBuffer::new(),
            input: InputState::new(),
            session: SessionState::new(),
            stats: Default::default(),
            event_receiver,
            command_sender,
            telnet_buffer: TelnetBuffer::new(),
            selected_guilds: Vec::new(),
            guild_selection: GuildSelection::default(),
            should_quit: false,
            automation: Automation::new(),
            config_manager: None,
            user_config_loaded: true,
            player_profile: PlayerRuntimeProfile::default(),
            guild_dialog: None,
            generic_commands: GenericCommands::default(),
            generic_commands_dialog: None,
            settings_dialog: None,
            player_logger: None,
            raw_logger: None,
            scrollback: Scrollback::new(),
        };
        (app, command_receiver)
    }

    #[test]
    fn command_effect_send_writes_to_command_sender() {
        let (mut app, command_receiver) = test_app();

        let followed = app.apply_command_effects(vec![command::CommandEffect::Send("look".into())]);

        assert!(followed);
        assert_eq!(command_receiver.try_recv().as_deref(), Ok("look"));
    }

    #[test]
    fn command_effect_automation_expands_vars_and_sends() {
        let (mut app, command_receiver) = test_app();
        app.automation.set_var("rig", "satchel".to_string());

        let followed = app.apply_command_effects(vec![command::CommandEffect::Automation(
            Action::Send("put all essence in {rig}".to_string()),
        )]);

        assert!(followed);
        assert_eq!(
            command_receiver.try_recv().as_deref(),
            Ok("put all essence in satchel")
        );
    }

    #[test]
    fn command_effect_output_appends_to_output_buffer() {
        let (mut app, _command_receiver) = test_app();

        let followed = app.apply_command_effects(vec![command::CommandEffect::Output(
            StyledLine::new("hello"),
        )]);

        assert!(!followed);
        assert_eq!(app.output.plain_lines(), vec!["hello"]);
    }

    #[test]
    fn command_effect_open_dialog_opens_selected_dialog() {
        let (mut app, _command_receiver) = test_app();
        app.session.set_login_name("tester".to_string());
        app.session
            .update_login_state("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >");

        app.apply_command_effects(vec![command::CommandEffect::OpenDialog(
            command::DialogKind::Guilds,
        )]);

        assert!(app.guild_dialog.is_some());
    }

    #[test]
    fn command_effect_toggle_raw_logs_reports_unavailable_logger() {
        let (mut app, _command_receiver) = test_app();

        let followed = app.apply_command_effects(vec![command::CommandEffect::ToggleRawLogs]);

        assert!(!followed);
        assert_eq!(
            app.output.plain_lines(),
            vec!["Raw logging unavailable: HOME is not set."]
        );
    }

    #[test]
    fn command_effect_quit_sets_app_quit_flag() {
        let (mut app, _command_receiver) = test_app();

        app.apply_command_effects(vec![command::CommandEffect::Quit]);

        assert!(app.should_quit());
    }
}
