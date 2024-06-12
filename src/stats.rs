use crate::ansi_colors;
use crate::message::Message;
use iced::{color, Element};

#[derive(Default, Debug, Copy, Clone)]
pub struct Stats {
    hp: i32,
    max_hp: i32,
    sp: i32,
    max_sp: i32,
    ep: i32,
    max_ep: i32,
}

impl Stats {
    pub fn hp_text_element(&self) -> Element<'_, Message> {
        // let color = ansi_colors::get_color(&self.color, self.bold);
        // iced::Element::from(iced::widget::text(&self.text).color(color))
        iced::Element::from(iced::widget::text(format!(
            "HP: {}/{}",
            self.hp, self.max_hp
        )))
    }

    pub fn sp_text_element(&self) -> Element<'_, Message> {
        // let color = ansi_colors::get_color(&self.color, self.bold);
        // iced::Element::from(iced::widget::text(&self.text).color(color))
        iced::Element::from(iced::widget::text(format!(
            "SP: {}/{}",
            self.sp, self.max_sp
        )))
    }

    pub fn ep_text_element(&self) -> Element<'_, Message> {
        // let color = ansi_colors::get_color(&self.color, self.bold);
        // iced::Element::from(iced::widget::text(&self.text).color(color))
        iced::Element::from(iced::widget::text(format!(
            "EP: {}/{}",
            self.ep, self.max_ep
        )))
    }
}
