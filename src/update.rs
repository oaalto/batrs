use crate::ansi_text::StyledLine;
use crate::message::Message;
use crate::{mud, triggers, BatApp, State, SCROLLABLE_ID};
use bytes::{BufMut, BytesMut};
use iced::widget::scrollable;
use iced::Command;
use std::io::BufRead;

pub fn update(app: &mut BatApp, message: Message) -> Command<Message> {
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
                let buffer = app.buffer.replace(BytesMut::with_capacity(1024)).unwrap();

                let mut lines = buffer
                    .lines()
                    .map(|l| l.unwrap_or_default())
                    .map(|line| StyledLine::new(&line))
                    .map(|mut styled_line| {
                        process_triggers(app, &mut styled_line);
                        styled_line
                    })
                    .collect();

                app.lines.append(&mut lines);

                scrollable::snap_to(SCROLLABLE_ID.clone(), scrollable::RelativeOffset::END)
            }
            mud::Event::DataReceived(data) => {
                if let Some(buffer) = &mut app.buffer {
                    buffer.put(data);
                }

                Command::none()
            }
        },
    }
}

fn process_triggers(app: &mut BatApp, styled_line: &mut StyledLine) {
    triggers::TRIGGERS
        .iter()
        .for_each(|trigger| trigger.process(app, styled_line));
}
