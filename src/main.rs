#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::app::{AppEvent, BatApp};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::future;
use futures::stream::StreamExt;
use libmudtelnet::Parser;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_util::codec::{BytesCodec, FramedRead};

mod abilities;
mod ansi;
mod app;
mod automation;
mod command;
mod config;
mod generic_commands;
mod guilds;
mod stats;
mod triggers;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let (event_receiver, command_sender) = setup_connection();

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let app = BatApp::new(event_receiver, command_sender);
    let result = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result?;
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: BatApp,
) -> std::io::Result<()> {
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| app.draw(frame))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_millis(0));

        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key_event(key);
        }

        app.read_input();

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_quit() {
            break;
        }
    }

    Ok(())
}

fn setup_connection() -> (Receiver<AppEvent>, Sender<String>) {
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
                        eprintln!("failed to read from socket; error={e}");
                        future::ready(None)
                    }
                });

                stream
                    .for_each(|data| {
                        let sender = event_sender.clone();

                        async move {
                            sender
                                .send(AppEvent::RawInput(data.to_vec()))
                                .expect("failed to send raw input to channel");
                            let mut parser = Parser::new();
                            let events = parser.receive(&data);
                            for event in events {
                                sender
                                    .send(AppEvent::Telnet(event))
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
                        eprintln!("{e}");
                    }
                }
            }
        })
    });

    (event_receiver, command_sender)
}
