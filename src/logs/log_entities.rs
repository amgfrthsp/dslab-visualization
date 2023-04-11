use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EventLog {
    pub node_cnt: usize,
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    TypeLocalMessageSent(EventLocalMessageSent),
    TypeLocalMessageReceived(EventLocalMessageReceived),
    TypeMessageSent(EventMessageSent),
    TypeMessageReceived(EventMessageReceived),
    TypeNodeUp(EventNodeUp),
    TypeNodeDown(EventNodeDown),
    TypeTimerSet(EventTimerSet),
    TypeTimerRemoved(EventTimerRemoved),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventLocalMessageSent {
    pub timestamp: f64,
    pub msg: LocalMessage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventLocalMessageReceived {
    pub timestamp: f64,
    pub msg: LocalMessage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventMessageSent {
    pub timestamp: f64,
    pub msg: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventMessageReceived {
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
pub struct EventTimerSet {
    pub timestamp: f64,
    pub timer: Timer,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventTimerRemoved {
    pub timestamp: f64,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalMessage {
    pub id: String,
    pub node_id: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timer {
    pub id: String,
    pub node_id: String,
    pub delay: f64,
}
