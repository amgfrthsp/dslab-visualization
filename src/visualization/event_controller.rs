use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{collections::HashMap, f32::consts::PI};

use crate::logs::log_entities::*;

use super::state::state::State;
use super::utilities::CIRCLE_RADIUS;

#[derive(Debug)]
pub enum ControllerStateCommand {
    SendMessage(String),
    ProcessLocalMessage(String),
    NodeConnected(String),
    NodeDisconnected(String),
    AddNode(ControllerNode),
    TimerSet(String),
    DisableLink((String, String)),
    EnableLink((String, String)),
    DropIncoming(String),
    PassIncoming(String),
    DropOutgoing(String),
    PassOutgoing(String),
    MakePartition((Vec<String>, Vec<String>)),
    ResetNetwork(),
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
        let mut events: Vec<Event> = Vec::new();

        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let event: Event = serde_json::from_str(&line.unwrap()).unwrap();
            events.push(event);
        }

        let mut node_cnt = 0;

        for event in &events {
            if let Event::NodeAdded {
                time: _,
                node: _,
                node_id: _,
            } = event
            {
                node_cnt += 1;
            } else {
                break;
            }
        }

        let center = Vec2::new(screen_width() / 2., screen_height() / 2.);
        for i in 0..node_cnt {
            let angle = (2.0 * PI / (node_cnt as f32)) * (i as f32);
            let pos = center + Vec2::from_angle(angle as f32) * CIRCLE_RADIUS;
            if let Event::NodeAdded {
                time,
                node,
                node_id,
            } = &events[i]
            {
                self.commands.push((
                    *time,
                    ControllerStateCommand::AddNode(ControllerNode {
                        name: node.clone(),
                        id: *node_id,
                        pos,
                    }),
                ));
            }
        }

        for event in events.split_off(node_cnt) {
            match event {
                Event::NodeAdded {
                    time,
                    node,
                    node_id,
                } => {
                    let x = gen_range(0.3, 0.8);
                    let y = gen_range(0.3, 0.8);
                    let pos = Vec2::from((x * screen_height(), y * screen_width()));
                    self.commands.push((
                        time,
                        ControllerStateCommand::AddNode(ControllerNode {
                            name: node,
                            id: node_id,
                            pos,
                        }),
                    ));
                }
                Event::ProcessAdded { .. } => {}
                Event::LocalMessageSent {
                    time,
                    msg_id,
                    node,
                    proc,
                    msg_tip,
                    msg_data,
                } => {
                    let controller_msg = ControllerLocalMessage {
                        id: msg_id.clone(),
                        node,
                        proc,
                        tip: msg_tip,
                        data: msg_data,
                        time,
                        msg_type: LocalMessageType::Sent,
                    };
                    self.local_messages.insert(msg_id.clone(), controller_msg);
                    self.commands.push((
                        time,
                        ControllerStateCommand::ProcessLocalMessage(msg_id.clone()),
                    ));
                }
                Event::LocalMessageReceived {
                    time,
                    msg_id,
                    node,
                    proc,
                    msg_tip,
                    msg_data,
                } => {
                    let controller_msg = ControllerLocalMessage {
                        id: msg_id.clone(),
                        node,
                        proc,
                        tip: msg_tip,
                        data: msg_data,
                        time,
                        msg_type: LocalMessageType::Received,
                    };
                    self.local_messages.insert(msg_id.clone(), controller_msg);
                    self.commands.push((
                        time,
                        ControllerStateCommand::ProcessLocalMessage(msg_id.clone()),
                    ));
                }
                Event::MessageSent {
                    time,
                    msg_id,
                    src_node,
                    src_proc,
                    dest_node,
                    dest_proc,
                    msg_tip,
                    msg_data,
                } => {
                    let msg = ControllerMessage {
                        id: msg_id.clone(),
                        src_node,
                        src_proc,
                        dest_node,
                        dest_proc,
                        tip: msg_tip,
                        data: msg_data,
                        time_sent: time,
                        time_received: -1.0,
                    };
                    self.messages.insert(msg.id.clone(), msg);
                    self.commands
                        .push((time, ControllerStateCommand::SendMessage(msg_id)));
                }
                Event::MessageReceived { time, msg_id } => {
                    self.messages.get_mut(&msg_id).unwrap().time_received = time;
                }
                Event::MessageDropped { .. } => {}
                Event::NodeDisconnected { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::NodeDisconnected(node)));
                }
                Event::NodeConnected { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::NodeConnected(node)));
                }
                Event::NodeCrashed { .. } => {}
                Event::NodeRecovered { .. } => {}
                Event::TimerSet {
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
                Event::TimerFired { time, timer_id } => {
                    self.timers.get_mut(&timer_id).unwrap().time_removed = time;
                }
                Event::TimerCancelled { time, timer_id } => {
                    self.timers.get_mut(&timer_id).unwrap().time_removed = time;
                }
                Event::LinkDisabled { time, from, to } => {
                    self.commands
                        .push((time, ControllerStateCommand::DisableLink((from, to))));
                }
                Event::LinkEnabled { time, from, to } => {
                    self.commands
                        .push((time, ControllerStateCommand::EnableLink((from, to))));
                }
                Event::DropIncoming { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::DropIncoming(node)));
                }
                Event::PassIncoming { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::PassIncoming(node)));
                }
                Event::DropOutgoing { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::DropOutgoing(node)));
                }
                Event::PassOutgoing { time, node } => {
                    self.commands
                        .push((time, ControllerStateCommand::PassOutgoing(node)));
                }
                Event::MakePartition {
                    time,
                    group1,
                    group2,
                } => {
                    self.commands.push((
                        time,
                        ControllerStateCommand::MakePartition((group1, group2)),
                    ));
                }
                Event::ResetNetwork { time } => {
                    self.commands
                        .push((time, ControllerStateCommand::ResetNetwork()));
                }
            }
        }
    }

    pub fn send_commands(&mut self, state: &mut State) {
        self.commands.sort_by(|a, b| a.0.total_cmp(&b.0));
        for command in &self.commands {
            match &command.1 {
                ControllerStateCommand::AddNode(node) => {
                    state.add_node(command.0, node.name.clone(), node.id, node.pos);
                }
                ControllerStateCommand::SendMessage(id) => {
                    let msg = self.messages.get(id).unwrap();
                    state.send_message(
                        msg.id.clone(),
                        msg.time_sent,
                        &msg.src_node,
                        &msg.dest_node,
                        msg.tip.clone(),
                        msg.data.clone(),
                        (msg.time_received - msg.time_sent) as f32,
                    );
                }
                ControllerStateCommand::ProcessLocalMessage(id) => {
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
                ControllerStateCommand::DisableLink(link) => {
                    state.process_link_disabled(command.0, link.0.clone(), link.1.clone());
                }
                ControllerStateCommand::EnableLink(link) => {
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
                ControllerStateCommand::MakePartition((group1, group2)) => {
                    state.process_make_partition(command.0, group1.clone(), group2.clone());
                }
                ControllerStateCommand::ResetNetwork() => {
                    state.process_reset_network(command.0);
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
