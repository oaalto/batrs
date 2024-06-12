use bytes::Bytes;
use std::net::SocketAddr;

use futures::channel::mpsc;
use futures::future;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use iced::futures;
use iced::subscription::{self, Subscription};
use libmudtelnet::events::TelnetEvents;
use libmudtelnet::Parser;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, FramedRead};

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

            let _ = output.send(Event::Connected(Connection(sender))).await;

            tokio::spawn(async move {
                let stream = FramedRead::new(reader, BytesCodec::new()).filter_map(|i| match i {
                    //BytesMut into Bytes
                    Ok(i) => future::ready(Some(i.freeze())),
                    Err(e) => {
                        eprintln!("failed to read from socket; error={}", e);
                        future::ready(None)
                    }
                });

                stream
                    .for_each(|data| {
                        let mut out = output.clone();

                        async move {
                            let mut parser = Parser::new();
                            let events = parser.receive(&data);
                            for event in events {
                                log_event(&event, &mut out).await;
                            }
                        }
                    })
                    .await;
            });

            let mut parser = Parser::new();
            loop {
                let message = receiver.select_next_some().await;
                let events = parser.send_text(&message);
                let res = writer.write_all(&events.to_bytes()).await;
                if let Err(e) = res {
                    eprintln!("{}", e);
                }
            }
        },
    )
}

async fn log_event(event: &TelnetEvents, out: &mut mpsc::Sender<Event>) {
    match event {
        TelnetEvents::IAC(iac) => {
            println!("IAC Command: {}", iac.command);
            if 249 == iac.command {
                let _ = out.send(Event::CommandGoAhead).await;
            }
        }
        TelnetEvents::Negotiation(neg) => {
            println!("Negotiation: {:?}", neg);
        }
        TelnetEvents::Subnegotiation(sub_neg) => {
            println!("Subnegotiation: {:?}", sub_neg);
        }
        TelnetEvents::DataReceive(bytes) => {
            let _ = out.send(Event::DataReceived(bytes.clone())).await;
        }
        TelnetEvents::DataSend(_) => {}
        TelnetEvents::DecompressImmediate(_) => {
            println!("Decompress data");
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected(Connection),
    CommandGoAhead,
    DataReceived(Bytes),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<String>);

impl Connection {
    pub fn send(&mut self, data: String) {
        self.0.try_send(data).expect("Send message to echo server");
    }
}
