use crate::ansi::StyledLine;
use crate::automation::{Action, Automation};
use crate::guilds::{Guild, ReaverGuild};
use crate::stats::Stats;
use crate::{command, triggers};
use bytes::{BufMut, BytesMut};
use chrono::{DateTime, Local, Timelike};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use libmudtelnet::events::TelnetEvents;
use libmudtelnet::telnet::op_command;
use log::debug;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::BufRead;
use std::mem;
use std::sync::mpsc::{Receiver, Sender};
use unicode_segmentation::UnicodeSegmentation;

static CARRIAGE_RETURN_NEW_LINE: &[u8] = &[13, 10];

pub struct BatApp {
    pub lines: Vec<StyledLine>,
    pub current_typed_input: String,
    pub displayed_input: String,
    pub stats: Stats,
    pub event_receiver: Receiver<TelnetEvents>,
    pub command_sender: Sender<String>,
    pub buffer: Option<BytesMut>,
    pub selected_guilds: Vec<Box<dyn Guild>>,
    pub should_quit: bool,
    pub history: Vec<String>,
    pub cur_history_pos: usize,
    pub automation: Automation,
    pub login_state: LoginState,
    pub login_name: Option<String>,
    pub last_login_input: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginState {
    Choice,
    Name,
    Password,
    LoggedIn,
}

impl BatApp {
    pub fn new(
        event_receiver: Receiver<TelnetEvents>,
        command_sender: Sender<String>,
    ) -> Self {
        let mut app = BatApp {
            lines: vec![],
            current_typed_input: String::new(),
            displayed_input: String::new(),
            stats: Default::default(),
            event_receiver,
            command_sender,
            buffer: Some(BytesMut::with_capacity(1024)),
            selected_guilds: vec![Box::new(ReaverGuild::default())],
            should_quit: false,
            history: vec![],
            cur_history_pos: 1,
            automation: Automation::new(),
            login_state: LoginState::Choice,
            login_name: None,
            last_login_input: None,
        };

        for guild in &app.selected_guilds {
            guild.register_automation(&mut app.automation);
        }

        app
    }

    fn handle_event(&mut self, event: &TelnetEvents) {
        match event {
            TelnetEvents::IAC(iac) => {
                debug!("IAC: {iac:?}");
                if op_command::GA == iac.command {
                    let buffer = self.buffer.replace(BytesMut::with_capacity(1024)).unwrap();
                    self.process_input_data(buffer);
                }
            }
            TelnetEvents::Negotiation(neg) => {
                debug!("Negotiation: {neg:?}");
            }
            TelnetEvents::Subnegotiation(sub_neg) => {
                debug!("Subnegotiation: {sub_neg:?}");
            }
            TelnetEvents::DataReceive(bytes) => {
                if !bytes.ends_with(CARRIAGE_RETURN_NEW_LINE) {
                    if let Some(buffer) = &mut self.buffer {
                        buffer.put(bytes.clone());
                    }
                    return;
                }

                let mut buffer = self.buffer.replace(BytesMut::with_capacity(1024)).unwrap();
                buffer.put(bytes.clone());

                self.process_input_data(buffer);
            }
            TelnetEvents::DataSend(_) => {}
            TelnetEvents::DecompressImmediate(_) => {
                debug!("Decompress data");
            }
        }
    }

    #[allow(clippy::lines_filter_map_ok)]
    fn process_input_data(&mut self, bytes: BytesMut) {
        let mut lines = Vec::new();

        for line in bytes.lines().filter_map(Result::ok) {
            let mut styled_line = StyledLine::new(&line);
            self.update_login_state(&styled_line.plain_line);

            if self.is_logged_in() {
                let mut new_lines = triggers::process(self, &mut styled_line);
                new_lines.push(styled_line);
                lines.extend(new_lines);
            } else {
                lines.push(styled_line);
            }
        }

        if self.is_logged_in() {
            for line in &lines {
                self.run_automation(&line.plain_line);
            }
        }

        remove_gagged_lines(&mut lines);

        self.lines.append(&mut lines);
    }

