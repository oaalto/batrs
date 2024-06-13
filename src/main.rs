use bytes::BytesMut;
use iced::widget::{self, scrollable};
use iced::{Command, Element, Font, Subscription, Theme};
use once_cell::sync::Lazy;

use crate::ansi_text::StyledLine;
use crate::message::Message;
use crate::stats::Stats;

mod ansi_codes;
mod ansi_colors;
mod ansi_text;
mod message;
mod mud;
mod stats;
mod triggers;
mod update;
mod view;

pub fn main() -> iced::Result {
    iced::program("BatMUD", BatApp::update, BatApp::view)
        .load(BatApp::load)
        .subscription(BatApp::subscription)
        .theme(|_| Theme::Dark)
        .default_font(Font::MONOSPACE)
        .run_with(init_app)
}

#[derive(Default)]
struct BatApp {
    lines: Vec<StyledLine>,
    input: String,
    state: State,
    buffer: Option<BytesMut>,
    stats: Stats,
}

fn init_app() -> BatApp {
    BatApp {
        buffer: Some(BytesMut::with_capacity(1024)),
        ..Default::default()
    }
}

impl BatApp {
    fn load() -> Command<Message> {
        widget::focus_next()
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
