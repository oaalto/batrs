use crate::ansi_text::StyledLine;
use crate::message::Message;
use crate::{mud, BatRs, State, SCROLLABLE_ID};
use bytes::BufMut;
use iced::widget::scrollable;
use iced::Command;
use std::io::BufRead;

pub fn update(app: &mut BatRs, message: Message) -> Command<Message> {
    match message {
        Message::NewMessageChanged(new_message) => {
            app.input = new_message;

            Command::none()
        }
        Message::Send(message) => match &mut app.state {
            State::Connected(connection) => {
                app.input.clear();

                connection.send(message);

                Command::none()
            }
            State::Disconnected => Command::none(),
        },
        Message::Mud(event) => match event {
            mud::Event::Connected(connection) => {
                app.state = State::Connected(connection);

                Command::none()
            }
            mud::Event::CommandGoAhead => {
                let lines: Vec<String> =
                    app.buffer.lines().map(|l| l.unwrap_or_default()).collect();
                lines
                    .iter()
                    .for_each(|line| app.lines.push(StyledLine::new(line)));

                app.buffer.clear();
                scrollable::snap_to(SCROLLABLE_ID.clone(), scrollable::RelativeOffset::END)
            }
            mud::Event::DataReceived(data) => {
                app.buffer.put(data);

                Command::none()
            }
        },
    }
}
