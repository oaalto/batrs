use crate::ansi::StyledLine;
use crate::guilds::{Guild, ReaverGuild};
use crate::stats::Stats;
use crate::{command, triggers};
use bytes::{BufMut, BytesMut};
use chrono::{DateTime, Local, Timelike};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use libmudtelnet::events::TelnetEvents;
use libmudtelnet::telnet::op_command;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
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
}

impl BatApp {
    pub fn new(
        event_receiver: Receiver<TelnetEvents>,
        command_sender: Sender<String>,
    ) -> Self {
        BatApp {
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
        }
    }

    fn handle_event(&mut self, event: &TelnetEvents) {
        match event {
            TelnetEvents::IAC(iac) => {
                println!("IAC: {iac:?}");
                if op_command::GA == iac.command {
                    let buffer = self.buffer.replace(BytesMut::with_capacity(1024)).unwrap();
                    self.process_input_data(buffer);
                }
            }
            TelnetEvents::Negotiation(neg) => {
                println!("Negotiation: {neg:?}");
            }
            TelnetEvents::Subnegotiation(sub_neg) => {
                println!("Subnegotiation: {sub_neg:?}");
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
                println!("Decompress data");
            }
        }
    }

    #[allow(clippy::lines_filter_map_ok)]
    fn process_input_data(&mut self, bytes: BytesMut) {
        let mut lines = bytes
            .lines()
            .filter_map(Result::ok)
            .map(|line| StyledLine::new(&line))
            .flat_map(|mut styled_line| {
                let mut new_lines = triggers::process(self, &mut styled_line);
                new_lines.push(styled_line);
                new_lines
            })
            .collect();

        remove_gagged_lines(&mut lines);

        self.lines.append(&mut lines);
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
            KeyCode::Up => self.move_history(-1),
            KeyCode::Down => self.move_history(1),
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
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(frame.area());

        let main_area = root[0];
        let input_area = root[1];

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(1)])
            .split(main_area);

        self.stats.render(frame, main_chunks[0]);

        let output_area = main_chunks[1];
        let visible_height = output_area.height.saturating_sub(2) as usize;
        let output_lines = self.visible_lines(visible_height);
        let output = Paragraph::new(Text::from(output_lines))
            .block(Block::default().title("Output").borders(Borders::ALL))
            .wrap(Wrap { trim: false });
        frame.render_widget(output, output_area);

        let input_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(12)])
            .split(input_area);

        let input_block = Block::default().title("Input").borders(Borders::ALL);
        let input = Paragraph::new(self.displayed_input.as_str())
            .block(input_block.clone())
            .wrap(Wrap { trim: false });
        frame.render_widget(input, input_chunks[0]);

        let clock = show_clock();
        let clock_widget = Paragraph::new(clock)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(clock_widget, input_chunks[1]);

        let input_inner = input_block.inner(input_chunks[0]);
        if input_inner.width > 0 && input_inner.height > 0 {
            let cursor_offset = self.displayed_input.graphemes(true).count() as u16;
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
        let mut ctx = command::CommandContext::default();
        let cmd = command::process(&self.displayed_input, &mut ctx, &self.selected_guilds);

        if ctx.should_quit {
            self.should_quit = true;
            return;
        }

        if let Some(s) = cmd {
            if let Err(e) = self.command_sender.send(s) {
                eprintln!("failed to send data: {e}");
            }
        }

        self.history.push(mem::take(&mut self.displayed_input));
        self.cur_history_pos = self.history.len();
    }

    fn visible_lines(&self, height: usize) -> Vec<Line<'_>> {
        if height == 0 || self.lines.is_empty() {
            return Vec::new();
        }

        let start = self.lines.len().saturating_sub(height);
        self.lines[start..]
            .iter()
            .map(StyledLine::to_line)
            .collect()
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
