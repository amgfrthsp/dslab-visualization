use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EventLog {
    pub node_cnt: usize,
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    TypeSent(EventSent),
    TypeReceived(EventReceived),
    TypeNodeUp(EventNodeUp),
    TypeNodeDown(EventNodeDown),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventSent {
    pub timestamp: f64,
    pub msg: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventReceived {
    pub timestamp: f64,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventNodeUp {
    pub timestamp: f64,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventNodeDown {
    pub timestamp: f64,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub data: String,
}
