use iced::widget::{
    column, row, scrollable, text, text_input, Column, Rule, Scrollable, TextInput,
};
use iced::{Element, Length};

use crate::message::Message;
use crate::{BatApp, SCROLLABLE_ID};

pub fn view(app: &BatApp) -> Element<Message> {
    let lines = view_lines(app);

    let input = view_input(app);

    let stats = view_stats(app);

    let column = column![lines, input]
        .height(Length::Fill)
        .width(Length::FillPortion(6))
        .padding(20)
        .spacing(10);

    let divider = Rule::vertical(1);

    row![column, divider, stats].into()
}

fn view_stats(app: &BatApp) -> Column<Message> {
    column![
        app.stats.hp_text_element(),
        app.stats.sp_text_element(),
        app.stats.ep_text_element(),
    ]
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .padding(10)
}

fn view_input(app: &BatApp) -> TextInput<Message> {
    let mut input = text_input("", &app.input)
        .on_input(Message::NewMessageChanged)
        .padding(10);

    if !app.input.is_empty() {
        input = input.on_submit(Message::Send(app.input.clone()));
    }

    input
}

fn view_lines(app: &BatApp) -> Scrollable<Message> {
    scrollable(column(
        app.lines
            .iter()
            .map(|line| line.to_row())
            .map(Element::from),
    ))
    .id(SCROLLABLE_ID.clone())
    .height(Length::Fill)
}
