use crate::message::Message;
use iced::Element;

#[derive(Default, Debug, Copy, Clone)]
pub struct Stats {
    hp: i32,
    max_hp: i32,
    sp: i32,
    max_sp: i32,
    ep: i32,
    max_ep: i32,
    exp: i32,
}

impl Stats {
    pub fn new(stats: [i32; 7]) -> Self {
        Self {
            hp: stats[0],
            max_hp: stats[1],
            sp: stats[2],
            max_sp: stats[3],
            ep: stats[4],
            max_ep: stats[5],
            exp: stats[6],
        }
    }

    pub fn hp_text_element(&self) -> Element<'_, Message> {
        iced::Element::from(iced::widget::text(format!(
            "HP: {}/{}",
            self.hp, self.max_hp
        )))
    }

    pub fn sp_text_element(&self) -> Element<'_, Message> {
        iced::Element::from(iced::widget::text(format!(
            "SP: {}/{}",
            self.sp, self.max_sp
        )))
    }

    pub fn ep_text_element(&self) -> Element<'_, Message> {
        iced::Element::from(iced::widget::text(format!(
            "EP: {}/{}",
            self.ep, self.max_ep
        )))
    }

    pub fn exp_text_element(&self) -> Element<'_, Message> {
        iced::Element::from(iced::widget::text(format!("Exp: {}", self.exp)))
    }
}
