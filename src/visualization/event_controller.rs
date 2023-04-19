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
            //println!("{:?}", event);
            events.push(event);
        }

        let mut node_cnt = 0;

        for event in &events {
            if let Event::TypeNodeAdded(_) = event {
                node_cnt += 1;
            } else {
                break;
            }
        }

        let center = Vec2::new(screen_width() / 2., screen_height() / 2.);
        for i in 0..node_cnt {
            let angle = (2.0 * PI / (node_cnt as f32)) * (i as f32);
            let pos = center + Vec2::from_angle(angle as f32) * CIRCLE_RADIUS;
            if let Event::TypeNodeAdded(node_added) = &events[i] {
                self.commands.push((
                    node_added.timestamp,
                    ControllerStateCommand::AddNode(ControllerNode {
                        name: node_added.name.clone(),
                        id: node_added.id,
                        pos,
                    }),
                ));
            }
        }

        for event in events.split_off(node_cnt) {
            match event {
                Event::TypeNodeAdded(e) => {
                    let x = gen_range(0.3, 0.8);
                    let y = gen_range(0.3, 0.8);
                    let pos = Vec2::from((x * screen_height(), y * screen_width()));
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::AddNode(ControllerNode {
                            name: e.name,
                            id: e.id,
                            pos,
                        }),
                    ));
                }
                Event::TypeLocalMessageSent(e) => {
                    let msg = ControllerLocalMessage {
                        id: e.msg.id.clone(),
                        node_name: e.msg.node_name,
                        data: e.msg.data,
                        timestampt: e.timestamp,
                        msg_type: LocalMessageType::Sent,
                    };
                    self.local_messages.insert(e.msg.id.clone(), msg);
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::ProcessLocalMessage(e.msg.id),
                    ));
                }
                Event::TypeLocalMessageReceived(e) => {
                    let msg = ControllerLocalMessage {
                        id: e.msg.id.clone(),
                        node_name: e.msg.node_name,
                        data: e.msg.data,
                        timestampt: e.timestamp,
                        msg_type: LocalMessageType::Received,
                    };
                    self.local_messages.insert(e.msg.id.clone(), msg);
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::ProcessLocalMessage(e.msg.id),
                    ));
                }
                Event::TypeMessageSent(e) => {
                    let msg = ControllerMessage {
                        id: e.msg.id.clone(),
                        src: e.msg.src,
                        dest: e.msg.dest,
                        tip: e.msg.tip,
                        data: e.msg.data,
                        time_sent: e.timestamp,
                        time_received: -1.0,
                    };
                    self.messages.insert(e.msg.id.clone(), msg);
                    self.commands
                        .push((e.timestamp, ControllerStateCommand::SendMessage(e.msg.id)));
                }
                Event::TypeMessageReceived(e) => {
                    self.messages.get_mut(&e.id).unwrap().time_received = e.timestamp;
                }
                Event::TypeNodeDisconnected(e) => {
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::NodeDisconnected(e.node_name),
                    ));
                }
                Event::TypeNodeConnected(e) => {
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::NodeConnected(e.node_name),
                    ));
                }
                Event::TypeTimerSet(e) => {
                    let timer = ControllerTimer {
                        id: e.timer.id.clone(),
                        node_name: e.timer.node_name,
                        delay: e.timer.delay,
                        time_set: e.timestamp,
                        time_removed: -1.,
                    };
                    self.timers.insert(e.timer.id.clone(), timer);
                    self.commands
                        .push((e.timestamp, ControllerStateCommand::TimerSet(e.timer.id)));
                }
                Event::TypeTimerRemoved(e) => {
                    self.timers.get_mut(&e.id).unwrap().time_removed = e.timestamp;
                }
                Event::TypeLinkDisabled(e) => {
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::DisableLink((e.from, e.to)),
                    ));
                }
                Event::TypeLinkEnabled(e) => {
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::EnableLink((e.from, e.to)),
                    ));
                }
                Event::TypeDropIncoming(e) => {
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::DropIncoming(e.node_name),
                    ));
                }
                Event::TypePassIncoming(e) => {
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::PassIncoming(e.node_name),
                    ));
                }
                Event::TypeDropOutgoing(e) => {
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::DropOutgoing(e.node_name),
                    ));
                }
                Event::TypePassOutgoing(e) => {
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::PassOutgoing(e.node_name),
                    ));
                }
                Event::TypeMakePartition(e) => {
                    self.commands.push((
                        e.timestamp,
                        ControllerStateCommand::MakePartition((e.group1, e.group2)),
                    ));
                }
                Event::TypeResetNetwork(e) => {
                    self.commands
                        .push((e.timestamp, ControllerStateCommand::ResetNetwork()));
                }
            }
        }
    }

    pub fn send_commands(&mut self, state: &mut State) {
        self.commands.sort_by(|a, b| a.0.total_cmp(&b.0));
        for command in &self.commands {
            //println!("{}", command.0);
            match &command.1 {
                ControllerStateCommand::AddNode(node) => {
                    state.add_node(command.0, node.name.clone(), node.id, node.pos);
                }
                ControllerStateCommand::SendMessage(id) => {
                    let msg = self.messages.get(id).unwrap();
                    state.send_message(
                        msg.id.clone(),
                        msg.time_sent,
                        &msg.src,
                        &msg.dest,
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
                        msg.timestampt,
                        msg.id.clone(),
                        msg.node_name.clone(),
                        msg.data.clone(),
                        is_sent,
                    );
                }
                ControllerStateCommand::NodeDisconnected(node_name) => {
                    state.process_node_disconnected(command.0, node_name.clone())
                }
                ControllerStateCommand::NodeConnected(node_name) => {
                    state.process_node_connected(command.0, node_name.clone())
                }
                ControllerStateCommand::TimerSet(id) => {
                    let timer = self.timers.get(id).unwrap();
                    state.process_timer_set(
                        timer.id.clone(),
                        timer.time_set,
                        timer.node_name.clone(),
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
                ControllerStateCommand::DropIncoming(node_name) => {
                    state.process_drop_incoming(command.0, node_name.clone());
                }
                ControllerStateCommand::PassIncoming(node_name) => {
                    state.process_pass_incoming(command.0, node_name.clone());
                }
                ControllerStateCommand::DropOutgoing(node_name) => {
                    state.process_drop_outgoing(command.0, node_name.clone());
                }
                ControllerStateCommand::PassOutgoing(node_name) => {
                    state.process_pass_outgoing(command.0, node_name.clone());
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
    node_name: String,
    data: String,
    timestampt: f64,
    msg_type: LocalMessageType,
}

pub struct ControllerMessage {
    id: String,
    src: String,
    dest: String,
    tip: String,
    data: String,
    time_sent: f64,
    time_received: f64,
}

pub struct ControllerTimer {
    id: String,
    node_name: String,
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
