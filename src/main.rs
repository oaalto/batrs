use crate::ansi_text::StyledLine;
use crate::message::Message;
use crate::update::update;
use crate::view::view;
use bytes::{BufMut, BytesMut};
use iced::alignment::Alignment;
use iced::widget::{self, column, row, scrollable, text_input};
use iced::{Command, Element, Font, Length, Subscription, Theme};
use once_cell::sync::Lazy;
use std::io::BufRead;

mod ansi_codes;
mod ansi_colors;
mod ansi_text;
mod message;
mod mud;
mod update;
mod view;

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

    pub fn is_connected(&self) -> bool {
        matches!(self.state, State::Connected(_))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        update::update(self, message)
    }

    fn subscription(&self) -> Subscription<Message> {
        mud::connect().map(Message::Mud)
    }

    fn view(&self) -> Element<Message> {
        view::view(self)
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

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
