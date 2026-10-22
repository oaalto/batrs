#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::app::{
    AppEvent, BatApp, ConnectionChannels, ConnectionCoordinator, ConnectionId,
    INITIAL_CONNECTION_ID, ReconnectResult,
};
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste, Event, EventStream};

const CLOCK_REDRAW_INTERVAL: Duration = Duration::from_secs(1);
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::stream::StreamExt;
use libmudtelnet::Parser;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::sync::{Arc, mpsc};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::runtime::{Handle, Runtime};
use tokio::sync::Notify;
use tokio::time::Instant;
use tokio_util::codec::{BytesCodec, FramedRead};

mod abilities;
mod ansi;
mod app;
mod automation;
mod combat_awareness;
mod combat_damage;
mod command;
mod config;
mod generic_commands;
mod guilds;
mod player_profile;
mod secondary_status;
mod stats;
mod triggers;
mod ui;

fn main() {
    if let Err(error) = run() {
        report_error(error.as_ref());
    }
}

/// Print a fatal client error without debug wrappers and exit unsuccessfully.
fn report_error(error: &dyn std::error::Error) -> ! {
    eprintln!("{error}");
    std::process::exit(1);
}

/// Run the client, restoring the terminal before propagating application errors.
fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let runtime = Runtime::new()?;
    let network_ready = Arc::new(Notify::new());
    let address = std::net::SocketAddr::from(([95, 175, 124, 84], 23));
    let channels = runtime
        .block_on(setup_connection(
            INITIAL_CONNECTION_ID,
            address,
            network_ready.clone(),
        ))
        .map_err(std::io::Error::other)?;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let viewer_port = combat_damage::parse_port_from_args(std::env::args());
    let app = BatApp::new(
        channels.event_receiver,
        channels.command_sender,
        Box::new(TcpConnectionCoordinator {
            runtime: runtime.handle().clone(),
            address,
            network_ready: network_ready.clone(),
        }),
        viewer_port,
    );
    let result = runtime.block_on(run_app(&mut terminal, app, &network_ready));

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableBracketedPaste,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    result?;
    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: BatApp,
    network_ready: &Notify,
) -> std::io::Result<()> {
    let mut events = EventStream::new();
    let mut last_redraw_tick = Instant::now();

    loop {
        if app.take_pending_terminal_clear() {
            terminal.clear()?;
        }
        if app.take_redraw() {
            terminal.draw(|frame| app.draw(frame))?;
        }

        if let Some(event) = wait_for_input(
            &mut events,
            network_ready,
            last_redraw_tick + CLOCK_REDRAW_INTERVAL,
        )
        .await?
        {
            handle_terminal_event(event, &mut app);
        }

        app.read_input();

        let now = Instant::now();
        if now.duration_since(last_redraw_tick) >= CLOCK_REDRAW_INTERVAL {
            last_redraw_tick = now;
            app.request_redraw();
        }

        if app.should_quit() {
            break;
        }
    }

    Ok(())
}

/// Wait for terminal input, queued game output, or the clock redraw deadline.
/// Terminal read failures propagate to the caller.
async fn wait_for_input(
    events: &mut (impl futures::Stream<Item = std::io::Result<Event>> + Unpin),
    network_ready: &Notify,
    deadline: Instant,
) -> std::io::Result<Option<Event>> {
    tokio::select! {
        event = events.next() => event.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "terminal event stream ended")
        })?.map(Some),
        () = network_ready.notified() => Ok(None),
        () = tokio::time::sleep_until(deadline) => Ok(None),
    }
}

fn handle_terminal_event(event: Event, app: &mut BatApp) {
    match event {
        Event::Key(key) => app.handle_key_event(key),
        Event::Paste(text) => app.handle_paste_event(text),
        Event::Resize(_, _) => app.request_redraw(),
        _ => {}
    }
}

struct TcpConnectionCoordinator {
    runtime: Handle,
    address: std::net::SocketAddr,
    network_ready: Arc<Notify>,
}

impl ConnectionCoordinator for TcpConnectionCoordinator {
    /// Start socket setup on the existing runtime; completion wakes the TUI.
    fn reconnect(&mut self, connection_id: ConnectionId) -> ReconnectResult {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let address = self.address;
        let network_ready = self.network_ready.clone();
        self.runtime.spawn(async move {
            let result = setup_connection(connection_id, address, network_ready.clone()).await;
            if sender.send(result).is_ok() {
                network_ready.notify_one();
            }
        });
        ReconnectResult::Pending(receiver)
    }
}

