mod dialogs;
mod input_state;
mod output_buffer;
mod player_logger;
mod raw_logger;
mod scrollback;
mod session_lifecycle;
mod session_state;
mod telnet_buffer;
mod util;

#[cfg(test)]
mod fake_connection_coordinator;

use crate::ansi::StyledLine;
use crate::automation::{Action, Automation};
use crate::combat_awareness::{CombatAwareness, CombatAwarenessEffect};
use crate::config::{ConfigManager, GenericCommandsConfig, UserSettings};
use crate::generic_commands::GenericCommands;
use crate::guilds::{
    Guild,
    catalog::{self, GuildSelection},
    grouping::{DEFAULT_GUILD_PRIMARY_KEYWORD, thematic_index_for_keyword},
};
use crate::player_profile::{self, PlayerRuntimeProfile};
use crate::secondary_status::SecondaryStatus;
use crate::stats::Stats;
use crate::ui::{Renderer, ViewModel};
use crate::{command, triggers};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use dialogs::{GenericCommandsDialog, GuildDialog, SettingsDialog, apply_guild_dialog_keystroke};
use input_state::InputState;
use libmudtelnet::events::TelnetEvents;
use log::{error, warn};
use player_logger::PlayerLogger;
use ratatui::Frame;
use ratatui::text::Line;
use raw_logger::RawLogger;
use std::sync::mpsc::{Receiver, Sender};
use telnet_buffer::TelnetBuffer;
use util::show_clock;

use output_buffer::OutputBuffer;
use scrollback::Scrollback;
use session_lifecycle::{
    FreshSessionPlan, FreshSessionReset, OutputDisposition, ReconnectAttemptResult,
    SessionLifecycle, complete_connect, prepare_connect,
};
use session_state::{LoginState, SessionState};

pub type ConnectionId = u64;

pub const INITIAL_CONNECTION_ID: ConnectionId = 0;
const MOUSE_WHEEL_SCROLL_LINES: usize = 3;

pub enum AppEvent {
    RawInput {
        connection_id: ConnectionId,
        bytes: Vec<u8>,
    },
    Telnet {
        connection_id: ConnectionId,
        event: TelnetEvents,
    },
}

pub struct ConnectionChannels {
    pub event_receiver: Receiver<AppEvent>,
    pub command_sender: Sender<String>,
}

pub enum ReconnectResult {
    Connected(ConnectionChannels),
    Failed(String),
}

pub trait ConnectionCoordinator {
    fn reconnect(&mut self, connection_id: ConnectionId) -> ReconnectResult;
}

