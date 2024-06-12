use crate::mud;

#[derive(Debug, Clone)]
pub enum Message {
    NewMessageChanged(String),
    Send(String),
    Mud(mud::Event),
}