/// Connect and start cancellable reader/writer tasks on the caller's runtime.
/// Socket setup errors include the connection operation's context.
async fn setup_connection(
    connection_id: ConnectionId,
    address: std::net::SocketAddr,
    network_ready: Arc<Notify>,
) -> Result<ConnectionChannels, String> {
    let stream = TcpStream::connect(address)
        .await
        .map_err(|e| format!("unable to connect to BatMUD: {e}"))?;
    let (event_sender, event_receiver) = mpsc::channel();
    let (command_sender, mut command_receiver) = tokio::sync::mpsc::unbounded_channel::<String>();

    tokio::spawn(async move {
        let (reader, mut writer) = stream.into_split();
        let read = async {
            let mut stream = FramedRead::new(reader, BytesCodec::new());
            let mut parser = Parser::new();
            while let Some(data) = stream.next().await {
                let data = match data {
                    Ok(data) => data,
                    Err(error) => {
                        log::error!("failed to read from socket; error={error}");
                        break;
                    }
                };
                if event_sender
                    .send(AppEvent::RawInput {
                        connection_id,
                        bytes: data.to_vec(),
                    })
                    .is_err()
                {
                    break;
                }
                for event in parser.receive(&data) {
                    if event_sender
                        .send(AppEvent::Telnet {
                            connection_id,
                            event,
                        })
                        .is_err()
                    {
                        return;
                    }
                }
                network_ready.notify_one();
            }
        };
        let write = async {
            let mut parser = Parser::new();
            while let Some(message) = command_receiver.recv().await {
                let events = parser.send_text(&message);
                if let Err(error) = writer.write_all(&events.to_bytes()).await {
                    log::error!("failed to write telnet command to socket; error={error}");
                    break;
                }
            }
        };
        tokio::select! {
            () = read => {},
            () = write => {},
        }
    });

    Ok(ConnectionChannels {
        event_receiver,
        command_sender,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fatal_error_process_fixture() {
        if std::env::var_os("BATRS_TEST_FATAL_ERROR").is_some() {
            report_error(&std::io::Error::other(
                "unable to connect to BatMUD: No route to host (os error 113)",
            ));
        }
    }

    #[test]
    fn fatal_error_prints_only_message_and_exits_unsuccessfully() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tests::fatal_error_process_fixture",
                "--nocapture",
            ])
            .env("BATRS_TEST_FATAL_ERROR", "1")
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "unable to connect to BatMUD: No route to host (os error 113)\n"
        );
    }

    #[tokio::test]
    async fn reconnect_inside_runtime_returns_pending_and_connects_locally() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ready = Arc::new(Notify::new());
        let mut coordinator = TcpConnectionCoordinator {
            runtime: tokio::runtime::Handle::current(),
            address: listener.local_addr().unwrap(),
            network_ready: ready.clone(),
        };
        let started = std::time::Instant::now();
        let ReconnectResult::Pending(result) = coordinator.reconnect(1) else {
            panic!("reconnect must return pending without blocking the runtime");
        };
        assert!(started.elapsed() < Duration::from_millis(100));
        let channels = tokio::time::timeout(Duration::from_secs(1), result)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let (mut peer, _) = listener.accept().await.unwrap();
        channels.command_sender.send("look".into()).unwrap();
        use tokio::io::AsyncReadExt;
        let mut command = [0; 4];
        tokio::time::timeout(Duration::from_secs(1), peer.read_exact(&mut command))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(&command, b"look");
        peer.write_all(b"hello\r\n").await.unwrap();
        tokio::time::timeout(Duration::from_secs(1), ready.notified())
            .await
            .unwrap();
        let receiver = channels.event_receiver;
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if let Ok(AppEvent::RawInput {
                    connection_id,
                    bytes,
                }) = receiver.try_recv()
                {
                    assert_eq!(connection_id, 1);
                    assert_eq!(bytes, b"hello\r\n");
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn reconnect_failure_wakes_client_and_reports_connection_error() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let ready = Arc::new(Notify::new());
        let mut coordinator = TcpConnectionCoordinator {
            runtime: Handle::current(),
            address,
            network_ready: ready.clone(),
        };
        let ReconnectResult::Pending(result) = coordinator.reconnect(1) else {
            panic!("reconnect must return pending");
        };
        tokio::time::timeout(Duration::from_secs(1), ready.notified())
            .await
            .unwrap();
        let Err(error) = result.await.unwrap() else {
            panic!("closed port must fail")
        };
        assert!(error.contains("unable to connect to BatMUD"));
    }

    #[tokio::test]
    async fn game_output_wakes_wait_without_keyboard_input_or_clock_tick() {
        let network_ready = Arc::new(Notify::new());
        let sender_ready = network_ready.clone();
        let (sender, receiver) = mpsc::channel();
        let producer = tokio::spawn(async move {
            tokio::task::yield_now().await;
            sender.send("response").unwrap();
            sender_ready.notify_one();
        });
        let mut events = futures::stream::pending();
        let result = tokio::time::timeout(
            Duration::from_millis(200),
            wait_for_input(
                &mut events,
                &network_ready,
                Instant::now() + CLOCK_REDRAW_INTERVAL,
            ),
        )
        .await
        .expect("game output must not wait for the one-second clock tick")
        .unwrap();
        assert_eq!(result, None);
        assert_eq!(receiver.try_recv(), Ok("response"));
        producer.await.unwrap();
    }

    #[tokio::test]
    async fn game_output_queued_before_wait_is_not_lost() {
        let network_ready = Notify::new();
        network_ready.notify_one();
        let mut events = futures::stream::pending();
        assert_eq!(
            tokio::time::timeout(
                Duration::from_millis(200),
                wait_for_input(
                    &mut events,
                    &network_ready,
                    Instant::now() + CLOCK_REDRAW_INTERVAL
                ),
            )
            .await
            .unwrap()
            .unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn terminal_input_and_clock_deadline_still_wake_wait() {
        let network_ready = Notify::new();
        let mut events = futures::stream::iter([Ok(Event::Resize(80, 24))]);
        assert_eq!(
            wait_for_input(
                &mut events,
                &network_ready,
                Instant::now() + CLOCK_REDRAW_INTERVAL
            )
            .await
            .unwrap(),
            Some(Event::Resize(80, 24))
        );
        let mut events = futures::stream::pending();
        assert_eq!(
            wait_for_input(&mut events, &network_ready, Instant::now())
                .await
                .unwrap(),
            None
        );
    }
}
