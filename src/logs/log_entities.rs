use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EventLog {
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    NodeAdded {
        time: f64,
        node_name: String,
        node_id: u32,
    },
    LocalMessageSent {
        time: f64,
        msg_id: String,
        node_name: String,
        proc: String,
        msg_tip: String,
        msg_data: String,
    },
    LocalMessageReceived {
        time: f64,
        msg_id: String,
        node_name: String,
        proc: String,
        msg_tip: String,
        msg_data: String,
    },
    MessageSent {
        time: f64,
        msg_id: String,
        src_node: String,
        src_proc: String,
        dest_node: String,
        dest_proc: String,
        msg_tip: String,
        msg_data: String,
    },
    MessageReceived {
        time: f64,
        msg_id: String,
    },
    NodeConnected {
        time: f64,
        node_name: String,
    },
    NodeDisconnected {
        time: f64,
        node_name: String,
    },
    TimerSet {
        time: f64,
        timer_id: String,
        node_name: String,
        delay: f64,
    },
    TimerRemoved {
        time: f64,
        timer_id: String,
    },
    LinkDisabled {
        time: f64,
        from: String,
        to: String,
    },
    LinkEnabled {
        time: f64,
        from: String,
        to: String,
    },
    DropIncoming {
        time: f64,
        node_name: String,
    },
    PassIncoming {
        time: f64,
        node_name: String,
    },
    DropOutgoing {
        time: f64,
        node_name: String,
    },
    PassOutgoing {
        time: f64,
        node_name: String,
    },
    MakePartition {
        time: f64,
        group1: Vec<String>,
        group2: Vec<String>,
    },
    ResetNetwork {
        time: f64,
    },
}
