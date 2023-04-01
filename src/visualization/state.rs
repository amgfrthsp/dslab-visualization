use std::{
    cell::RefCell,
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

#[derive(Clone)]
pub struct UIData {
    show_node_windows: HashMap<String, bool>,
    show_msg_windows: HashMap<String, bool>,
}

pub struct State {
    nodes: HashMap<String, Rc<RefCell<StateNode>>>,
    messages: HashMap<String, StateMessage>,
    event_queue: VecDeque<EventQueueItem>,
    current_time: f64,
    last_updated: f64,
    paused: bool,
    global_speed: f32,
    last_clicked: f64,
    selected_node: Option<String>,
    selected_mouse_position: Vec2,
    ui_data: UIData,
}

impl State {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            messages: HashMap::new(),
            event_queue: VecDeque::new(),
            current_time: 0.0,
            last_updated: 0.0,
            paused: false,
            global_speed: 1.0,
            last_clicked: -1.,
            selected_node: None,
            selected_mouse_position: Vec2::new(0., 0.),
            ui_data: UIData {
                show_node_windows: HashMap::new(),
                show_msg_windows: HashMap::new(),
            },
        }
    }

    pub fn add_node(&mut self, timestamp: f64, id: String, pos: Vec2) {
        let node = StateNode {
            id: id,
            pos: pos,
            alive: true,
        };
        self.nodes
            .insert(node.id.clone(), Rc::new(RefCell::new(node)));
        /*self.event_queue
        .push_back((timestamp, StateEvent::AddNode(StateNode { id, pos })));*/
    }

    pub fn send_message(
        &mut self,
        id: String,
        timestamp: f64,
        from: &str,
        to: &str,
        data: String,
        duration: f32,
    ) {
        let dist = calc_dist(
            self.nodes.get(from).unwrap().borrow().pos,
            self.nodes.get(to).unwrap().borrow().pos,
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
                id: id,
                pos: self.nodes.get(from).unwrap().borrow().pos,
                from: self.nodes.get(from).unwrap().clone(),
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
        for (_, msg) in &mut self.messages {
            msg.update(self.global_speed);
        }
        self.messages.retain(|_, msg| !msg.is_delivered());
    }

    pub fn draw(&mut self) {
        for (_, node) in &self.nodes {
            node.borrow().draw();
        }
        for (_, msg) in &self.messages {
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
            if self.selected_node.is_none() {
                if let Some(node_id) = self.get_node_by_mouse_pos(mouse_pos) {
                    self.selected_node = Some(node_id);
                    self.selected_mouse_position = Vec2::new(mouse_pos.0, mouse_pos.1);
                }
            } else {
                let node_id = self.selected_node.clone().unwrap();
                let node = self.nodes.get_mut(&node_id).unwrap();
                let drag_direction =
                    Vec2::new(mouse_pos.0, mouse_pos.1) - self.selected_mouse_position;
                if !drag_direction.is_nan() {
                    let new_pos = node.borrow().pos + drag_direction;
                    node.borrow_mut().update_pos(new_pos);
                }
                self.selected_mouse_position = Vec2::new(mouse_pos.0, mouse_pos.1);
            }

            if let Some(msg_id) = self.get_msg_by_mouse_pos(mouse_pos) {
                self.ui_data.show_msg_windows.insert(msg_id, true);
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            self.last_clicked = self.current_time;
        }
        if is_mouse_button_released(MouseButton::Left) {
            if self.current_time - self.last_clicked <= SINGLE_CLICK_DELAY
                && self.selected_node.is_some()
            {
                self.ui_data
                    .show_node_windows
                    .insert(self.selected_node.clone().unwrap(), true);
            }
            self.selected_node = None;
        }
    }

    pub fn get_msg_by_mouse_pos(&mut self, mouse_pos: (f32, f32)) -> Option<String> {
        for (_, msg) in &self.messages {
            if calc_dist(Vec2::new(mouse_pos.0, mouse_pos.1), msg.pos) < MESSAGE_RADIUS {
                return Some(msg.id.clone());
            }
        }
        return None;
    }

    pub fn get_node_by_mouse_pos(&mut self, mouse_pos: (f32, f32)) -> Option<String> {
        for (_, node) in &self.nodes {
            if calc_dist(Vec2::new(mouse_pos.0, mouse_pos.1), node.borrow().pos) < NODE_RADIUS {
                return Some(node.borrow().id.clone());
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
                            if node.borrow().alive {
                                "Alive"
                            } else {
                                "Crashed"
                            }
                        ));
                    });
            }
            for (msg_id, show_window) in &mut self.ui_data.show_msg_windows {
                if !self.messages.contains_key(msg_id) {
                    continue;
                }
                let msg = self.messages.get(msg_id).unwrap();
                egui::Window::new(format!("Message {}", msg_id))
                    .open(show_window)
                    .show(egui_ctx, |ui| {
                        ui.label(format!("From: {}", msg.from.borrow().id.clone()));
                        ui.label(format!("To: {}", msg.to.borrow().id.clone()));
                        ui.label(format!("Data: {}", msg.data.clone()));
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
                self.nodes
                    .insert(node.id.clone(), Rc::new(RefCell::new(node)));
            }
            StateEvent::SendMessage(msg) => {
                self.messages.insert(msg.id.clone(), msg.clone());
            }
            StateEvent::NodeDown(id) => self.nodes.get_mut(&id).unwrap().borrow_mut().make_dead(),
            StateEvent::NodeUp(id) => self.nodes.get_mut(&id).unwrap().borrow_mut().make_alive(),
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
    pub fn update_pos(&mut self, new_pos: Vec2) {
        self.pos = new_pos;
    }

    pub fn draw(&self) {
        println!("From node {}: {}", self.id, self.pos);
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
    id: String,
    pos: Vec2,
    from: Rc<RefCell<StateNode>>,
    to: Rc<RefCell<StateNode>>,
    speed: f32,
    data: String,
    drop: bool,
}

impl StateMessage {
    pub fn update(&mut self, global_speed: f32) {
        println!(
            "From message to node {}: {}",
            self.to.borrow().id,
            self.to.borrow().pos
        );
        self.pos += (self.to.borrow().pos - self.pos).normalize() * self.speed * global_speed;
    }

    pub fn draw(&self) {
        let overall_dist = calc_dist(self.from.borrow().pos, self.to.borrow().pos);
        let color =
            if self.drop && calc_dist(self.from.borrow().pos, self.pos) >= overall_dist * 0.4 {
                RED
            } else {
                BLUE
            };
        draw_circle(self.pos.x, self.pos.y, MESSAGE_RADIUS, color);
    }

    pub fn is_delivered(&self) -> bool {
        if !self.drop {
            calc_dist(self.pos, self.to.borrow().pos) < 5.0
        } else {
            let overall_dist = calc_dist(self.from.borrow().pos, self.to.borrow().pos);
            calc_dist(self.from.borrow().pos, self.pos) >= overall_dist * 0.7
        }
    }
}
