use macroquad::prelude::*;
use std::fs;
use std::{collections::HashMap, f32::consts::PI};

use crate::logs::log_entities::*;

use super::state::State;
use super::utilities::CIRCLE_RADIUS;

#[derive(Debug)]
pub enum ControllerStateCommand {
    SendMessage(String),
    NodeUp(String),
    NodeDown(String),
    AddNode(ControllerNode),
}

pub struct EventController {
    messages: HashMap<String, ControllerMessage>,
    commands: Vec<(f64, ControllerStateCommand)>,
}

impl EventController {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
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
                Event::TypeSent(e) => {
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
                Event::TypeReceived(e) => {
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
            }
        }
    }

    pub fn send_commands(&self, state: &mut State) {
        for command in &self.commands {
            println!("{:?}", command);
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
                ControllerStateCommand::NodeDown(id) => {
                    state.process_node_down(command.0, id.to_string())
                }
                ControllerStateCommand::NodeUp(id) => {
                    state.process_node_up(command.0, id.to_string())
                }
            }
        }
    }
}

pub struct ControllerMessage {
    id: String,
    from: String,
    to: String,
    data: String,
    time_sent: f64,
    time_received: f64,
}
#[derive(Debug)]
pub struct ControllerNode {
    id: String,
    pos: Vec2,
}
