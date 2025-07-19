use crate::ansi::StyledLine;
use crate::guilds::{Guild, ReaverGuild};
use crate::stats::Stats;
use crate::{command, triggers};
use bytes::{BufMut, BytesMut};
use chrono::{DateTime, Local, Timelike};
use egui::{Color32, FontId, ScrollArea, TextStyle, Ui, ViewportCommand};
use libmudtelnet::events::TelnetEvents;
use libmudtelnet::telnet::op_command;
use std::cmp::{max, min};
use std::io::BufRead;
use std::mem;
use std::sync::mpsc::{Receiver, Sender};

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
    pub fullscreen: bool,
    pub history: Vec<String>,
    pub cur_history_pos: usize,
}

impl BatApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        event_receiver: Receiver<TelnetEvents>,
        command_sender: Sender<String>,
    ) -> Self {
        cc.egui_ctx.style_mut(|style| {
            let monospace = FontId::monospace(16.0);
            style.override_font_id = Some(monospace);
            style.visuals.panel_fill = Color32::BLACK;
        });

        cc.egui_ctx
            .send_viewport_cmd(ViewportCommand::Maximized(true));

        cc.egui_ctx
            .send_viewport_cmd(ViewportCommand::Fullscreen(true));

        BatApp {
            lines: vec![],
            current_typed_input: String::new(),
            displayed_input: String::new(),
            stats: Default::default(),
            event_receiver,
            command_sender,
            buffer: Some(BytesMut::with_capacity(1024)),
            selected_guilds: vec![Box::new(ReaverGuild::default())],
            fullscreen: true,
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

    fn read_input(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(&event);
        }
    }

    fn send_output(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            let cmd = command::process(&self.displayed_input, ctx, &self.selected_guilds);

            if let Some(s) = cmd {
                if let Err(e) = self.command_sender.send(s) {
                    eprintln!("failed to send data: {e}");
                }
            }

            self.history.push(mem::take(&mut self.displayed_input));
            self.cur_history_pos = self.history.len();
        }
    }

    fn handle_history_input(&mut self, ctx: &egui::Context) {
        let prev_pos = self.cur_history_pos;

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.cur_history_pos = max(
                0,
                self.cur_history_pos
                    .checked_sub(1)
                    .unwrap_or(self.cur_history_pos),
            );
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.cur_history_pos = min(
                self.history.len(),
                self.cur_history_pos
                    .checked_add(1)
                    .unwrap_or(self.cur_history_pos),
            );
        }

        if prev_pos != self.cur_history_pos {
            #[allow(clippy::comparison_chain)]
            if self.cur_history_pos < self.history.len() {
                self.displayed_input
                    .clone_from(&self.history[self.cur_history_pos]);
            } else if self.cur_history_pos == self.history.len() {
                self.displayed_input.clone_from(&self.current_typed_input);
            }
        }
    }
}

impl eframe::App for BatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.read_input();

        egui::SidePanel::left("stats").show(ctx, |ui| {
            self.stats.show(ui);
        });

        egui::TopBottomPanel::bottom("input_panel").show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center).with_cross_justify(true),
                |ui| {
                    show_clock(ui);

                    let response = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::singleline(&mut self.displayed_input),
                    );
                    response.request_focus();
                    if response.changed() {
                        self.current_typed_input.clone_from(&self.displayed_input);
                        self.cur_history_pos = self.history.len();
                    }
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let text_style = TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);

            ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show_rows(ui, row_height, self.lines.len(), |ui, row_range| {
                    for row in row_range {
                        self.lines[row].show(ui);
                    }
                });
        });

        self.handle_history_input(ctx);

        self.send_output(ctx);

        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            self.fullscreen = !self.fullscreen;
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(self.fullscreen));
        }

        ctx.request_repaint();
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}
}

fn show_clock(ui: &mut Ui) {
    let local: DateTime<Local> = Local::now();
    ui.label(format!(
        "{:02}:{:02}:{:02}",
        local.hour(),
        local.minute(),
        local.second()
    ));
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
