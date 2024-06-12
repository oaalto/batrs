use crate::message::Message;
use crate::{BatRs, SCROLLABLE_ID};
use iced::widget::{column, row, scrollable, text_input};
use iced::{Alignment, Element, Length};

pub fn view(app: &BatRs) -> Element<Message> {
    let lines = scrollable(column(
        app.lines
            .iter()
            .map(|line| line.to_row())
            .map(Element::from),
    ))
    .id(SCROLLABLE_ID.clone())
    .height(Length::Fill)
    .width(Length::Fill);

    let mut input = text_input("", &app.new_message)
        .on_input(Message::NewMessageChanged)
        .padding(10);

    if app.is_connected() && !app.new_message.is_empty() {
        input = input.on_submit(Message::Send(app.new_message.clone()));
    }

    let new_message_input = row![input]
        .align_items(Alignment::Center)
        .width(Length::Fill);

    iced::widget::column![lines, new_message_input]
        .height(Length::Fill)
        .padding(20)
        .spacing(10)
        .into()
}
