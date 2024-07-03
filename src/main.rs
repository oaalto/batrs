#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::app::BatApp;
use std::net::TcpStream;

mod ansi;
mod app;
mod stats;
mod triggers;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let stream = TcpStream::connect("95.175.124.84:23").expect("connection to succeed");
    /* stream
            .set_nonblocking(true)
            .expect("set_nonblocking call failed");
    */
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(true),
        ..Default::default()
    };
    eframe::run_native(
        "BatMUD Client",
        native_options,
        Box::new(|cc| Box::new(BatApp::new(cc, stream))),
    )
}
