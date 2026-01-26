mod input_state;
mod output_buffer;
mod session_state;
mod telnet_buffer;

use crate::ansi::StyledLine;
use crate::automation::{Action, Automation};
use crate::config::ConfigManager;
use crate::guilds::{Guild, ReaverGuild};
use crate::stats::Stats;
use crate::ui::{Renderer, ViewModel};
use crate::{command, triggers};
use chrono::{DateTime, Local, Timelike};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use input_state::InputState;
use libmudtelnet::events::TelnetEvents;
use ratatui::text::Line;
use ratatui::Frame;
use std::sync::mpsc::{Receiver, Sender};
use telnet_buffer::TelnetBuffer;

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
    should_quit: bool,
    automation: Automation,
    config_manager: Option<ConfigManager>,
    user_config_loaded: bool,
}

impl BatApp {
    pub fn new(
        event_receiver: Receiver<TelnetEvents>,
        command_sender: Sender<String>,
    ) -> Self {
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
            selected_guilds: vec![Box::new(ReaverGuild::default())],
            should_quit: false,
            automation: Automation::new(),
            config_manager,
            user_config_loaded: false,
        };

        for guild in &app.selected_guilds {
            guild.register_automation(&mut app.automation);
        }

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

            if self.session.is_logged_in() {
                let mut ctx = triggers::TriggerContext {
                    stats: &mut self.stats,
                };
                let mut new_lines =
                    triggers::process(&mut ctx, &self.selected_guilds, &mut styled_line);
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
        match event.code {
            KeyCode::Enter => self.submit_input(),
            KeyCode::Backspace => self.input.backspace(),
            KeyCode::Up => {
                if self.session.is_logged_in() {
                    self.input.move_history(-1);
                }
            }
            KeyCode::Down => {
                if self.session.is_logged_in() {
                    self.input.move_history(1);
                }
            }
            KeyCode::Char(c) => {
                if !event.modifiers.contains(KeyModifiers::CONTROL)
                    && !event.modifiers.contains(KeyModifiers::ALT)
                {
                    self.input.insert_char(c);
                }
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        let output_area_height = frame.area().height.saturating_sub(2);
        let visible_height = output_area_height.saturating_sub(1) as usize;
        let output_lines: Vec<Line<'_>> =
            self.output.lines().iter().map(StyledLine::to_line).collect();
        let scroll_offset = self.output.lines().len().saturating_sub(visible_height);
        let scroll_offset = scroll_offset.min(u16::MAX as usize) as u16;
        let show_stats = self.session.is_logged_in();
        let stats_line = if show_stats {
            self.stats.render_inline()
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
            clock: show_clock(),
            input_text,
            cursor_offset: self.input.cursor_offset(hide_input),
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
                let mut ctx = command::CommandContext::new(self.automation.snapshot_flags());
                let outcome = command::process(&input, &mut ctx, &self.selected_guilds);
                if outcome.should_quit {
                    self.should_quit = true;
                    return;
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

        let mut ctx = command::CommandContext::new(self.automation.snapshot_flags());
        let outcome =
            command::process(self.input.displayed_input(), &mut ctx, &self.selected_guilds);

        if outcome.should_quit {
            self.should_quit = true;
            return;
        }

        self.apply_automation_actions(outcome.automation_actions);

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

    fn send_command(&mut self, command: String) {
        if let Err(e) = self.command_sender.send(command) {
            eprintln!("failed to send data: {e}");
        }
    }

    // TODO: keep around scroll position when manual scrolling is added.
}

impl BatApp {
    fn load_user_config(&mut self) {
        if self.user_config_loaded {
            return;
        }
        self.user_config_loaded = true;
        let Some(manager) = self.config_manager.as_mut() else {
            return;
        };
        let Some(player_name) = self.session.login_name() else {
            eprintln!("logged in without a known player name; skipping user config");
            return;
        };
        if let Err(e) = manager.load_user(player_name) {
            eprintln!("failed to load user config for {player_name}: {e}");
        }
    }
}

fn show_clock() -> String {
    let local: DateTime<Local> = Local::now();
    format!(
        "{:02}:{:02}:{:02}",
        local.hour(),
        local.minute(),
        local.second()
    )
}