    fn update_login_state(&mut self, line: &str) {
        let line = line.trim_end();
        if line == "You entered a wrong password!" {
            self.login_state = LoginState::Choice;
            self.login_name = None;
            self.last_login_input = None;
            self.clear_input();
            return;
        }

        if line == "Please enter your choice or name:" {
            self.login_state = LoginState::Choice;
            self.login_name = None;
            self.last_login_input = None;
            self.clear_input();
            return;
        }

        if line.starts_with("What is your name:") {
            self.login_state = LoginState::Name;
            self.clear_input();
            return;
        }

        if line.starts_with("Password:") {
            self.login_state = LoginState::Password;
            if self.login_name.is_none() {
                self.login_name = self.last_login_input.clone();
            }
            self.clear_input();
            return;
        }

        if !self.is_logged_in()
            && (PROMPT_REGEX.is_match(line) || SHORT_SCORE_REGEX.is_match(line))
        {
            self.login_state = LoginState::LoggedIn;
        }
    }

    fn clear_input(&mut self) {
        self.displayed_input.clear();
        self.current_typed_input.clear();
    }

    fn is_logged_in(&self) -> bool {
        self.login_state == LoginState::LoggedIn
    }
    pub fn read_input(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(&event);
        }
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Enter => self.submit_input(),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Up => {
                if self.is_logged_in() {
                    self.move_history(-1);
                }
            }
            KeyCode::Down => {
                if self.is_logged_in() {
                    self.move_history(1);
                }
            }
            KeyCode::Char(c) => {
                if !event.modifiers.contains(KeyModifiers::CONTROL)
                    && !event.modifiers.contains(KeyModifiers::ALT)
                {
                    self.insert_char(c);
                }
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

        let output_area = root[0];
        let stats_area = root[1];
        let input_area = root[2];

        let visible_height = output_area.height.saturating_sub(1) as usize;
        let output_lines: Vec<Line<'_>> = self.lines.iter().map(StyledLine::to_line).collect();
        let scroll_offset = self.lines.len().saturating_sub(visible_height);
        let scroll_offset = scroll_offset.min(u16::MAX as usize) as u16;
        let output = Paragraph::new(Text::from(output_lines)).scroll((scroll_offset, 0));
        frame.render_widget(output, output_area);

        if self.is_logged_in() {
            let stats_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(10), Constraint::Length(12)])
                .split(stats_area);

            let stats_line = self.stats.render_inline();
            let stats_widget = Paragraph::new(stats_line);
            frame.render_widget(stats_widget, stats_chunks[0]);

            let clock = show_clock();
            let clock_widget = Paragraph::new(clock).alignment(Alignment::Center);
            frame.render_widget(clock_widget, stats_chunks[1]);
        } else {
            let clock = show_clock();
            let clock_widget = Paragraph::new(clock).alignment(Alignment::Right);
            frame.render_widget(clock_widget, stats_area);
        }

