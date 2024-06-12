use crate::ansi_text::StyledLine;
use bytes::{BufMut, BytesMut};
use iced::alignment::Alignment;
use iced::widget::{self, column, row, scrollable, text_input};
use iced::{Command, Element, Font, Length, Subscription, Theme};
use once_cell::sync::Lazy;
use std::io::BufRead;

mod ansi_codes;
mod ansi_colors;
mod ansi_text;
mod mud;

pub fn main() -> iced::Result {
    iced::program("BatMUD", BatRs::update, BatRs::view)
        .load(BatRs::load)
        .subscription(BatRs::subscription)
        .theme(|_| Theme::Dark)
        .default_font(Font::MONOSPACE)
        .run_with(init_app)
}

#[derive(Default)]
struct BatRs {
    lines: Vec<StyledLine>,
    new_message: String,
    state: State,
    buffer: BytesMut,
}

#[derive(Debug, Clone)]
enum Message {
    NewMessageChanged(String),
    Send(String),
    Mud(mud::Event),
}

fn init_app() -> BatRs {
    BatRs {
        buffer: BytesMut::with_capacity(1024),
        ..Default::default()
    }
}

impl BatRs {
    fn load() -> Command<Message> {
        widget::focus_next()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NewMessageChanged(new_message) => {
                self.new_message = new_message;

                Command::none()
            }
            Message::Send(message) => match &mut self.state {
                State::Connected(connection) => {
                    self.new_message.clear();

                    connection.send(message);

                    Command::none()
                }
                State::Disconnected => Command::none(),
            },
            Message::Mud(event) => match event {
                mud::Event::Connected(connection) => {
                    self.state = State::Connected(connection);

                    Command::none()
                }
                mud::Event::CommandGoAhead => {
                    let lines: Vec<String> =
                        self.buffer.lines().map(|l| l.unwrap_or_default()).collect();
                    lines
                        .iter()
                        .for_each(|line| self.lines.push(StyledLine::new(line)));

                    self.buffer.clear();
                    scrollable::snap_to(MESSAGE_LOG.clone(), scrollable::RelativeOffset::END)
                }
                mud::Event::DataReceived(data) => {
                    self.buffer.put(data);

                    Command::none()
                }
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        mud::connect().map(Message::Mud)
    }

    fn view(&self) -> Element<Message> {
        let lines = scrollable(column(
            self.lines
                .iter()
                .map(|line| line.to_row())
                .map(Element::from),
        ))
        .id(MESSAGE_LOG.clone())
        .height(Length::Fill)
        .width(Length::Fill);

        let mut input = text_input("", &self.new_message)
            .on_input(Message::NewMessageChanged)
            .padding(10);

        if matches!(self.state, State::Connected(_)) && !self.new_message.is_empty() {
            input = input.on_submit(Message::Send(self.new_message.clone()));
        }

        let new_message_input = row![input]
            .align_items(Alignment::Center)
            .width(Length::Fill);

        column![lines, new_message_input]
            .height(Length::Fill)
            .padding(20)
            .spacing(10)
            .into()
    }
}

enum State {
    Disconnected,
    Connected(mud::Connection),
}

impl Default for State {
    fn default() -> Self {
        Self::Disconnected
    }
}

static MESSAGE_LOG: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
