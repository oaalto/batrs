use crate::ansi::StyledLine;
use std::io::{BufRead, Read, Write};
use std::net::TcpStream;

use crate::stats::Stats;
use crate::triggers;
use bytes::{BufMut, BytesMut};
use egui::{FontId, ScrollArea, TextStyle};
use libmudtelnet::events::TelnetEvents;
use libmudtelnet::telnet::op_command;
use libmudtelnet::Parser;

pub struct BatApp {
    pub lines: Vec<StyledLine>,
    pub input: String,
    pub send_input: bool,
    pub buffer: Option<BytesMut>,
    pub stats: Stats,
    pub stream: TcpStream,
    pub parser: Parser,
}

impl BatApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, stream: TcpStream) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        cc.egui_ctx.style_mut(|style| {
            let monospace = FontId::monospace(16.0);
            style.override_font_id = Some(monospace);
        });

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        /*if let Some(storage) = cc.storage {
                    let saved_state = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
                    return BatApp {
                        state: saved_state,
                        ..Default::default()
                    };
                }
        */
        BatApp {
            lines: vec![],
            input: "".to_string(),
            send_input: false,
            buffer: Some(BytesMut::with_capacity(1024)),
            stats: Default::default(),
            stream,
            parser: Parser::new(),
        }
    }

    fn handle_event(&mut self, event: &TelnetEvents) {
        match event {
            TelnetEvents::IAC(iac) => {
                if op_command::GA == iac.command {
                    let buffer = self.buffer.replace(BytesMut::with_capacity(1024)).unwrap();

                    let mut lines = buffer
                        .lines()
                        .map(|l| l.unwrap_or_default())
                        .map(|line| StyledLine::new(&line))
                        // TODO: a better way than this...
                        .map(|mut styled_line| {
                            process_triggers(self, &mut styled_line);
                            styled_line
                        })
                        .collect();

                    self.lines.append(&mut lines);
                } else {
                    println!("Unknown IAC Command: {}", iac.command);
                }
            }
            TelnetEvents::Negotiation(neg) => {
                println!("Negotiation: {:?}", neg);
            }
            TelnetEvents::Subnegotiation(sub_neg) => {
                println!("Subnegotiation: {:?}", sub_neg);
            }
            TelnetEvents::DataReceive(bytes) => {
                if let Some(buffer) = &mut self.buffer {
                    buffer.put(bytes.clone());
                }
            }
            TelnetEvents::DataSend(_) => {}
            TelnetEvents::DecompressImmediate(_) => {
                println!("Decompress data");
            }
        }
    }

    fn read_input(&mut self) {
        let mut buffer = [0; 1024];

        if self.stream.read(&mut buffer[..]).is_ok() {
            let events = self.parser.receive(&buffer);
            for event in events {
                self.handle_event(&event);
            }
        }
    }

    fn send_output(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            let events = self.parser.send_text(&self.input);
            if let Err(e) = self.stream.write_all(&events.to_bytes()) {
                eprintln!("failed to send data: {}", e);
            }
            self.input.clear();
        }
    }
}

impl eframe::App for BatApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.read_input();

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.

        egui::TopBottomPanel::bottom("input_panel").show(ctx, |ui| {
            let response = ui.add_sized(
                ui.available_size(),
                egui::TextEdit::singleline(&mut self.input),
            );
            response.request_focus();
        });

        egui::SidePanel::right("stats").show(ctx, |ui| {
            self.stats.show(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let text_style = TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);

            ScrollArea::vertical().stick_to_bottom(true).show_rows(
                ui,
                row_height,
                self.lines.len(),
                |ui, row_range| {
                    ui.set_min_size(ui.available_size());

                    for row in row_range {
                        self.lines[row].show(ui);
                    }
                },
            );
        });

        self.send_output(ctx);

        ctx.request_repaint();
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }
}

fn process_triggers(app: &mut BatApp, styled_line: &mut StyledLine) {
    triggers::TRIGGERS
        .iter()
        .for_each(|trigger| trigger.process(app, styled_line));
}
