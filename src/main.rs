#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::app::BatApp;
use futures::future;
use futures::stream::StreamExt;
use libmudtelnet::events::TelnetEvents;
use libmudtelnet::Parser;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_util::codec::{BytesCodec, FramedRead};

mod ansi;
mod app;
mod command;
mod stats;
mod triggers;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let (event_receiver, command_sender) = setup_connection();

    eframe::run_native(
        "BatMUD Client",
        Default::default(),
        Box::new(|cc| Box::new(BatApp::new(cc, event_receiver, command_sender))),
    )
}

fn setup_connection() -> (Receiver<TelnetEvents>, Sender<String>) {
    let rt = Runtime::new().expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    let (event_sender, event_receiver) = mpsc::channel();
    let (command_sender, command_receiver) = mpsc::channel::<String>();

    // Execute the runtime in its own thread.
    std::thread::spawn(move || {
        rt.block_on(async {
            let stream = TcpStream::connect("95.175.124.84:23")
                .await
                .expect("connection to succeed");
            let (reader, mut writer) = stream.into_split();

            tokio::spawn(async move {
                let stream = FramedRead::new(reader, BytesCodec::new()).filter_map(|i| match i {
                    Ok(i) => future::ready(Some(i.freeze())),
                    Err(e) => {
                        eprintln!("failed to read from socket; error={}", e);
                        future::ready(None)
                    }
                });

                stream
                    .for_each(|data| {
                        let sender = event_sender.clone();

                        async move {
                            let mut parser = Parser::new();
                            let events = parser.receive(&data);
                            for event in events {
                                sender
                                    .send(event)
                                    .expect("failed to send telnet event to channel");
                            }
                        }
                    })
                    .await;
            });

            let mut parser = Parser::new();
            loop {
                if let Ok(message) = command_receiver.recv() {
                    let events = parser.send_text(&message);
                    if let Err(e) = writer.write_all(&events.to_bytes()).await {
                        eprintln!("{}", e);
                    }
                }
            }
        })
    });

    (event_receiver, command_sender)
}