pub struct BatApp {
    output: OutputBuffer,
    input: InputState,
    session: SessionState,
    stats: Stats,
    secondary_status: SecondaryStatus,
    combat_awareness: CombatAwareness,
    event_receiver: Receiver<AppEvent>,
    command_sender: Sender<String>,
    connection_coordinator: Box<dyn ConnectionCoordinator>,
    session_lifecycle: SessionLifecycle,
    telnet_buffer: TelnetBuffer,
    selected_guilds: Vec<Box<dyn Guild>>,
    guild_selection: GuildSelection,
    should_quit: bool,
    pending_terminal_clear: bool,
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
    pub fn new(
        event_receiver: Receiver<AppEvent>,
        command_sender: Sender<String>,
        connection_coordinator: Box<dyn ConnectionCoordinator>,
    ) -> Self {
        let config_manager = match ConfigManager::new() {
            Ok(mut manager) => {
                if let Err(e) = manager.init_base() {
                    error!("failed to initialize base config: {e}");
                }
                Some(manager)
            }
            Err(e) => {
                error!("failed to initialize config manager: {e}");
                None
            }
        };
        let mut app = BatApp {
            output: OutputBuffer::new(),
            input: InputState::new(),
            session: SessionState::new(),
            stats: Default::default(),
            secondary_status: SecondaryStatus::default(),
            combat_awareness: CombatAwareness::default(),
            event_receiver,
            command_sender,
            connection_coordinator,
            session_lifecycle: SessionLifecycle::new(),
            telnet_buffer: TelnetBuffer::new(),
            selected_guilds: Vec::new(),
            guild_selection: GuildSelection::default(),
            should_quit: false,
            pending_terminal_clear: false,
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
        let mut automation_lines = Vec::new();

        for line in lines {
            let mut styled_line = StyledLine::new(&line);
            let was_logged_in = self.session.is_logged_in();
            if self.session.update_login_state(&styled_line.plain_line) {
                self.input.clear_all();
            }
            if !was_logged_in && self.session.is_logged_in() {
                if let Some(login_name) = self.session.login_name()
                    && let Some(disposition) =
                        self.session_lifecycle.on_post_connect_login(login_name)
                    && disposition == OutputDisposition::ClearOutput
                {
                    self.output.clear();
                    self.scrollback.follow_latest();
                }
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
                warn!("failed to log line: {e}");
            }

            if self.session.is_logged_in() {
                let ca_result = self
                    .combat_awareness
                    .handle_incoming_line(&styled_line.plain_line);
                self.apply_combat_awareness_effects(ca_result.effects);
                if ca_result.gag {
                    styled_line.gag = true;
                    output_lines.push(styled_line);
                    continue;
                }
                let facts = triggers::TriggerFacts::new(
                    self.automation.snapshot_flags(),
                    self.automation.snapshot_vars(),
                    self.player_profile.settings.rig_for_triggers(),
                    self.session.login_name(),
                );
                let result =
                    triggers::process(&facts, &self.selected_guilds, &styled_line.plain_line);
                self.apply_stats_effects(result.stats);
                self.apply_secondary_status_effects(result.secondary_status);
                self.apply_automation_actions(result.actions);
                let mut new_lines = result.lines;
                self.apply_original_line_effects(&mut styled_line, result.original);
                automation_lines.extend(new_lines.iter().map(|line| line.plain_line.clone()));
                automation_lines.push(styled_line.plain_line.clone());
                new_lines.push(styled_line);
                output_lines.extend(new_lines);
            } else {
                output_lines.push(styled_line);
            }
        }

        if self.session.is_logged_in() {
            for line in &automation_lines {
                self.run_automation(line);
            }
        }

        self.output.append_lines(output_lines);
    }

    pub fn read_input(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                AppEvent::RawInput {
                    connection_id,
                    bytes,
                } => {
                    if self.session_lifecycle.is_stale(connection_id) {
                        continue;
                    }
                    if let Some(logger) = self.raw_logger.as_mut()
                        && let Err(e) = logger.write_bytes(&bytes)
                    {
                        warn!("failed to write raw log: {e}");
                    }
                }
                AppEvent::Telnet {
                    connection_id,
                    event,
                } => {
                    if self.session_lifecycle.is_stale(connection_id) {
                        continue;
                    }
                    let lines = self.telnet_buffer.handle_event(&event);
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

    pub fn take_pending_terminal_clear(&mut self) -> bool {
        let pending = self.pending_terminal_clear;
        self.pending_terminal_clear = false;
        pending
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
            KeyCode::Delete => self.input.delete(),
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

    pub fn handle_mouse_event(&mut self, event: MouseEvent) {
        if self.guild_dialog.is_some()
            || self.generic_commands_dialog.is_some()
            || self.settings_dialog.is_some()
        {
            return;
        }

        match event.kind {
            MouseEventKind::ScrollUp => self.scrollback.scroll_up(MOUSE_WHEEL_SCROLL_LINES),
            MouseEventKind::ScrollDown => self.scrollback.scroll_down(MOUSE_WHEEL_SCROLL_LINES),
            _ => {}
        }
    }

    pub fn handle_paste_event(&mut self, text: String) {
        if self.guild_dialog.is_some()
            || self.generic_commands_dialog.is_some()
            || self.settings_dialog.is_some()
        {
            return;
        }

        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let mut lines = normalized.split('\n').peekable();
        while let Some(line) = lines.next() {
            self.input.insert_str(line);
            if lines.peek().is_some() {
                self.submit_input();
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        let show_stats = self.session.is_logged_in();

        let combat_status_lines = if show_stats {
            crate::ui::render_combat_status_lines(
                self.combat_awareness.is_active(),
                self.combat_awareness.snapshot(),
                frame.area().width,
            )
        } else {
            Vec::new()
        };
        let secondary_status_lines = if show_stats {
            self.secondary_status
                .render_lines(frame.area().width, &self.guild_selection)
        } else {
            Vec::new()
        };

        let reserved_rows =
            2 + combat_status_lines.len() as u16 + secondary_status_lines.len() as u16;
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
            combat_status_lines,
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
                if !input.is_empty() && self.session.login_state() == LoginState::Name {
                    self.session.set_login_name(input.clone());
                }
                if !input.is_empty() && self.session.login_state() == LoginState::Choice {
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

        if self.apply_user_command_effects(effects) {
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
        self.apply_command_effects_inner(effects, false)
    }

    fn apply_user_command_effects(&mut self, effects: Vec<command::CommandEffect>) -> bool {
        self.apply_command_effects_inner(effects, true)
    }

    fn apply_command_effects_inner(
        &mut self,
        effects: Vec<command::CommandEffect>,
        count_user_game_sends: bool,
    ) -> bool {
        let mut sent_command = false;
        for effect in effects {
            match effect {
                command::CommandEffect::Send(command) => {
                    let count_for_probe = count_user_game_sends && !command.trim().is_empty();
                    if self.send_command(command) {
                        sent_command = true;
                        if count_for_probe
                            && let Some(effect) = self.combat_awareness.observe_user_game_command()
                        {
                            sent_command |= self.apply_combat_awareness_effects(vec![effect]);
                        }
                    }
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
                command::CommandEffect::Reconnect => self.start_reconnect(),
                command::CommandEffect::ToggleRawLogs => self.toggle_raw_logs(),
                command::CommandEffect::Quit => self.should_quit = true,
                command::CommandEffect::Redraw => self.pending_terminal_clear = true,
            }
        }
        sent_command
    }

    fn start_reconnect(&mut self) {
        let pre_connect_login_name = self.session.login_name().map(str::to_string);
        let plan = match prepare_connect(&mut self.session_lifecycle, pre_connect_login_name) {
            Ok(plan) => plan,
            Err(()) => {
                self.output
                    .append_lines(vec![StyledLine::new("Reconnect already in progress.")]);
                return;
            }
        };

        self.apply_fresh_session_plan(plan);
        self.output
            .append_lines(vec![StyledLine::new("Reconnect started.")]);

        match complete_connect(
            &mut self.session_lifecycle,
            self.connection_coordinator.as_mut(),
            plan,
        ) {
            ReconnectAttemptResult::Connected(channels) => self.install_connection(channels),
            ReconnectAttemptResult::Failed(error) => {
                self.output
                    .append_lines(vec![StyledLine::new(&format!("Reconnect failed: {error}"))]);
            }
        }
    }

    fn apply_fresh_session_plan(&mut self, plan: FreshSessionPlan) {
        for reset in plan.resets() {
            match reset {
                FreshSessionReset::Session => self.session.reset(),
                FreshSessionReset::Stats => self.stats = Stats::default(),
                FreshSessionReset::SecondaryStatus => {
                    self.secondary_status = SecondaryStatus::default();
                }
                FreshSessionReset::CombatAwareness => {
                    self.combat_awareness = CombatAwareness::default();
                }
                FreshSessionReset::TelnetBuffer => self.telnet_buffer = TelnetBuffer::new(),
                FreshSessionReset::GuildSelection => {
                    self.selected_guilds.clear();
                    self.guild_selection = GuildSelection::default();
                }
                FreshSessionReset::Automation => {
                    self.automation = Automation::new();
                    self.automation.set_flag("in_battle", false);
                }
                FreshSessionReset::UserConfigLoaded => self.user_config_loaded = false,
                FreshSessionReset::PlayerProfile => {
                    self.player_profile = PlayerRuntimeProfile::default();
                }
                FreshSessionReset::GenericCommands => {
                    self.generic_commands = GenericCommands::default();
                }
                FreshSessionReset::Dialogs => {
                    self.guild_dialog = None;
                    self.generic_commands_dialog = None;
                    self.settings_dialog = None;
                }
            }
        }
    }

    fn install_connection(&mut self, channels: ConnectionChannels) {
        self.event_receiver = channels.event_receiver;
        self.command_sender = channels.command_sender;
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

    fn apply_secondary_status_effects(
        &mut self,
        effects: Vec<crate::secondary_status::SecondaryStatusEffect>,
    ) {
        for effect in effects {
            self.secondary_status.apply_effect(effect);
        }
    }

    fn apply_combat_awareness_effects(&mut self, effects: Vec<CombatAwarenessEffect>) -> bool {
        let mut sent_command = false;
        for effect in effects {
            match effect {
                CombatAwarenessEffect::RoundStarted => {
                    self.stats
                        .apply_effect(crate::stats::StatsEffect::StartCombatRound);
                    self.automation.set_flag("in_battle", true);
                }
                CombatAwarenessEffect::CombatEnded => {
                    self.stats
                        .apply_effect(crate::stats::StatsEffect::EndCombat);
                    self.automation.set_flag("in_battle", false);
                }
                CombatAwarenessEffect::SendProbe => {
                    sent_command |=
                        self.send_command(crate::combat_awareness::PROBE_COMMAND.to_string());
                }
                CombatAwarenessEffect::SendShortScore => {
                    sent_command |= self.send_command("@sc".to_string());
                }
            }
        }
        sent_command
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
                error!("failed to normalize player config: {e}");
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
            warn!("failed to save user settings: {e}");
            return;
        }
        self.refresh_player_profile_from_config(false);
    }

    fn apply_guild_selection(&mut self, selection: GuildSelection) {
        self.selected_guilds = selection.build_guilds();
        self.guild_selection = selection.clone();
        self.secondary_status.sync_guild_selection(&selection);
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
                warn!("failed to load settings entries: {e}");
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
            KeyCode::Delete => dialog.backspace(),
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
            warn!("failed to save generic commands config: {e}");
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
            warn!("failed to save user guilds: {e}");
        }

        // Save mount name
        if let Err(e) = manager.save_user_setting(player_profile::TZARAKK_MOUNT_KEY, &mount_name) {
            warn!("failed to save tzarakk mount name: {e}");
        }

        if let Err(e) = manager.save_user_setting(player_profile::SABRE_WEAPON_KEY, &sabre_weapon) {
            warn!("failed to save sabre_weapon: {e}");
        }

        for (label_key, raw) in player_profile::RIFTWALKER_ENTITY_LABEL_KEYS
            .iter()
            .zip(riftwalker_entity_labels.iter())
        {
            if let Err(err) = manager.save_user_setting(label_key, raw) {
                warn!("failed to save {label_key}: {err}");
            }
        }

        self.refresh_player_profile_from_config(true);
    }

    fn send_command(&mut self, command: String) -> bool {
        match self.command_sender.send(command) {
            Ok(()) => true,
            Err(e) => {
                error!("failed to send data: {e}");
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
            warn!("logged in without a known player name; skipping user config");
            return;
        };
        let Some(manager) = self.config_manager.as_mut() else {
            return;
        };
        if let Err(e) = manager.load_user(player_name) {
            warn!("failed to load user config for {player_name}: {e}");
            return;
        }

        self.refresh_player_profile_from_config(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::fake_connection_coordinator::{FakeConnectionCoordinator, connection_channels};
    use crate::automation::Waiter;
    use crate::guilds::catalog::GuildKey;
    use regex::Regex;
    use std::sync::mpsc;

    fn test_app() -> (BatApp, mpsc::Receiver<String>) {
        let (app, command_receiver, _event_sender) =
            test_app_with_coordinator(Box::new(FakeConnectionCoordinator::default()));
        (app, command_receiver)
    }

    fn log_in(app: &mut BatApp) {
        app.session.set_login_name("tester".to_string());
        app.session
            .update_login_state("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >");
    }

    fn drain_commands(command_receiver: &mpsc::Receiver<String>) -> Vec<String> {
        let mut commands = Vec::new();
        while let Ok(command) = command_receiver.try_recv() {
            commands.push(command);
        }
        commands
    }

    fn test_app_with_coordinator(
        connection_coordinator: Box<dyn ConnectionCoordinator>,
    ) -> (BatApp, mpsc::Receiver<String>, mpsc::Sender<AppEvent>) {
        let (channels, command_receiver, event_sender) = connection_channels();
        let app = BatApp {
            output: OutputBuffer::new(),
            input: InputState::new(),
            session: SessionState::new(),
            stats: Default::default(),
            secondary_status: SecondaryStatus::default(),
            combat_awareness: CombatAwareness::default(),
            event_receiver: channels.event_receiver,
            command_sender: channels.command_sender,
            connection_coordinator,
            session_lifecycle: SessionLifecycle::new(),
            telnet_buffer: TelnetBuffer::new(),
            selected_guilds: Vec::new(),
            guild_selection: GuildSelection::default(),
            should_quit: false,
            pending_terminal_clear: false,
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
        (app, command_receiver, event_sender)
    }

    #[test]
    fn combat_round_starts_correlated_probe_before_short_score() {
        let (mut app, command_receiver) = test_app();
        log_in(&mut app);

        app.process_input_lines(vec!["*** Round 1 ***".to_string()]);

        assert_eq!(drain_commands(&command_receiver), vec!["@sc", "#scan all"]);
        assert_eq!(
            app.stats.start_combat_round_invocations(),
            1,
            "round header lifecycle fan-out must apply StartCombatRound once per line"
        );
        assert!(app.combat_awareness.is_active());
        assert!(app.automation.flag_is_set("in_battle"));
    }

    #[test]
    fn correlated_scan_rows_are_gagged_and_kept_out_of_automation() {
        let (mut app, command_receiver) = test_app();
        log_in(&mut app);
        app.automation.add_waiter(Waiter {
            pattern: Regex::new("Guard").unwrap(),
            actions: vec![Action::Send("waiter fired".to_string())],
            consume: true,
        });
        app.process_input_lines(vec!["*** Round 1 ***".to_string()]);
        drain_commands(&command_receiver);

        app.process_input_lines(vec![
            "scan all".to_string(),
            "Guard is noticeably hurt (50%).".to_string(),
            "The rain falls.".to_string(),
        ]);

        assert_eq!(drain_commands(&command_receiver), Vec::<String>::new());
        assert_eq!(
            app.output.plain_lines(),
            vec!["*** Round 1 ***", "The rain falls."]
        );
        assert_eq!(app.combat_awareness.snapshot().len(), 1);
    }

    #[test]
    fn nergal_resource_status_line_is_gagged_and_updates_secondary_status() {
        let (mut app, _command_receiver) = test_app();
        log_in(&mut app);
        app.apply_guild_selection(GuildSelection::from_playable_keys(
            [GuildKey::Nergal],
            Some("evil_religious"),
        ));

        app.process_input_lines(vec![
            "::..:. [Vitae: 22/1000  Potentia: 752/1000, Evolution points: 0]".to_string(),
        ]);

        let rendered_status: String = app
            .secondary_status
            .render_nergal_status_lines(200)
            .into_iter()
            .flat_map(|line| line.spans.into_iter())
            .map(|span| span.content.to_string())
            .collect();
        assert!(app.output.plain_lines().is_empty());
        assert_eq!(
            rendered_status,
            "Vitae: 22/1000 Potentia: 752/1000, Evolution points: 0"
        );
    }

    #[test]
    fn nergal_resource_status_line_is_not_gagged_without_selected_nergal_guild() {
        let (mut app, _command_receiver) = test_app();
        log_in(&mut app);
        let line = "::..:. [Vitae: 22/1000  Potentia: 752/1000, Evolution points: 0]";

        app.process_input_lines(vec![line.to_string()]);

        let rendered_status: String = app
            .secondary_status
            .render_nergal_status_lines(200)
            .into_iter()
            .flat_map(|line| line.spans.into_iter())
            .map(|span| span.content.to_string())
            .collect();
        assert_eq!(app.output.plain_lines(), vec![line]);
        assert!(rendered_status.is_empty());
        assert!(!app.secondary_status.has_nergal_resource_status());
    }

    #[test]
    fn prompt_and_short_score_updates_preserve_secondary_status() {
        let (mut app, _command_receiver) = test_app();
        log_in(&mut app);
        app.apply_guild_selection(GuildSelection::from_playable_keys(
            [GuildKey::Animist],
            Some("nature"),
        ));

        app.process_input_lines(vec![
            "Your soul companion: exc (88%) guarding you".to_string(),
        ]);
        assert!(app.secondary_status.has_soul_companion_status());

        app.process_input_lines(vec!["Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >".to_string()]);
        assert!(app.secondary_status.has_soul_companion_status());

        app.process_input_lines(vec![
            "H:571/802 [+20] S:635/635 [] E:311/311 [] $:2786 [] exp:21657 []".to_string(),
        ]);
        assert!(app.secondary_status.has_soul_companion_status());
    }

    #[test]
    fn deselecting_animist_clears_soul_companion_status() {
        let (mut app, _command_receiver) = test_app();
        log_in(&mut app);
        app.apply_guild_selection(GuildSelection::from_playable_keys(
            [GuildKey::Animist],
            Some("nature"),
        ));
        app.process_input_lines(vec![
            "Your soul companion: exc (88%) guarding you".to_string(),
        ]);
        assert!(app.secondary_status.has_soul_companion_status());

        app.apply_guild_selection(GuildSelection::from_playable_keys(
            [GuildKey::Nergal],
            Some("evil_religious"),
        ));

        assert!(!app.secondary_status.has_soul_companion_status());
        assert!(
            app.secondary_status
                .render_lines(200, &app.guild_selection)
                .is_empty()
        );
    }

    #[test]
    fn deselecting_nergal_clears_resource_status_and_minions() {
        use crate::secondary_status::NergalMinion;

        let (mut app, _command_receiver) = test_app();
        log_in(&mut app);
        app.apply_guild_selection(GuildSelection::from_playable_keys(
            [GuildKey::Nergal],
            Some("evil_religious"),
        ));

        app.process_input_lines(vec![
            "::..:. [Vitae: 22/1000  Potentia: 752/1000, Evolution points: 0]".to_string(),
        ]);
        app.secondary_status.upsert_nergal_minion(NergalMinion {
            name: "Weeping pixie".into(),
            hp: 364,
            max_hp: 425,
            sp: 447,
            max_sp: 467,
            ep: 138,
            max_ep: 155,
        });
        assert!(app.secondary_status.has_nergal_resource_status());
        assert!(app.secondary_status.has_nergal_minions());

        app.apply_guild_selection(GuildSelection::from_playable_keys(
            [GuildKey::Animist],
            Some("nature"),
        ));

        assert!(!app.secondary_status.has_nergal_resource_status());
        assert!(!app.secondary_status.has_nergal_minions());
        assert!(
            app.secondary_status
                .render_nergal_status_lines(200)
                .is_empty()
        );
        assert!(
            app.secondary_status
                .render_nergal_minion_lines(200)
                .is_empty()
        );
    }

    #[test]
    fn nergal_unsummon_clears_minion_status_with_selected_nergal_guild() {
        use crate::secondary_status::NergalMinion;

        let (mut app, _command_receiver) = test_app();
        log_in(&mut app);
        app.apply_guild_selection(GuildSelection::from_playable_keys(
            [GuildKey::Nergal],
            Some("evil_religious"),
        ));
        app.secondary_status.upsert_nergal_minion(NergalMinion {
            name: "Weeping pixie".into(),
            hp: 364,
            max_hp: 425,
            sp: 447,
            max_sp: 467,
            ep: 138,
            max_ep: 155,
        });
        assert!(
            !app.secondary_status
                .render_nergal_minion_lines(200)
                .is_empty()
        );

        app.process_input_lines(vec![
            "More thoughts infiltrate your mind. As you are evaluating your minions, one of them seems sub optimal for the servitude of the lord Nergal. You 'release' the host from the parasites influence. The host jerks violently couple of times as if regaining its free will but without the parasite the host is too weak to survive and collapses.".to_string(),
        ]);

        assert!(!app.secondary_status.has_nergal_minions());
        assert!(
            app.secondary_status
                .render_nergal_minion_lines(200)
                .is_empty()
        );
    }

    #[test]
    fn scan_capture_completes_when_next_round_header_arrives() {
        let (mut app, command_receiver) = test_app();
        log_in(&mut app);

        app.process_input_lines(vec![
            "********************** Round 1 **********************".to_string(),
            "Guard misses.".to_string(),
            "scan all".to_string(),
            "Guard is slightly hurt (70%).".to_string(),
            "********************** Round 2 **********************".to_string(),
        ]);
        assert_eq!(
            drain_commands(&command_receiver),
            vec!["@sc", "#scan all", "@sc", "#scan all"]
        );

        let rendered_scan: String = crate::ui::render_combat_status_lines(
            app.combat_awareness.is_active(),
            app.combat_awareness.snapshot(),
            120,
        )
        .into_iter()
        .flat_map(|line| line.spans.into_iter())
        .map(|span| span.content.to_string())
        .collect();
        assert_eq!(
            rendered_scan, "Guard is slightly hurt (70%).",
            "scan row should remain visible in the combat panel after the next round header"
        );
        assert_eq!(
            app.output.plain_lines(),
            vec![
                "********************** Round 1 **********************",
                "Guard misses.",
                "********************** Round 2 **********************"
            ]
        );
    }

    #[test]
    fn gagged_prompt_before_scan_echo_does_not_cancel_probe() {
        let (mut app, command_receiver) = test_app();
        log_in(&mut app);

        app.process_input_lines(vec![
            "********************** Round 1 **********************".to_string(),
            "Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >".to_string(),
            "scan all".to_string(),
            "Guard is slightly hurt (70%).".to_string(),
            "********************** Round 2 **********************".to_string(),
        ]);
        assert_eq!(
            drain_commands(&command_receiver),
            vec!["@sc", "#scan all", "@sc", "#scan all"]
        );

        let rendered_scan: String = crate::ui::render_combat_status_lines(
            app.combat_awareness.is_active(),
            app.combat_awareness.snapshot(),
            120,
        )
        .into_iter()
        .flat_map(|line| line.spans.into_iter())
        .map(|span| span.content.to_string())
        .collect();
        assert_eq!(rendered_scan, "Guard is slightly hurt (70%).");
        assert_eq!(
            app.output.plain_lines(),
            vec![
                "********************** Round 1 **********************",
                "********************** Round 2 **********************"
            ],
            "prompt, scan echo, and captured scan row should stay gagged"
        );
    }

    #[test]
    fn user_game_commands_probe_every_other_send_after_user_command() {
        let (mut app, command_receiver) = test_app();
        log_in(&mut app);
        app.process_input_lines(vec!["*** Round 1 ***".to_string()]);
        app.process_input_lines(vec![
            "scan all".to_string(),
            "Guard is noticeably hurt (50%).".to_string(),
            "The rain falls.".to_string(),
        ]);
        drain_commands(&command_receiver);

        app.input.insert_str("look");
        app.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        app.input.insert_str("north");
        app.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(
            drain_commands(&command_receiver),
            vec!["look", "north", "#scan all"]
        );
    }

    #[test]
    fn second_combat_round_clears_accumulated_short_score_diffs() {
        let (mut app, command_receiver) = test_app();
        log_in(&mut app);
        app.process_input_lines(vec!["*** Round 1 ***".to_string()]);
        app.process_input_lines(vec![
            "H:1/2 [+10] S:3/4 [-5] E:5/6 [] $:7 [-5] exp:8 [+9]".to_string(),
        ]);
        drain_commands(&command_receiver);

        let line_with_diffs: String = app
            .stats
            .render_inline()
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(
            line_with_diffs.contains("+10"),
            "first round short score should accumulate diffs; got {line_with_diffs:?}"
        );

        app.process_input_lines(vec!["*** Round 2 ***".to_string()]);
        drain_commands(&command_receiver);

        let line_after_round: String = app
            .stats
            .render_inline()
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(
            !line_after_round.contains("+10")
                && !line_after_round.contains("-5")
                && !line_after_round.contains("+9"),
            "new combat round should clear previous diffs via CA fan-out; got {line_after_round:?}"
        );
    }

    #[test]
    fn global_not_in_combat_clears_scan_state_and_stops_diff_accumulation() {
        let (mut app, command_receiver) = test_app();
        log_in(&mut app);
        app.automation.set_flag("is_lich", true);
        app.process_input_lines(vec!["*** Round 1 ***".to_string()]);
        app.process_input_lines(vec![
            "H:1/2 [+10] S:3/4 [] E:5/6 [] $:7 [] exp:8 []".to_string(),
            crate::combat_awareness::NOT_IN_COMBAT_LINE.to_string(),
            "H:1/2 [] S:3/4 [] E:5/6 [] $:7 [] exp:8 []".to_string(),
        ]);
        drain_commands(&command_receiver);

        let rendered: String = app
            .stats
            .render_inline()
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(!app.combat_awareness.is_active());
        assert!(!app.automation.flag_is_set("in_battle"));
        assert!(app.combat_awareness.snapshot().is_empty());
        assert_eq!(
            app.stats.end_combat_invocations(),
            1,
            "gagged probe combat-end must still fan out EndCombat exactly once"
        );
        assert!(
            !app.output
                .plain_lines()
                .contains(&crate::combat_awareness::NOT_IN_COMBAT_LINE),
            "probe-returned combat-end should stay gagged while probe is in flight"
        );
        assert!(
            drain_commands(&command_receiver).is_empty(),
            "gagged combat-end must not reach trigger pipeline"
        );
        assert!(
            !rendered.contains("+10"),
            "post-combat short score should replace accumulated diffs; got {rendered:?}"
        );
    }

    #[test]
    fn capturing_probe_combat_end_gags_line_and_clears_snapshot() {
        let (mut app, command_receiver) = test_app();
        log_in(&mut app);
        app.automation.set_flag("is_lich", true);
        app.process_input_lines(vec!["*** Round 1 ***".to_string()]);
        drain_commands(&command_receiver);

        app.process_input_lines(vec![
            "scan all".to_string(),
            "Guard is noticeably hurt (50%).".to_string(),
            crate::combat_awareness::NOT_IN_COMBAT_LINE.to_string(),
        ]);
        drain_commands(&command_receiver);

        assert!(!app.combat_awareness.is_active());
        assert!(!app.automation.flag_is_set("in_battle"));
        assert!(app.combat_awareness.snapshot().is_empty());
        assert_eq!(
            app.stats.end_combat_invocations(),
            1,
            "capturing-probe combat-end must fan out EndCombat exactly once"
        );
        assert!(
            !app.output
                .plain_lines()
                .contains(&crate::combat_awareness::NOT_IN_COMBAT_LINE)
        );
        assert!(drain_commands(&command_receiver).is_empty());
    }

    #[test]
    fn combat_end_fan_out_runs_once_per_line() {
        let (mut app, command_receiver) = test_app();
        log_in(&mut app);
        app.automation.set_flag("is_lich", true);
        app.process_input_lines(vec!["*** Round 1 ***".to_string()]);
        app.process_input_lines(vec![
            "scan all".to_string(),
            "Guard is noticeably hurt (50%).".to_string(),
            "done".to_string(),
        ]);
        drain_commands(&command_receiver);
        assert_eq!(app.combat_awareness.snapshot().len(), 1);
        app.process_input_lines(vec![
            "H:1/2 [+10] S:3/4 [] E:5/6 [] $:7 [] exp:8 []".to_string(),
        ]);
        drain_commands(&command_receiver);

        app.process_input_lines(vec![
            crate::combat_awareness::NOT_IN_COMBAT_LINE.to_string(),
            "H:1/2 [] S:3/4 [] E:5/6 [] $:7 [] exp:8 []".to_string(),
        ]);

        assert_eq!(
            drain_commands(&command_receiver),
            vec!["@lich drain"],
            "organic combat-end reaches triggers; lifecycle fans out once via CA"
        );
        assert!(!app.combat_awareness.is_active());
        assert!(!app.automation.flag_is_set("in_battle"));
        assert!(app.combat_awareness.snapshot().is_empty());
        assert_eq!(
            app.stats.end_combat_invocations(),
            1,
            "organic combat-end must fan out EndCombat exactly once per line"
        );

        let rendered: String = app
            .stats
            .render_inline()
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(
            !rendered.contains("+10"),
            "post-combat short score must not retain in-round diffs after single EndCombat fan-out; got {rendered:?}"
        );
    }

    #[test]
    fn second_combat_end_line_increments_end_combat_fan_out_once_per_line() {
        let (mut app, command_receiver) = test_app();
        log_in(&mut app);
        app.process_input_lines(vec!["*** Round 1 ***".to_string()]);
        drain_commands(&command_receiver);

        app.process_input_lines(vec![
            crate::combat_awareness::NOT_IN_COMBAT_LINE.to_string(),
            crate::combat_awareness::NOT_IN_COMBAT_LINE.to_string(),
        ]);
        drain_commands(&command_receiver);

        assert_eq!(
            app.stats.end_combat_invocations(),
            2,
            "each combat-end line must fan out EndCombat once, not reuse a prior path"
        );
    }

    #[test]
    fn monk_not_in_combat_resets_skill_vars_through_trigger_pipeline() {
        const ARMOUR_VAR: &str = "monk_current_armour_skill";
        const DISRUPT_VAR: &str = "monk_current_disrupt_skill";
        const AREA_VAR: &str = "monk_current_area_skill";
        const AVOID_VAR: &str = "monk_current_avoid_skill";

        let (mut app, command_receiver) = test_app();
        log_in(&mut app);
        app.apply_guild_selection(GuildSelection::from_playable_keys(
            [GuildKey::Monk],
            Some("evil_religious"),
        ));
        app.automation
            .set_var(ARMOUR_VAR, "earthquake kick".to_string());
        app.automation
            .set_var(DISRUPT_VAR, "geyser force kick".to_string());
        app.automation
            .set_var(AREA_VAR, "winged horse kick".to_string());
        app.automation
            .set_var(AVOID_VAR, "elder cobra kick".to_string());

        app.process_input_lines(vec!["*** Round 1 ***".to_string()]);
        app.process_input_lines(vec![
            "scan all".to_string(),
            "Guard is noticeably hurt (50%).".to_string(),
            "done".to_string(),
        ]);
        drain_commands(&command_receiver);

        app.process_input_lines(vec![
            crate::combat_awareness::NOT_IN_COMBAT_LINE.to_string(),
        ]);

        assert_eq!(
            app.automation.get_var(ARMOUR_VAR),
            Some(&"falling boulder strike".to_string())
        );
        assert_eq!(
            app.automation.get_var(DISRUPT_VAR),
            Some(&"wave crest strike".to_string())
        );
        assert_eq!(
            app.automation.get_var(AREA_VAR),
            Some(&"hydra fang strike".to_string())
        );
        assert_eq!(
            app.automation.get_var(AVOID_VAR),
            Some(&"falcon talon strike".to_string())
        );
    }

    #[test]
    fn command_effect_send_writes_to_command_sender() {
        let (mut app, command_receiver) = test_app();

        let followed = app.apply_command_effects(vec![command::CommandEffect::Send("look".into())]);

        assert!(followed);
        assert_eq!(command_receiver.try_recv().as_deref(), Ok("look"));
    }

    #[test]
    fn enter_on_empty_logged_in_input_sends_empty_command() {
        let (mut app, command_receiver) = test_app();
        app.session.set_login_name("tester".to_string());
        app.session
            .update_login_state("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >");

        app.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(command_receiver.try_recv().as_deref(), Ok(""));
    }

    #[test]
    fn enter_on_empty_pre_login_input_sends_empty_command() {
        let (mut app, command_receiver) = test_app();

        app.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(command_receiver.try_recv().as_deref(), Ok(""));
    }

    #[test]
    fn delete_removes_character_after_command_input_cursor() {
        let (mut app, _command_receiver) = test_app();
        app.input.insert_str("look");
        app.input.move_cursor_left();

        app.handle_key_event(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));

        assert_eq!(app.input.displayed_input(), "loo");
    }

    #[test]
    fn delete_removes_character_from_settings_dialog_value() {
        let (mut app, _command_receiver) = test_app();
        app.settings_dialog = Some(SettingsDialog::new(vec![crate::config::SettingEntry {
            key: "mount".to_string(),
            value: "wolf".to_string(),
        }]));

        app.handle_key_event(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));

        let dialog = app.settings_dialog.as_ref().expect("settings dialog");
        assert_eq!(dialog.entries()[0].value, "wol");
    }

    #[test]
    fn mouse_wheel_scrolls_output_without_changing_command_history() {
        let (mut app, _command_receiver) = test_app();
        log_in(&mut app);
        app.input.push_history("look".to_string());
        app.input.insert_str("current");
        app.scrollback.update_viewport(100, 20);

        app.handle_mouse_event(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(app.scrollback.offset(), 77);
        assert_eq!(app.input.displayed_input(), "current");
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

    #[test]
    fn command_effect_redraw_sets_pending_terminal_clear_without_mutating_output_or_scrollback() {
        let (mut app, _command_receiver) = test_app();
        for index in 0..100 {
            app.output
                .append_lines(vec![StyledLine::new(&format!("line {index}"))]);
        }
        app.scrollback.update_viewport(100, 20);
        app.scrollback.page_up();
        let line_count_before = app.output.plain_lines().len();
        let scroll_offset_before = app.scrollback.offset();

        let followed = app.apply_command_effects(vec![command::CommandEffect::Redraw]);

        assert!(!followed);
        assert!(app.take_pending_terminal_clear());
        assert!(!app.take_pending_terminal_clear());
        assert_eq!(app.output.plain_lines().len(), line_count_before);
        assert_eq!(app.scrollback.offset(), scroll_offset_before);
    }

    #[test]
    fn reconnect_effect_starts_fresh_connection_and_replaces_command_sender() {
        let (new_channels, new_command_receiver, _new_event_sender) = connection_channels();
        let (coordinator, calls, _results) =
            FakeConnectionCoordinator::new(vec![ReconnectResult::Connected(new_channels)]);
        let (mut app, old_command_receiver, _event_sender) =
            test_app_with_coordinator(Box::new(coordinator));
        app.session.set_login_name("tester".to_string());
        app.session
            .update_login_state("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >");

        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);
        app.apply_command_effects(vec![command::CommandEffect::Send("look".into())]);

        assert_eq!(*calls.borrow(), vec![1]);
        assert_eq!(app.output.plain_lines(), vec!["Reconnect started."]);
        assert!(!app.session.is_logged_in());
        assert!(old_command_receiver.try_recv().is_err());
        assert_eq!(new_command_receiver.try_recv().as_deref(), Ok("look"));
    }

    #[test]
    fn reconnect_failure_reports_error_keeps_fresh_state_and_allows_retry() {
        let (new_channels, _new_command_receiver, _new_event_sender) = connection_channels();
        let (coordinator, calls, _results) = FakeConnectionCoordinator::new(vec![
            ReconnectResult::Failed("socket refused".to_string()),
            ReconnectResult::Connected(new_channels),
        ]);
        let (mut app, _command_receiver, _event_sender) =
            test_app_with_coordinator(Box::new(coordinator));
        app.session.set_login_name("tester".to_string());
        app.session
            .update_login_state("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >");

        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);
        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);

        assert_eq!(*calls.borrow(), vec![1, 2]);
        assert_eq!(
            app.output.plain_lines(),
            vec![
                "Reconnect started.",
                "Reconnect failed: socket refused",
                "Reconnect started."
            ]
        );
        assert!(!app.session.is_logged_in());
    }

    #[test]
    fn reconnect_rejects_duplicate_attempt_while_pending() {
        let (coordinator, calls, _results) = FakeConnectionCoordinator::new(Vec::new());
        let (mut app, _command_receiver, _event_sender) =
            test_app_with_coordinator(Box::new(coordinator));
        app.session_lifecycle.set_reconnect_in_progress(true);

        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);

        assert!(calls.borrow().is_empty());
        assert_eq!(
            app.output.plain_lines(),
            vec!["Reconnect already in progress."]
        );
    }

    #[test]
    fn reconnect_completion_clears_duplicate_guard() {
        let (new_channels, _new_command_receiver, _new_event_sender) = connection_channels();
        let (coordinator, calls, _results) =
            FakeConnectionCoordinator::new(vec![ReconnectResult::Connected(new_channels)]);
        let (mut app, _command_receiver, _event_sender) =
            test_app_with_coordinator(Box::new(coordinator));

        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);
        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);

        assert_eq!(*calls.borrow(), vec![1, 2]);
    }

    #[test]
    fn reconnect_resets_login_dependent_runtime_state_but_keeps_visible_history() {
        let (new_channels, _new_command_receiver, _new_event_sender) = connection_channels();
        let (coordinator, _calls, _results) =
            FakeConnectionCoordinator::new(vec![ReconnectResult::Connected(new_channels)]);
        let (mut app, _command_receiver, _event_sender) =
            test_app_with_coordinator(Box::new(coordinator));
        app.output.append_lines(vec![StyledLine::new("old output")]);
        app.input.push_history("look".to_string());
        app.session.set_login_name("tester".to_string());
        app.session
            .update_login_state("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >");
        app.apply_guild_selection(GuildSelection::from_playable_keys(
            [GuildKey::Disciple],
            Some("magical"),
        ));
        app.generic_commands
            .apply_config(&["cure_spells".to_string()], &["clw".to_string()]);
        app.automation.set_var("rig", "satchel".to_string());
        app.automation.set_flag("in_battle", true);
        app.secondary_status.apply_effect(
            crate::secondary_status::SecondaryStatusEffect::SetTzarakkMountStatus {
                name: "horse".to_string(),
                percent: 100,
                description: "healthy".to_string(),
            },
        );
        app.guild_dialog = Some(GuildDialog::new(
            catalog::playable_entries_list(),
            Vec::new(),
            DEFAULT_GUILD_PRIMARY_KEYWORD,
            String::new(),
            String::new(),
            Default::default(),
        ));

        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);

        assert_eq!(
            app.output.plain_lines(),
            vec!["old output", "Reconnect started."]
        );
        app.input.move_history(-1);
        assert_eq!(app.input.displayed_input(), "look");
        assert_eq!(app.session.login_state(), LoginState::Choice);
        assert!(app.selected_guilds.is_empty());
        assert_eq!(
            app.generic_commands.render_command("clw", ""),
            Some("@cast 'cure light wounds' me".to_string())
        );
        assert!(app.automation.get_var("rig").is_none());
        assert!(!app.automation.flag_is_set("in_battle"));
        assert!(!app.secondary_status.has_tzarakk_mount_status());
        assert!(app.guild_dialog.is_none());
    }

    #[test]
    fn reconnect_defers_profile_reload_until_next_login() {
        let (new_channels, _new_command_receiver, _new_event_sender) = connection_channels();
        let (coordinator, _calls, _results) =
            FakeConnectionCoordinator::new(vec![ReconnectResult::Connected(new_channels)]);
        let (mut app, _command_receiver, _event_sender) =
            test_app_with_coordinator(Box::new(coordinator));
        app.user_config_loaded = true;
        app.session.set_login_name("tester".to_string());
        app.session
            .update_login_state("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >");

        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);

        assert!(!app.user_config_loaded);

        app.session.set_login_name("tester".to_string());
        app.process_input_lines(vec!["Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >".to_string()]);

        assert!(app.user_config_loaded);
    }

    #[test]
    fn reconnect_ignores_stale_connection_events() {
        let (coordinator, _calls, _results) =
            FakeConnectionCoordinator::new(vec![ReconnectResult::Failed("offline".to_string())]);
        let (mut app, _command_receiver, event_sender) =
            test_app_with_coordinator(Box::new(coordinator));

        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);
        event_sender
            .send(AppEvent::Telnet {
                connection_id: INITIAL_CONNECTION_ID,
                event: TelnetEvents::DataReceive(bytes::Bytes::from_static(b"stale output\r\n")),
            })
            .unwrap();
        event_sender
            .send(AppEvent::Telnet {
                connection_id: 1,
                event: TelnetEvents::DataReceive(bytes::Bytes::from_static(b"fresh output\r\n")),
            })
            .unwrap();

        app.read_input();

        assert_eq!(
            app.output.plain_lines(),
            vec![
                "Reconnect started.",
                "Reconnect failed: offline",
                "fresh output"
            ]
        );
    }

    #[test]
    fn reconnect_same_character_preserves_output_and_scrollback_on_login() {
        let (new_channels, _new_command_receiver, _new_event_sender) = connection_channels();
        let (coordinator, _calls, _results) =
            FakeConnectionCoordinator::new(vec![ReconnectResult::Connected(new_channels)]);
        let (mut app, _command_receiver, _event_sender) =
            test_app_with_coordinator(Box::new(coordinator));
        for index in 0..100 {
            app.output
                .append_lines(vec![StyledLine::new(&format!("line {index}"))]);
        }
        app.scrollback.update_viewport(100, 20);
        app.scrollback.page_up();
        let scroll_offset_before = app.scrollback.offset();
        log_in(&mut app);

        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);

        app.session.set_login_name("tester".to_string());
        app.process_input_lines(vec!["Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >".to_string()]);

        assert!(
            app.output
                .plain_lines()
                .iter()
                .any(|line| line.starts_with("line "))
        );
        assert_eq!(app.scrollback.offset(), scroll_offset_before);
    }

    #[test]
    fn reconnect_different_character_clears_output_and_resets_scrollback_on_login() {
        let (new_channels, _new_command_receiver, _new_event_sender) = connection_channels();
        let (coordinator, _calls, _results) =
            FakeConnectionCoordinator::new(vec![ReconnectResult::Connected(new_channels)]);
        let (mut app, _command_receiver, _event_sender) =
            test_app_with_coordinator(Box::new(coordinator));
        app.output.append_lines(vec![StyledLine::new("old output")]);
        app.scrollback.update_viewport(1, 20);
        app.scrollback.page_up();
        log_in(&mut app);

        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);

        app.session.set_login_name("other".to_string());
        app.process_input_lines(vec!["Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >".to_string()]);

        assert!(app.output.plain_lines().is_empty());
        assert_eq!(app.scrollback.offset(), 0);
    }

    #[test]
    fn reconnect_before_login_clears_output_on_first_login() {
        let (new_channels, _new_command_receiver, _new_event_sender) = connection_channels();
        let (coordinator, _calls, _results) =
            FakeConnectionCoordinator::new(vec![ReconnectResult::Connected(new_channels)]);
        let (mut app, _command_receiver, _event_sender) =
            test_app_with_coordinator(Box::new(coordinator));
        app.output
            .append_lines(vec![StyledLine::new("pre-connect output")]);

        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);

        app.session.set_login_name("tester".to_string());
        app.process_input_lines(vec!["Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >".to_string()]);

        assert!(app.output.plain_lines().is_empty());
        assert_eq!(app.scrollback.offset(), 0);
    }

