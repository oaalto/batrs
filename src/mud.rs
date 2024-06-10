use iced::futures;
use iced::subscription::{self, Subscription};
use iced::widget::text;

use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use std::fmt;
use std::io::BufRead;
use std::net::SocketAddr;
use futures::future;
use libmudtelnet::events::TelnetEvents;
use libmudtelnet::Parser;
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio::io::AsyncWriteExt;

pub fn connect() -> Subscription<Event> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let addr = "95.175.124.84:23".parse::<SocketAddr>().unwrap();

            let stream = TcpStream::connect(addr).await.unwrap();
            let (reader, mut writer) = stream.into_split();

            // Send the sender back to the application
            let (sender, mut receiver) = mpsc::channel(100);

            let _ = output
                .send(Event::Connected(Connection(sender)))
                .await;

            tokio::spawn(
                async move {
                    let stream = FramedRead::new(reader, BytesCodec::new())
                        .filter_map(|i| match i {
                            //BytesMut into Bytes
                            Ok(i) => future::ready(Some(i.freeze())),
                            Err(e) => {
                                println!("failed to read from socket; error={}", e);
                                future::ready(None)
                            }
                        });

                    stream.for_each(|data| {
                        let mut out = output.clone();

                        async move {
                            let mut parser = Parser::new();
                            let events = parser.receive(&data);
                            for event in events {
                                if let TelnetEvents::DataReceive(data) = event {
                                    let lines: Vec<String> = data.lines()
                                        .map(|l| l.unwrap_or(String::new())).collect();
                                    for line in lines {
                                        let _ = out.send(Event::MessageReceived(Message::Telnet(line))).await;
                                    }
                                }
                            }
                        }
                    }).await;
                });

            let mut parser = Parser::new();
            loop {
                let message = receiver.select_next_some().await;
                println!("message: {}", &message.to_string());
                let events = parser.send_text(&message.to_string());
                let res = writer.write_all(&events.to_bytes()).await;
                if let Err(e) = res {
                    eprintln!("{}", e.to_string());
                }
            }
        },
    )
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected(Connection),
    Disconnected,
    MessageReceived(Message),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<Message>);

impl Connection {
    pub fn send(&mut self, message: Message) {
        self.0
            .try_send(message)
            .expect("Send message to echo server");
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Connected,
    Disconnected,
    User(String),
    Telnet(String),
}

impl Message {
    pub fn new(message: &str) -> Option<Self> {
        if message.is_empty() {
            None
        } else {
            Some(Message::User(message.to_string()))
        }
    }

    pub fn connected() -> Self {
        Message::Connected
    }

    pub fn disconnected() -> Self {
        Message::Disconnected
    }

    pub fn as_str(&self) -> &str {
        match self {
            Message::Connected => "Connected successfully!",
            Message::Disconnected => "Connection lost... Retrying...",
            Message::User(message) => message.as_str(),
            Message::Telnet(message) => message.as_str(),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_str())
    }
}

impl<'a> text::IntoFragment<'a> for &'a Message {
    fn into_fragment(self) -> text::Fragment<'a> {
        text::Fragment::Borrowed(self.as_str())
    }
}
