use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    f32::consts::PI,
    rc::Rc,
};

use egui::ScrollArea;
use macroquad::prelude::*;

use super::utilities::*;

#[derive(Clone, Debug)]
pub enum StateEvent {
    AddNode(StateNode),
    SendMessage(String),
    NodeUp(String),
    NodeDown(String),
    TimerSet(StateTimer),
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
    travelling_messages: HashMap<String, Rc<RefCell<StateMessage>>>,
    all_messages: HashMap<String, Rc<RefCell<StateMessage>>>,
    event_queue: VecDeque<EventQueueItem>,
    current_time: f64,
    last_updated: f64,
    paused: bool,
    global_speed: f32,
    last_clicked: f64,
    selected_node: Option<String>,
    selected_mouse_position: Vec2,
    hovered_timer: Option<StateTimer>,
    ui_data: UIData,
}

impl State {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            travelling_messages: HashMap::new(),
            all_messages: HashMap::new(),
            event_queue: VecDeque::new(),
            current_time: 0.0,
            last_updated: 0.0,
            paused: false,
            global_speed: 1.0,
            last_clicked: -1.,
            selected_node: None,
            selected_mouse_position: Vec2::new(0., 0.),
            hovered_timer: None,
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
            sent_messages: Vec::new(),
            received_messages: Vec::new(),
            timers: VecDeque::new(),
            free_timer_slots: (0..TIMERS_MAX_NUMBER).collect(),
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
        let msg = StateMessage {
            id: id.clone(),
            pos: self.nodes.get(from).unwrap().borrow().pos,
            from: Rc::clone(self.nodes.get(from).unwrap()),
            to: Rc::clone(self.nodes.get(to).unwrap()),
            status: MessageStatus::Queued,
            time_sent: timestamp as f32,
            time_delivered: timestamp as f32 + duration,
            data,
            drop: duration <= 0.0,
        };
        self.all_messages
            .insert(id.clone(), Rc::new(RefCell::new(msg)));
        self.event_queue.push_back(EventQueueItem {
            timestamp: timestamp,
            event: StateEvent::SendMessage(id.clone()),
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

    pub fn process_timer_set(
        &mut self,
        id: String,
        time_set: f64,
        node_id: String,
        delay: f64,
        time_removed: f64,
    ) {
        let timer = StateTimer {
            id,
            time_set,
            node_id,
            delay,
            time_removed,
            k: -1,
        };
        self.event_queue.push_back(EventQueueItem {
            timestamp: time_set,
            event: StateEvent::TimerSet(timer),
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
        for (_, node) in &mut self.nodes {
            node.borrow_mut().update(self.current_time);
        }
        for (_, msg) in &mut self.travelling_messages {
            msg.borrow_mut()
                .update(self.global_speed, self.current_time as f32);
            msg.borrow_mut().update_status();
            let im_msg = msg.borrow();
            if im_msg.is_delivered() && !im_msg.drop {
                im_msg
                    .to
                    .borrow_mut()
                    .received_messages
                    .push(im_msg.id.clone());
            }
        }
        self.travelling_messages
            .retain(|_, msg| !msg.borrow().is_delivered());
    }

    pub fn draw(&mut self) {
        for (_, node) in &self.nodes {
            node.borrow().draw(self.current_time);
        }
        for (_, msg) in &self.travelling_messages {
            msg.borrow().draw();
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
            self.global_speed += GLOBAL_SPEED_DELTA;
        }
        if is_key_down(KeyCode::Down) {
            if self.global_speed - GLOBAL_SPEED_DELTA > 0.0 {
                self.global_speed -= GLOBAL_SPEED_DELTA;
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
        for (_, node) in &self.nodes {
            self.hovered_timer = node.borrow().check_for_hovered_timer();
            if self.hovered_timer.is_some() {
                break;
            }
        }
    }

    pub fn get_msg_by_mouse_pos(&mut self, mouse_pos: (f32, f32)) -> Option<String> {
        for (_, msg) in &self.travelling_messages {
            if calc_dist(Vec2::new(mouse_pos.0, mouse_pos.1), msg.borrow().pos) < MESSAGE_RADIUS {
                return Some(msg.borrow().id.clone());
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
            if let Some(timer) = &self.hovered_timer {
                egui::Window::new(format!("Timer"))
                    .default_pos(mouse_position())
                    .show(egui_ctx, |ui| {
                        ui.label(format!("Id: {}", timer.id));
                        ui.label(format!("Timer delay: {}", timer.delay));
                        ui.label(format!("Time set: {}", timer.time_set));
                        ui.label(format!("Time removed: {}", timer.time_removed));
                    });
            }
            for (node_id, show_window) in &mut self.ui_data.show_node_windows {
                let node = self.nodes.get(node_id).unwrap().borrow();
                egui::Window::new(format!("Node {}", node_id))
                    .open(show_window)
                    .show(egui_ctx, |ui| {
                        ui.label(format!(
                            "Status: {}",
                            if node.alive { "Alive" } else { "Crashed" }
                        ));
                        ui.collapsing("Sent messages", |ui| {
                            ui.set_max_height(screen_height() * 0.3);
                            ScrollArea::vertical().show(ui, |ui| {
                                for msg_id in &node.sent_messages {
                                    let msg = self.all_messages.get(msg_id).unwrap().borrow();
                                    ui.label(format!("Message {}", msg.id));
                                    ui.label(format!("To: {}", msg.to.borrow().id));
                                    ui.label(format!("Sent at: {}", msg.time_sent));
                                    ui.label(format!("Status: {:?}", msg.status));
                                    ui.label(format!("Data: {}", msg.data));
                                    ui.separator();
                                }
                            });
                            ui.set_max_height(f32::INFINITY);
                        });
                        ui.collapsing("Received messages", |ui| {
                            ui.set_max_height(screen_height() * 0.3);
                            ScrollArea::vertical().show(ui, |ui| {
                                for msg_id in &node.received_messages {
                                    let msg = self.all_messages.get(msg_id).unwrap().borrow();
                                    ui.label(format!("Message {}", msg.id));
                                    ui.label(format!("From: {}", msg.from.borrow().id));
                                    ui.label(format!("Received at: {}", msg.time_delivered));
                                    ui.label(format!("Data: {}", msg.data));
                                    ui.separator();
                                }
                            });
                            ui.set_max_height(f32::INFINITY);
                        });
                    });
            }
            for (msg_id, show_window) in &mut self.ui_data.show_msg_windows {
                if !self.travelling_messages.contains_key(msg_id) {
                    continue;
                }
                let msg = self.travelling_messages.get(msg_id).unwrap().borrow();
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
            StateEvent::SendMessage(msg_id) => {
                self.travelling_messages.insert(
                    msg_id.clone(),
                    Rc::clone(self.all_messages.get(&msg_id).unwrap()),
                );
                let mut msg = self.all_messages.get_mut(&msg_id).unwrap().borrow_mut();
                msg.update_start_pos();
                msg.set_status(MessageStatus::OnTheWay);
                msg.from.borrow_mut().sent_messages.push(msg.id.clone());
            }
            StateEvent::NodeDown(id) => self.nodes.get_mut(&id).unwrap().borrow_mut().make_dead(),
            StateEvent::NodeUp(id) => self.nodes.get_mut(&id).unwrap().borrow_mut().make_alive(),
            StateEvent::TimerSet(timer) => self
                .nodes
                .get_mut(&timer.node_id)
                .unwrap()
                .borrow_mut()
                .timers
                .push_back(timer),
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct StateNode {
    id: String,
    pos: Vec2,
    alive: bool,
    sent_messages: Vec<String>,
    received_messages: Vec<String>,
    timers: VecDeque<StateTimer>,
    free_timer_slots: VecDeque<usize>,
}

impl StateNode {
    pub fn update_pos(&mut self, new_pos: Vec2) {
        self.pos = new_pos;
    }

    pub fn update(&mut self, current_time: f64) {
        for timer in &mut self.timers {
            if timer.k == -1 {
                if !self.free_timer_slots.is_empty() {
                    timer.k = *self.free_timer_slots.front().unwrap() as i32;
                    self.free_timer_slots.pop_front();
                }
            } else if current_time >= timer.time_removed + 1.5 {
                self.free_timer_slots.push_back(timer.k as usize);
            }
        }
        self.timers
            .retain(|timer| current_time < timer.time_removed + 1.5);
    }

    pub fn check_for_hovered_timer(&self) -> Option<StateTimer> {
        let mut hovered_timer: Option<StateTimer> = None;
        for timer in &self.timers {
            if timer.check_hovered(self.pos) {
                hovered_timer = Some(timer.clone());
                break;
            }
        }
        return hovered_timer;
    }

    pub fn draw(&self, current_time: f64) {
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

        for i in 0..self.timers.len() {
            if self.timers[i].k == -1 {
                break;
            }
            self.timers[i].draw(self.pos, current_time);
        }
    }

    pub fn make_alive(&mut self) {
        self.alive = true;
    }

    pub fn make_dead(&mut self) {
        self.alive = false;
    }
}

#[derive(Debug, Clone)]
pub struct StateTimer {
    id: String,
    time_set: f64,
    node_id: String,
    delay: f64,
    time_removed: f64,
    k: i32,
}

impl StateTimer {
    pub fn get_position(&self, node_pos: Vec2) -> Vec2 {
        let angle = (2.0 * PI / (TIMERS_MAX_NUMBER as f32)) * (self.k as f32);
        return node_pos + Vec2::from_angle(angle as f32) * (NODE_RADIUS + TIMER_RADIUS + 5.);
    }

    pub fn check_hovered(&self, node_pos: Vec2) -> bool {
        let mouse_pos = Vec2::from(mouse_position());
        return calc_dist(self.get_position(node_pos), mouse_pos) <= TIMER_RADIUS;
    }

    pub fn draw(&self, node_pos: Vec2, current_time: f64) {
        let pos = self.get_position(node_pos);
        let mut color = TIMER_COLOR;
        if current_time >= self.time_removed {
            color = if self.time_removed < self.time_set + self.delay {
                CANCELLED_TIMER_COLOR
            } else {
                READY_TIMER_COLOR
            };
        }
        let end_angle =
            ((current_time - self.time_set) * 2. * (PI as f64) / self.delay) as f32 - PI / 2.;
        draw_circle_segment(
            pos.x,
            pos.y,
            TIMER_RADIUS,
            -PI / 2.,
            end_angle as f32,
            color,
        );
        draw_circle_lines(pos.x, pos.y, TIMER_RADIUS, 2., color)
    }
}

#[derive(Clone, Debug)]
pub enum MessageStatus {
    Queued,
    OnTheWay,
    Dropped,
    Delivered,
}

#[derive(Debug, Clone)]
pub struct StateMessage {
    id: String,
    pos: Vec2,
    from: Rc<RefCell<StateNode>>,
    to: Rc<RefCell<StateNode>>,
    status: MessageStatus,
    time_sent: f32,
    time_delivered: f32,
    data: String,
    drop: bool,
}

impl StateMessage {
    pub fn update(&mut self, global_speed: f32, current_time: f32) {
        let direction = self.to.borrow().pos - self.pos;
        let travel_time_left = self.time_delivered - current_time;
        let mut own_speed = if !self.drop {
            1.0 / (FPS * travel_time_left / direction.length())
        } else {
            1.0 / (FPS * 3.0 / direction.length())
        };
        if own_speed < 0. {
            own_speed = MAX_MESSAGE_SPEED;
        }
        self.pos += direction.normalize() * own_speed * global_speed;
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

    pub fn update_start_pos(&mut self) {
        self.pos = self.from.borrow().pos;
    }

    pub fn update_status(&mut self) {
        if self.is_delivered() {
            self.status = if self.drop {
                MessageStatus::Dropped
            } else {
                MessageStatus::Delivered
            };
        }
    }

    pub fn set_status(&mut self, status: MessageStatus) {
        self.status = status;
    }
}
