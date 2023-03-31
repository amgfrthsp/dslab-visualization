use std::{
    collections::{HashMap, HashSet, VecDeque},
    f32::EPSILON,
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

#[derive(Clone)]
pub struct UIData {
    show_node_windows: HashMap<String, bool>,
}

pub struct State {
    nodes: HashMap<String, Rc<StateNode>>,
    messages: Vec<StateMessage>,
    event_queue: VecDeque<EventQueueItem>,
    current_time: f64,
    last_updated: f64,
    paused: bool,
    global_speed: f32,
    ui_data: UIData,
}

impl State {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            messages: vec![],
            event_queue: VecDeque::new(),
            current_time: 0.0,
            last_updated: 0.0,
            paused: false,
            global_speed: 1.0,
            ui_data: UIData {
                show_node_windows: HashMap::new(),
            },
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
        self.check_keyboard_events();

        if self.paused {
            self.last_updated = get_time();
            return;
        } else {
            self.current_time += (get_time() - self.last_updated) * (self.global_speed as f64);
            self.last_updated = get_time();
        }

        while let Some(event) = self.event_queue.front() {
            if self.process_event(event.timestamp, event.event.clone()) {
                self.event_queue.pop_front();
            } else {
                break;
            }
        }
        for msg in &mut self.messages {
            msg.update(self.global_speed);
        }
        self.messages.retain(|msg| !msg.is_delivered());
    }

    pub fn draw(&mut self) {
        //self.draw_ui();
        for (_, node) in &self.nodes {
            node.draw();
        }
        for msg in &self.messages {
            msg.draw();
        }
        self.draw_time();
        self.draw_speed();
    }

    pub fn draw_time(&self) {
        let time_str = (self.current_time.floor() as u32).to_string();
        draw_text_ex(
            &time_str,
            screen_width() * 0.87,
            screen_height() * 0.96,
            TextParams {
                font_size: (screen_width() / 12.0).floor() as u16,
                color: WHITE,
                ..Default::default()
            },
        );
    }

    pub fn draw_speed(&self) {
        let speed = format!("speed:{:.2}", self.global_speed);
        draw_text_ex(
            &speed,
            screen_width() * 0.02,
            screen_height() * 0.97,
            TextParams {
                font_size: (screen_width() / 24.0).floor() as u16,
                color: WHITE,
                ..Default::default()
            },
        );
    }

    pub fn check_keyboard_events(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }
        if is_key_down(KeyCode::Up) {
            self.global_speed += SPEED_DELTA;
        }
        if is_key_down(KeyCode::Down) {
            if self.global_speed - SPEED_DELTA > 0.0 {
                self.global_speed -= SPEED_DELTA;
            }
        }
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = mouse_position();
            let node_id = self.get_node_by_mouse_pos(mouse_pos.0, mouse_pos.1);
            if node_id.is_some() {
                self.ui_data
                    .show_node_windows
                    .insert(node_id.unwrap().0, true);
            }
        }
    }

    // -> Option(String, bool) -- bool stands for whether a node id or a message id is returned
    pub fn get_node_by_mouse_pos(&mut self, x: f32, y: f32) -> Option<(String, bool)> {
        for (_, node) in &self.nodes {
            if calc_dist(Vec2 { x, y }, node.pos) < NODE_RADIUS {
                return Some((node.id.clone(), true));
            }
        }
        return None;
    }

    pub fn draw_ui(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            for (node_id, show_window) in &mut self.ui_data.show_node_windows {
                let node = self.nodes.get(node_id).unwrap();
                egui::Window::new(format!("Node {}", node_id))
                    .open(show_window)
                    .show(egui_ctx, |ui| {
                        ui.label(format!(
                            "Status: {}",
                            if node.alive { "Alive" } else { "Crashed" }
                        ));
                    });
            }
        });
    }

    pub fn process_event(&mut self, timestamp: f64, event: StateEvent) -> bool {
        if self.current_time < timestamp {
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
    pub fn update(&mut self, global_speed: f32) {
        self.pos += (self.to.pos - self.pos).normalize() * self.speed * global_speed;
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
            calc_dist(self.pos, self.to.pos) < 5.0
        } else {
            let overall_dist = calc_dist(self.from.pos, self.to.pos);
            calc_dist(self.from.pos, self.pos) >= overall_dist * 0.7
        }
    }
}