    #[test]
    fn reconnect_retry_after_failure_preserves_output_for_same_character_login() {
        let (new_channels, _new_command_receiver, _new_event_sender) = connection_channels();
        let (coordinator, _calls, _results) = FakeConnectionCoordinator::new(vec![
            ReconnectResult::Failed("socket refused".to_string()),
            ReconnectResult::Connected(new_channels),
        ]);
        let (mut app, _command_receiver, _event_sender) =
            test_app_with_coordinator(Box::new(coordinator));
        app.output.append_lines(vec![StyledLine::new("old output")]);
        log_in(&mut app);

        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);
        app.apply_command_effects(vec![command::CommandEffect::Reconnect]);

        app.session.set_login_name("tester".to_string());
        app.process_input_lines(vec!["Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >".to_string()]);

        assert!(app.output.plain_lines().contains(&"old output"));
    }

    #[test]
    fn paste_inserts_text_into_current_input() {
        let (mut app, _command_receiver) = test_app();

        app.handle_paste_event("look very carefully".to_string());

        assert_eq!(app.input.displayed_input(), "look very carefully");
    }

    #[test]
    fn multiline_paste_submits_completed_lines_and_keeps_remainder() {
        let (mut app, command_receiver) = test_app();
        app.session.set_login_name("tester".to_string());
        app.session
            .update_login_state("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >");

        app.handle_paste_event("look\r\nsay hello\nnorth".to_string());

        assert_eq!(command_receiver.try_recv().as_deref(), Ok("look"));
        assert_eq!(command_receiver.try_recv().as_deref(), Ok("say hello"));
        assert_eq!(app.input.displayed_input(), "north");
    }
}
