use macroquad::prelude::*;
use std::fs;
use std::{collections::HashMap, f32::consts::PI};

use crate::logs::log_entities::*;

use super::state::state::State;
use super::utilities::CIRCLE_RADIUS;

#[derive(Debug)]
pub enum ControllerStateCommand {
    SendMessage(String),
    ProcessLocalMessage(String),
    NodeUp(String),
    NodeDown(String),
    AddNode(ControllerNode),
    TimerSet(String),
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
        let contents = fs::read_to_string(filename).unwrap();
        let log: EventLog = serde_json::from_str(&contents).unwrap();

        let center = Vec2::new(screen_width() / 2., screen_height() / 2.);
        for i in 0..log.node_cnt {
            let angle = (2.0 * PI / (log.node_cnt as f32)) * (i as f32);
            let pos = center + Vec2::from_angle(angle as f32) * CIRCLE_RADIUS;
            self.commands.push((
                0.0,
                ControllerStateCommand::AddNode(ControllerNode {
                    id: i.to_string(),
                    pos,
                }),
            ));
        }

        for event in log.events {
            match event {
                Event::TypeLocalMessageSent(e) => {
                    let msg = ControllerLocalMessage {
                        id: e.msg.id.clone(),
                        node_id: e.msg.node_id,
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
                        node_id: e.msg.node_id,
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
                        from: e.msg.from,
                        to: e.msg.to,
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
                Event::TypeNodeDown(e) => {
                    self.commands
                        .push((e.timestamp, ControllerStateCommand::NodeDown(e.id)));
                }
                Event::TypeNodeUp(e) => {
                    self.commands
                        .push((e.timestamp, ControllerStateCommand::NodeUp(e.id)));
                }
                Event::TypeTimerSet(e) => {
                    let timer = ControllerTimer {
                        id: e.timer.id.clone(),
                        node_id: e.timer.node_id,
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
            }
        }
    }

    pub fn send_commands(&self, state: &mut State) {
        for command in &self.commands {
            match &command.1 {
                ControllerStateCommand::AddNode(node) => {
                    state.add_node(0.0, node.id.clone(), node.pos);
                }
                ControllerStateCommand::SendMessage(id) => {
                    let msg = self.messages.get(id).unwrap();
                    state.send_message(
                        msg.id.clone(),
                        msg.time_sent,
                        &msg.from,
                        &msg.to,
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
                        msg.node_id.clone(),
                        msg.data.clone(),
                        is_sent,
                    );
                }
                ControllerStateCommand::NodeDown(id) => {
                    state.process_node_down(command.0, id.to_string())
                }
                ControllerStateCommand::NodeUp(id) => {
                    state.process_node_up(command.0, id.to_string())
                }
                ControllerStateCommand::TimerSet(id) => {
                    let timer = self.timers.get(id).unwrap();
                    state.process_timer_set(
                        timer.id.clone(),
                        timer.time_set,
                        timer.node_id.clone(),
                        timer.delay,
                        timer.time_removed,
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
    node_id: String,
    data: String,
    timestampt: f64,
    msg_type: LocalMessageType,
}

pub struct ControllerMessage {
    id: String,
    from: String,
    to: String,
    data: String,
    time_sent: f64,
    time_received: f64,
}

pub struct ControllerTimer {
    id: String,
    node_id: String,
    time_set: f64,
    delay: f64,
    time_removed: f64,
}

#[derive(Debug)]
pub struct ControllerNode {
    id: String,
    pos: Vec2,
}
