use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{collections::HashMap, f32::consts::PI};

use crate::logs::log_entities::*;

use super::state::state::State;
use super::utilities::{prettify_json_string, CIRCLE_RADIUS};

#[derive(Debug)]
pub enum ControllerStateCommand {
    MessageSent(String),
    LocalMessageEmerged(String),
    NodeConnected(String),
    NodeDisconnected(String),
    NodeStarted(ControllerNode),
    TimerSet(String),
    LinkDisabled((String, String)),
    LinkEnabled((String, String)),
    DropIncoming(String),
    PassIncoming(String),
    DropOutgoing(String),
    PassOutgoing(String),
    NetworkPartition((Vec<String>, Vec<String>)),
    NetworkReset(),
    NodeStateUpdated((String, String)),
}

pub struct EventController {
    local_messages: HashMap<String, ControllerLocalMessage>,
    messages: HashMap<String, ControllerMessage>,
    timers: HashMap<String, ControllerTimer>,
    commands: Vec<(f64, ControllerStateCommand)>,
}

impl EventController {
    pub fn new() -> Self {
        Self {
            local_messages: HashMap::new(),
            messages: HashMap::new(),
            timers: HashMap::new(),
            commands: vec![],
        }
    }

    pub fn parse_log(&mut self, filename: &str) {
        let mut events: Vec<LogEntry> = Vec::new();

        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let event: LogEntry = serde_json::from_str(&line.unwrap()).unwrap();
            events.push(event);
        }

        let mut node_cnt = 0;
        let mut process_cnt = 0;

        for event in &events {
            match event {
                LogEntry::NodeStarted { .. } => node_cnt += 1,
                LogEntry::ProcessStarted { .. } => process_cnt += 1,
                _ => break,
            }
        }

        let center = Vec2::new(screen_width() / 2., screen_height() / 2.);
        let mut k = 0;
        for i in 0..(node_cnt + process_cnt) {
            let angle = (2.0 * PI / (node_cnt as f32)) * (k as f32);
            let pos = center + Vec2::from_angle(angle as f32) * CIRCLE_RADIUS;
            if let LogEntry::NodeStarted {
                time,
                node,
                node_id,
            } = &events[i]
            {
                self.commands.push((
                    *time,
                    ControllerStateCommand::NodeStarted(ControllerNode {
                        name: node.clone(),
                        id: *node_id,
                        pos,
                    }),
                ));
                k += 1;
            }
        }

