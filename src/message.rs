use crate::mud;
use crate::stats::Stats;

#[derive(Debug, Clone)]
pub enum Message {
    NewMessageChanged(String),
    Send(String),
    Mud(mud::Event),
    UpdateStats(Stats),
}
