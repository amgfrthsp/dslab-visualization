use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum LogEntry {
    NodeStarted {
        time: f64,
        node: String,
        node_id: u32,
    },
    ProcessStarted {
        time: f64,
        node: String,
        proc: String,
    },
    LocalMessageSent {
        time: f64,
        msg_id: String,
        node: String,
        proc: String,
        msg_tip: String,
        msg_data: String,
    },
    LocalMessageReceived {
        time: f64,
        msg_id: String,
        node: String,
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
    MessageDropped {
        time: f64,
        msg_id: String,
    },
    NodeConnected {
        time: f64,
        node: String,
    },
    NodeDisconnected {
        time: f64,
        node: String,
    },
    NodeCrashed {
        time: f64,
        node: String,
    },
    NodeRecovered {
        time: f64,
        node: String,
    },
    TimerSet {
        time: f64,
        timer_id: String,
        timer_name: String,
        node: String,
        proc: String,
        delay: f64,
    },
    TimerFired {
        time: f64,
        timer_id: String,
    },
    TimerCancelled {
        time: f64,
        timer_id: String,
    },
    LinkDisabled {
        time: f64,
        from_node: String,
        from_proc: String,
        to_node: String,
        to_proc: String,
    },
    LinkEnabled {
        time: f64,
        from_node: String,
        from_proc: String,
        to_node: String,
        to_proc: String,
    },
    DropIncoming {
        time: f64,
        node: String,
    },
    PassIncoming {
        time: f64,
        node: String,
    },
    DropOutgoing {
        time: f64,
        node: String,
    },
    PassOutgoing {
        time: f64,
        node: String,
    },
    MakePartition {
        time: f64,
        group1_nodes: Vec<String>,
        group1_procs: Vec<String>,
        group2_nodes: Vec<String>,
        group2_procs: Vec<String>,
    },
    ResetNetwork {
        time: f64,
    },
}
