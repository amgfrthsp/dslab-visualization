use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use macroquad::prelude::*;

use super::utilities::*;

#[derive(Clone)]
pub enum StateEvent {
    AddNode(StateNode),
    SendMessage(StateMessage),
    NodeUp(String),
    NodeDown(String),
}

#[derive(Clone)]
pub struct EventQueueItem {
    timestamp: f64,
    event: StateEvent,
}

pub struct State {
    nodes: HashMap<String, Rc<StateNode>>,
    messages: Vec<StateMessage>,
    event_queue: VecDeque<EventQueueItem>,
    start_time: f64,
}

impl State {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            messages: vec![],
            event_queue: VecDeque::new(),
            start_time: 0.0,
        }
    }

    pub fn add_node(&mut self, timestamp: f64, id: String, pos: Vec2) {
        let node = StateNode { id, pos };
        self.nodes.insert(node.id.clone(), Rc::new(node));
        /*self.event_queue
        .push_back((timestamp, StateEvent::AddNode(StateNode { id, pos })));*/
    }

    pub fn send_message(
        &mut self,
        timestamp: f64,
        from: &str,
        to: &str,
        data: String,
        duration: f32,
    ) {
        if duration < 0.0 {
            panic!("bad duration");
        }
        let dist = calc_dist(
            self.nodes.get(from).unwrap().pos,
            self.nodes.get(to).unwrap().pos,
        );
        let speed = 1.0 / (FPS * duration / dist);
        self.event_queue.push_back(EventQueueItem {
            timestamp: timestamp,
            event: StateEvent::SendMessage(StateMessage {
                pos: self.nodes.get(from).unwrap().pos,
                to: Rc::clone(self.nodes.get(to).unwrap()),
                speed,
                data,
            }),
        });
    }

    pub fn update(&mut self) {
        if self.start_time == 0.0 && !self.event_queue.is_empty() {
            self.start_time = get_time();
        }

        while let Some(event) = self.event_queue.front() {
            if self.process_event(event.timestamp, event.event.clone()) {
                self.event_queue.pop_front();
            } else {
                break;
            }
        }
        for msg in &mut self.messages {
            msg.update();
        }
        self.messages.retain(|msg| !msg.is_delivered());
    }

    pub fn draw(&self) {
        for (_, node) in &self.nodes {
            node.draw();
        }
        for msg in &self.messages {
            msg.draw();
        }
    }

    pub fn get_current_time(&self) -> f64 {
        get_time() - self.start_time
    }

    pub fn process_event(&mut self, timestamp: f64, event: StateEvent) -> bool {
        if timestamp < self.get_current_time() {
            return false;
        }
        match event {
            StateEvent::AddNode(node) => {
                self.nodes.insert(node.id.clone(), Rc::new(node));
            }
            StateEvent::SendMessage(msg) => {
                self.messages.push(msg.clone());
            }
            StateEvent::NodeDown(id) => panic!("not implemented"),
            StateEvent::NodeUp(id) => panic!("not implemented"),
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct StateNode {
    id: String,
    pos: Vec2,
}

impl StateNode {
    pub fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, NODE_RADIUS, YELLOW);
    }
}

#[derive(Debug, Clone)]
pub struct StateMessage {
    pos: Vec2,
    to: Rc<StateNode>,
    speed: f32,
    data: String,
}

impl StateMessage {
    pub fn update(&mut self) {
        self.pos += (self.to.pos - self.pos).normalize() * self.speed;
    }

    pub fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, MESSAGE_RADIUS, BLUE);
    }

    pub fn is_delivered(&self) -> bool {
        calc_dist(self.pos, self.to.pos) < 2.0
    }
}