        let input_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(12)])
            .split(input_area);

        let input_block = Block::default();
        let input_text = format!("> {}", self.displayed_input_text());
        let input = Paragraph::new(input_text)
            .block(input_block.clone())
            .wrap(Wrap { trim: false });
        frame.render_widget(input, input_chunks[0]);

        frame.render_widget(Paragraph::new(""), input_chunks[1]);

        let input_inner = input_block.inner(input_chunks[0]);
        if input_inner.width > 0 && input_inner.height > 0 {
            let cursor_offset = self.cursor_offset();
            let cursor_x = input_inner
                .x
                .saturating_add(cursor_offset.min(input_inner.width.saturating_sub(1)));
            let cursor_y = input_inner.y;
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }

    fn insert_char(&mut self, c: char) {
        self.displayed_input.push(c);
        self.current_typed_input.clone_from(&self.displayed_input);
        self.cur_history_pos = self.history.len();
    }

    fn backspace(&mut self) {
        if let Some((index, _)) = self.displayed_input.grapheme_indices(true).last() {
            self.displayed_input.truncate(index);
            self.current_typed_input.clone_from(&self.displayed_input);
            self.cur_history_pos = self.history.len();
        }
    }

    fn move_history(&mut self, direction: i32) {
        let prev_pos = self.cur_history_pos;

        if direction < 0 {
            if self.cur_history_pos > 0 {
                self.cur_history_pos = self.cur_history_pos.saturating_sub(1);
            }
        } else if self.cur_history_pos < self.history.len() {
            self.cur_history_pos = self.cur_history_pos.saturating_add(1);
        }

        if prev_pos != self.cur_history_pos {
            if self.cur_history_pos < self.history.len() {
                self.displayed_input
                    .clone_from(&self.history[self.cur_history_pos]);
            } else if self.cur_history_pos == self.history.len() {
                self.displayed_input.clone_from(&self.current_typed_input);
            }
        }
    }

    fn submit_input(&mut self) {
        if !self.is_logged_in() {
            let input = mem::take(&mut self.displayed_input);
            if input.is_empty() {
                return;
            }

            if input.starts_with('/') {
                let mut ctx = command::CommandContext::new(self.automation.snapshot_flags());
                let cmd = command::process(&input, &mut ctx, &self.selected_guilds);
                if ctx.should_quit {
                    self.should_quit = true;
                    return;
                }
                if let Some(s) = cmd {
                    self.send_command(s);
                }
            } else {
                if self.login_state == LoginState::Name {
                    self.login_name = Some(input.clone());
                }
                if self.login_state == LoginState::Choice {
                    self.last_login_input = Some(input.clone());
                }
                self.send_command(input);
            }

            self.current_typed_input.clear();
            return;
        }

        let mut ctx = command::CommandContext::new(self.automation.snapshot_flags());
        let cmd = command::process(&self.displayed_input, &mut ctx, &self.selected_guilds);

        if ctx.should_quit {
            self.should_quit = true;
            return;
        }

        self.apply_automation_actions(ctx.automation_actions);

        if let Some(s) = cmd {
            self.send_command(s);
        }

        self.history.push(mem::take(&mut self.displayed_input));
        self.cur_history_pos = self.history.len();
    }

    fn displayed_input_text(&self) -> String {
        if self.login_state == LoginState::Password {
            String::new()
        } else {
            self.displayed_input.clone()
        }
    }

    fn cursor_offset(&self) -> u16 {
        if self.login_state == LoginState::Password {
            2
        } else {
            self.displayed_input.graphemes(true).count() as u16 + 2
        }
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

fn show_clock() -> String {
    let local: DateTime<Local> = Local::now();
    format!(
        "{:02}:{:02}:{:02}",
        local.hour(),
        local.minute(),
        local.second()
    )
}

lazy_static! {
    static ref PROMPT_REGEX: Regex =
        Regex::new(r"^Hp:(.+)/(.+) Sp:(.+)/(.+) Ep:(.+)/(.+) Exp:(.+) >$").unwrap();
    static ref SHORT_SCORE_REGEX: Regex =
        Regex::new(r"^H:(.+)/(.+) \[(.*)\] S:(.+)/(.+) \[(.*)\] E:(.+)/(.+) \[(.*)\] \\$:(.+) \[(.*)\] exp:(.+) \[(.*)\]$")
            .unwrap();
}

fn remove_gagged_lines(lines: &mut Vec<StyledLine>) {
    let num_lines = lines.len();
    let mut indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| if line.gag { index } else { num_lines + 1 })
        .filter(|index| *index < num_lines + 1)
        .collect();

    indices.sort_by(|a, b| b.cmp(a));

    indices.iter().for_each(|index| {
        lines.remove(*index);
    });
}