        for event in events.split_off(node_cnt + process_cnt) {
            match event {
                LogEntry::NodeStarted {
                    time,
                    node,
                    node_id,
                } => {
                    let x = gen_range(0.3, 0.8);
                    let y = gen_range(0.3, 0.8);
                    let pos = Vec2::from((x * screen_height(), y * screen_width()));
                    self.commands.push((
                        time,
                        ControllerStateCommand::NodeStarted(ControllerNode {
                            name: node,
                            id: node_id,
                            pos,
                        }),
                    ));
                }
                LogEntry::ProcessStarted { .. } => {}
                LogEntry::LocalMessageSent {
                    time,
                    msg_id,
                    node,
                    proc,
                    msg,
                } => {
                    let controller_msg = ControllerLocalMessage {
                        id: msg_id.clone(),
                        node,
                        proc,
                        tip: msg.tip,
                        data: prettify_json_string(msg.data),
                        time,
                        msg_type: LocalMessageType::Sent,
                    };
                    self.local_messages.insert(msg_id.clone(), controller_msg);
                    self.commands.push((
                        time,
                        ControllerStateCommand::LocalMessageEmerged(msg_id.clone()),
                    ));
                }
                LogEntry::LocalMessageReceived {
                    time,
                    msg_id,
                    node,
                    proc,
                    msg,
                } => {
                    let controller_msg = ControllerLocalMessage {
                        id: msg_id.clone(),
                        node,
                        proc,
                        tip: msg.tip,
                        data: msg.data,
                        time,
                        msg_type: LocalMessageType::Received,
                    };
                    self.local_messages.insert(msg_id.clone(), controller_msg);
                    self.commands.push((
                        time,
                        ControllerStateCommand::LocalMessageEmerged(msg_id.clone()),
                    ));
                }
                LogEntry::MessageSent {
                    time,
                    msg_id,
                    src_node,
                    src_proc,
                    dest_node,
                    dest_proc,
                    msg,
                } => {
                    let cont_msg = ControllerMessage {
                        id: msg_id.clone(),
                        src_node,
                        src_proc,
                        dest_node,
                        dest_proc,
                        tip: msg.tip,
                        data: msg.data,
                        time_sent: time,
                        time_received: -1.0,
                        copies_received: 0,
                    };
                    self.messages.insert(cont_msg.id.clone(), cont_msg);
                    self.commands
                        .push((time, ControllerStateCommand::MessageSent(msg_id)));
                }
                LogEntry::MessageReceived { time, msg_id } => {
                    let msg = self.messages.get_mut(&msg_id).unwrap();
                    msg.time_received = time;
                    msg.copies_received += 1;
                }
                LogEntry::MessageDropped { .. } => {}
                LogEntry::NodeDisconnected { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::NodeDisconnected(node)));
                }
                LogEntry::NodeConnected { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::NodeConnected(node)));
                }
                LogEntry::NodeCrashed { .. } => {}
                LogEntry::NodeRecovered { .. } => {}
                LogEntry::TimerSet {
                    time,
                    timer_id,
                    timer_name,
                    node,
                    proc,
                    delay,
                } => {
                    let timer = ControllerTimer {
                        id: timer_id.clone(),
                        name: timer_name,
                        node,
                        proc,
                        delay,
                        time_set: time,
                        time_removed: -1.,
                    };
                    self.timers.insert(timer_id.clone(), timer);
                    self.commands
                        .push((time, ControllerStateCommand::TimerSet(timer_id)));
                }
                LogEntry::TimerFired { time, timer_id } => {
                    self.timers.get_mut(&timer_id).unwrap().time_removed = time;
                }
                LogEntry::TimerCancelled { time, timer_id } => {
                    self.timers.get_mut(&timer_id).unwrap().time_removed = time;
                }
                LogEntry::LinkDisabled { time, from, to } => {
                    self.commands
                        .push((time, ControllerStateCommand::LinkDisabled((from, to))));
                }
                LogEntry::LinkEnabled { time, from, to } => {
                    self.commands
                        .push((time, ControllerStateCommand::LinkEnabled((from, to))));
                }
                LogEntry::DropIncoming { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::DropIncoming(node)));
                }
                LogEntry::PassIncoming { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::PassIncoming(node)));
                }
                LogEntry::DropOutgoing { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::DropOutgoing(node)));
                }
                LogEntry::PassOutgoing { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::PassOutgoing(node)));
                }
                LogEntry::NetworkPartition {
                    time,
                    group1,
                    group2,
                } => {
                    self.commands.push((
                        time,
                        ControllerStateCommand::NetworkPartition((group1, group2)),
                    ));
                }
                LogEntry::NetworkReset { time } => {
                    self.commands
                        .push((time, ControllerStateCommand::NetworkReset()));
                }
                LogEntry::ProcessStateUpdated {
                    time,
                    node,
                    proc: _,
                    state,
                } => {
                    let pretty_state = prettify_json_string(state).replace("\\", "");
                    self.commands.push((
                        time,
                        ControllerStateCommand::NodeStateUpdated((node, pretty_state)),
                    ));
                }
            }
        }
    }

    pub fn send_commands(&mut self, state: &mut State) {
        self.commands.sort_by(|a, b| a.0.total_cmp(&b.0));
        for command in &self.commands {
            match &command.1 {
                ControllerStateCommand::NodeStarted(node) => {
                    state.process_node_started(command.0, node.name.clone(), node.id, node.pos);
                }
                ControllerStateCommand::MessageSent(id) => {
                    let msg = self.messages.get(id).unwrap();
                    state.process_message_sent(
                        msg.id.clone(),
                        msg.time_sent,
                        &msg.src_node,
                        &msg.dest_node,
                        msg.tip.clone(),
                        prettify_json_string(msg.data.clone()),
                        (msg.time_received - msg.time_sent) as f32,
                        msg.copies_received,
                    );
                }
                ControllerStateCommand::LocalMessageEmerged(id) => {
                    let msg = self.local_messages.get(id).unwrap();
                    let is_sent: bool;
                    match msg.msg_type {
                        LocalMessageType::Received => is_sent = false,
                        LocalMessageType::Sent => is_sent = true,
                    }
                    state.process_local_message(
                        msg.time,
                        msg.id.clone(),
                        msg.node.clone(),
                        msg.data.clone(),
                        is_sent,
                    );
                }
                ControllerStateCommand::NodeDisconnected(node) => {
                    state.process_node_disconnected(command.0, node.clone())
                }
                ControllerStateCommand::NodeConnected(node) => {
                    state.process_node_connected(command.0, node.clone())
                }
                ControllerStateCommand::TimerSet(id) => {
                    let timer = self.timers.get(id).unwrap();
                    state.process_timer_set(
                        timer.id.clone(),
                        timer.name.clone(),
                        timer.time_set,
                        timer.node.clone(),
                        timer.delay,
                        timer.time_removed,
                    );
                }
                ControllerStateCommand::LinkDisabled(link) => {
                    state.process_link_disabled(command.0, link.0.clone(), link.1.clone());
                }
                ControllerStateCommand::LinkEnabled(link) => {
                    state.process_link_enabled(command.0, link.0.clone(), link.1.clone());
                }
                ControllerStateCommand::DropIncoming(node) => {
                    state.process_drop_incoming(command.0, node.clone());
                }
                ControllerStateCommand::PassIncoming(node) => {
                    state.process_pass_incoming(command.0, node.clone());
                }
                ControllerStateCommand::DropOutgoing(node) => {
                    state.process_drop_outgoing(command.0, node.clone());
                }
                ControllerStateCommand::PassOutgoing(node) => {
                    state.process_pass_outgoing(command.0, node.clone());
                }
                ControllerStateCommand::NetworkPartition((group1, group2)) => {
                    state.process_network_partition(command.0, group1.clone(), group2.clone());
                }
                ControllerStateCommand::NetworkReset() => {
                    state.process_network_reset(command.0);
                }
                ControllerStateCommand::NodeStateUpdated((node, process_state)) => {
                    state.process_state_updated(
                        command.0,
                        node.to_string(),
                        process_state.to_string(),
                    );
                }
            }
        }
    }
}

pub enum LocalMessageType {
    Sent,
    Received,
}

pub struct ControllerLocalMessage {
    id: String,
    node: String,
    proc: String,
    tip: String,
    data: String,
    time: f64,
    msg_type: LocalMessageType,
}

pub struct ControllerMessage {
    id: String,
    src_node: String,
    src_proc: String,
    dest_node: String,
    dest_proc: String,
    tip: String,
    data: String,
    time_sent: f64,
    time_received: f64,
    copies_received: u64,
}

pub struct ControllerTimer {
    id: String,
    name: String,
    node: String,
    proc: String,
    time_set: f64,
    delay: f64,
    time_removed: f64,
}

#[derive(Debug)]
pub struct ControllerNode {
    name: String,
    id: u32,
    pos: Vec2,
}
