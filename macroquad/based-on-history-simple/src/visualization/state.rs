use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use macroquad::prelude::*;

use super::utilities::*;

#[derive(Clone, Debug)]
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
        let node = StateNode {
            id: id,
            pos: pos,
            alive: true,
        };
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
        let dist = calc_dist(
            self.nodes.get(from).unwrap().pos,
            self.nodes.get(to).unwrap().pos,
        );
        let drop = duration <= 0.0;
        let speed = if !drop {
            1.0 / (FPS * duration / dist)
        } else {
            1.0 / (FPS * 3.0 / dist)
        };
        self.event_queue.push_back(EventQueueItem {
            timestamp: timestamp,
            event: StateEvent::SendMessage(StateMessage {
                pos: self.nodes.get(from).unwrap().pos,
                from: Rc::clone(self.nodes.get(from).unwrap()),
                to: Rc::clone(self.nodes.get(to).unwrap()),
                speed,
                data,
                drop,
            }),
        });
    }

    pub fn process_node_down(&mut self, timestamp: f64, id: String) {
        self.event_queue.push_back(EventQueueItem {
            timestamp: timestamp,
            event: StateEvent::NodeDown(id),
        });
    }

    pub fn process_node_up(&mut self, timestamp: f64, id: String) {
        self.event_queue.push_back(EventQueueItem {
            timestamp: timestamp,
            event: StateEvent::NodeUp(id),
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
        let time = (self.get_current_time().floor() as u32).to_string();
        draw_text_ex(
            &time,
            screen_width() * 0.91,
            screen_height() * 0.96,
            TextParams {
                font_size: (screen_width() / 12.0).floor() as u16,
                color: WHITE,
                ..Default::default()
            },
        );
    }

    pub fn get_current_time(&self) -> f64 {
        get_time() - self.start_time
    }

    pub fn process_event(&mut self, timestamp: f64, event: StateEvent) -> bool {
        if self.get_current_time() < timestamp {
            return false;
        }
        match event {
            StateEvent::AddNode(node) => {
                self.nodes.insert(node.id.clone(), Rc::new(node));
            }
            StateEvent::SendMessage(msg) => {
                self.messages.push(msg.clone());
                println!("{:?}", msg);
            }
            StateEvent::NodeDown(id) => Rc::make_mut(self.nodes.get_mut(&id).unwrap()).make_dead(),
            StateEvent::NodeUp(id) => Rc::make_mut(self.nodes.get_mut(&id).unwrap()).make_alive(),
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct StateNode {
    id: String,
    pos: Vec2,
    alive: bool,
}

impl StateNode {
    pub fn draw(&self) {
        draw_circle(
            self.pos.x,
            self.pos.y,
            NODE_RADIUS,
            if self.alive {
                ALIVE_NODE_COLOR
            } else {
                DEAD_NODE_COLOR
            },
        );

        let offset = NODE_RADIUS / 2.25;

        draw_text_ex(
            &self.id,
            self.pos.x - offset,
            self.pos.y + offset,
            TextParams {
                font_size: (NODE_RADIUS * 2.0).floor() as u16,
                color: BLACK,
                ..Default::default()
            },
        );
    }

    pub fn make_alive(&mut self) {
        self.alive = true;
    }

    pub fn make_dead(&mut self) {
        self.alive = false;
    }
}

#[derive(Debug, Clone)]
pub struct StateMessage {
    pos: Vec2,
    from: Rc<StateNode>,
    to: Rc<StateNode>,
    speed: f32,
    data: String,
    drop: bool,
}

impl StateMessage {
    pub fn update(&mut self) {
        self.pos += (self.to.pos - self.pos).normalize() * self.speed;
    }

    pub fn draw(&self) {
        let overall_dist = calc_dist(self.from.pos, self.to.pos);
        let color = if self.drop && calc_dist(self.from.pos, self.pos) >= overall_dist * 0.4 {
            RED
        } else {
            BLUE
        };
        draw_circle(self.pos.x, self.pos.y, MESSAGE_RADIUS, color);
    }

    pub fn is_delivered(&self) -> bool {
        if !self.drop {
            calc_dist(self.pos, self.to.pos) < 2.0
        } else {
            let overall_dist = calc_dist(self.from.pos, self.to.pos);
            calc_dist(self.from.pos, self.pos) >= overall_dist * 0.7
        }
    }
}
