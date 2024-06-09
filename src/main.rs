mod mud;

use iced::alignment::{self, Alignment};
use iced::widget::{
    self, button, center, column, row, scrollable, text, text_input,
};
use iced::{color, Command, Element, Font, Length, Subscription, Theme};
use once_cell::sync::Lazy;

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
    messages: Vec<mud::Message>,
    new_message: String,
    state: State,
}

#[derive(Debug, Clone)]
enum Message {
    NewMessageChanged(String),
    Send(mud::Message),
    Mud(mud::Event),
    Server,
}

fn init_app() -> BatRs {
    BatRs {
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

                    self.messages.push(mud::Message::connected());

                    Command::none()
                }
                mud::Event::Disconnected => {
                    self.state = State::Disconnected;

                    self.messages.push(mud::Message::disconnected());

                    Command::none()
                }
                mud::Event::MessageReceived(message) => {
                    self.messages.push(message);

                    scrollable::snap_to(
                        MESSAGE_LOG.clone(),
                        scrollable::RelativeOffset::END,
                    )
                }
            },
            Message::Server => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        mud::connect().map(Message::Mud)
    }

    fn view(&self) -> Element<Message> {
        let message_log: Element<_> = if self.messages.is_empty() {
            center(
                text("Your messages will appear here...")
                    .color(color!(0x888888)),
            )
            .into()
        } else {
            scrollable(
                column(self.messages.iter().map(text).map(Element::from))
            )
            .id(MESSAGE_LOG.clone())
            .height(Length::Fill)
            .into()
        };

        let new_message_input = {
            let mut input = text_input("Type a message...", &self.new_message)
                .on_input(Message::NewMessageChanged)
                .padding(10);

            let mut button = button(
                text("Send")
                    .height(40)
                    .vertical_alignment(alignment::Vertical::Center),
            )
            .padding([0, 20]);

            if matches!(self.state, State::Connected(_)) {
                if let Some(message) = mud::Message::new(&self.new_message) {
                    input = input.on_submit(Message::Send(message.clone()));
                    button = button.on_press(Message::Send(message));
                }
            }

            row![input, button]
                .spacing(10)
                .align_items(Alignment::Center)
        };

        column![message_log, new_message_input]
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
