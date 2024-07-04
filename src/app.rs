use crate::ansi::StyledLine;
use crate::stats::Stats;
use crate::triggers;
use egui::{FontId, ScrollArea, TextStyle};
use libmudtelnet::events::TelnetEvents;
use std::io::BufRead;
use std::sync::mpsc::{Receiver, Sender};

pub struct BatApp {
    pub lines: Vec<StyledLine>,
    pub input: String,
    pub send_input: bool,
    pub stats: Stats,
    pub event_receiver: Receiver<TelnetEvents>,
    pub command_sender: Sender<String>,
}

impl BatApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        event_receiver: Receiver<TelnetEvents>,
        command_sender: Sender<String>,
    ) -> Self {
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
            stats: Default::default(),
            event_receiver,
            command_sender,
        }
    }

    fn handle_event(&mut self, event: &TelnetEvents) {
        println!("handling event: {:?}", event);
        match event {
            TelnetEvents::IAC(iac) => {
                println!("IAC: {:?}", iac);
            }
            TelnetEvents::Negotiation(neg) => {
                println!("Negotiation: {:?}", neg);
            }
            TelnetEvents::Subnegotiation(sub_neg) => {
                println!("Subnegotiation: {:?}", sub_neg);
            }
            TelnetEvents::DataReceive(bytes) => {
                let mut lines = bytes
                    .lines()
                    .map(|l| l.unwrap_or_default())
                    .map(|line| StyledLine::new(&line))
                    // TODO: a better way than this...
                    .map(|mut styled_line| {
                        process_triggers(self, &mut styled_line);
                        styled_line
                    })
                    .collect();

                remove_gagged_lines(&mut lines);

                self.lines.append(&mut lines);
            }
            TelnetEvents::DataSend(_) => {}
            TelnetEvents::DecompressImmediate(_) => {
                println!("Decompress data");
            }
        }
    }

    fn read_input(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(&event);
        }
    }

    fn send_output(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Err(e) = self.command_sender.send(self.input.clone()) {
                eprintln!("failed to send data: {}", e);
            }
            self.input.clear();
        }
    }
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

impl eframe::App for BatApp {
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

            ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show_rows(ui, row_height, self.lines.len(), |ui, row_range| {
                    for row in row_range {
                        self.lines[row].show(ui);
                    }
                });
        });

        self.send_output(ctx);

        ctx.request_repaint();
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }
}

fn process_triggers(app: &mut BatApp, styled_line: &mut StyledLine) {
    triggers::TRIGGERS
        .iter()
        .for_each(|trigger| trigger.process(app, styled_line));
}
