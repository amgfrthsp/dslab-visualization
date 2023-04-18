use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EventLog {
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    TypeNodeAdded(EventNodeAdded),
    TypeLocalMessageSent(EventLocalMessageSent),
    TypeLocalMessageReceived(EventLocalMessageReceived),
    TypeMessageSent(EventMessageSent),
    TypeMessageReceived(EventMessageReceived),
    TypeNodeConnected(EventNodeConnected),
    TypeNodeDisconnected(EventNodeDisconnected),
    TypeTimerSet(EventTimerSet),
    TypeTimerRemoved(EventTimerRemoved),

    TypeLinkDisabled(EventLinkDisabled),
    TypeLinkEnabled(EventLinkEnabled),
    TypeDropIncoming(EventDropIncoming),
    TypePassIncoming(EventPassIncoming),
    TypeDropOutgoing(EventDropOutgoing),
    TypePassOutgoing(EventPassOutgoing),
    TypeMakePartition(EventMakePartition),
    TypeResetNetwork(EventResetNetwork),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventNodeAdded {
    pub timestamp: f64,
    pub id: String,
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
pub struct EventNodeConnected {
    pub timestamp: f64,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventNodeDisconnected {
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
pub struct EventLinkDisabled {
    pub timestamp: f64,
    pub from: String,
    pub to: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventLinkEnabled {
    pub timestamp: f64,
    pub from: String,
    pub to: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventDropIncoming {
    pub timestamp: f64,
    pub node_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventPassIncoming {
    pub timestamp: f64,
    pub node_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventDropOutgoing {
    pub timestamp: f64,
    pub node_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventPassOutgoing {
    pub timestamp: f64,
    pub node_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventMakePartition {
    pub timestamp: f64,
    pub group1: Vec<String>,
    pub group2: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventResetNetwork {
    pub timestamp: f64,
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
    pub src: String,
    pub dest: String,
    pub tip: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timer {
    pub id: String,
    pub node_id: String,
    pub delay: f64,
}
