use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use egui::{Checkbox, Context, ScrollArea, Slider};
use macroquad::prelude::*;

use crate::visualization::utilities::*;

use super::local_message::*;
use super::message::*;
use super::node::*;
use super::timer::*;

#[derive(Clone, Debug)]
pub enum StateEvent {
    AddNode(String),
    SendMessage(String),
    SendLocalMessage(String),
    ReceiveLocalMessage(String),
    NodeConnected(String),
    NodeDisconnected(String),
    TimerSet(StateTimer),
}

#[derive(Clone)]
pub struct EventQueueItem {
    timestamp: f64,
    event: StateEvent,
}

#[derive(Clone)]
pub struct UIData {
    ordered_node_ids: Vec<String>,
    show_events_for_node: HashMap<String, bool>,
    show_node_windows: HashMap<String, bool>,
    show_msg_windows: HashMap<String, bool>,
    last_clicked: f64,
    selected_node: Option<String>,
    selected_mouse_position: Vec2,
    hovered_timer: Option<StateTimer>,
    show_timers: bool,
}

pub struct State {
    nodes: HashMap<String, Rc<RefCell<StateNode>>>,
    travelling_messages: HashMap<String, Rc<RefCell<StateMessage>>>,
    messages: HashMap<String, Rc<RefCell<StateMessage>>>,
    local_messages: HashMap<String, StateLocalMessage>,
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
            travelling_messages: HashMap::new(),
            messages: HashMap::new(),
            local_messages: HashMap::new(),
            event_queue: VecDeque::new(),
            current_time: 0.0,
            last_updated: 0.0,
            paused: false,
            global_speed: 0.01,
            ui_data: UIData {
                ordered_node_ids: Vec::new(),
                show_events_for_node: HashMap::new(),
                show_node_windows: HashMap::new(),
                show_msg_windows: HashMap::new(),
                last_clicked: -1.,
                selected_node: None,
                selected_mouse_position: Vec2::new(0., 0.),
                hovered_timer: None,
                show_timers: true,
            },
        }
    }

    pub fn add_node(&mut self, timestamp: f64, id: String, pos: Vec2) {
        let node = StateNode {
            id: id.clone(),
            pos: pos,
            connected: true,
            local_messages_sent: Vec::new(),
            local_messages_received: Vec::new(),
            messages_sent: Vec::new(),
            messages_received: Vec::new(),
            timers: VecDeque::new(),
            free_timer_slots: (0..TIMERS_MAX_NUMBER).collect(),
            show: false,
        };
        self.ui_data
            .show_events_for_node
            .insert(node.id.clone(), true);
        self.ui_data.ordered_node_ids.push(node.id.clone());
        self.event_queue.push_back(EventQueueItem {
            timestamp,
            event: StateEvent::AddNode(node.id.clone()),
        });
        self.nodes
            .insert(node.id.clone(), Rc::new(RefCell::new(node)));
    }

    pub fn send_message(
        &mut self,
        id: String,
        timestamp: f64,
        src: &str,
        dest: &str,
        tip: String,
        data: String,
        duration: f32,
    ) {
        let src_node = self.nodes.get(src).unwrap();
        let msg = StateMessage {
            id: id.clone(),
            pos: src_node.borrow().pos,
            src: Rc::clone(src_node),
            dest: Rc::clone(self.nodes.get(dest).unwrap()),
            tip,
            data,
            status: MessageStatus::Queued,
            time_sent: timestamp as f32,
            time_delivered: timestamp as f32 + duration,
            drop: duration <= 0.0,
        };
        self.messages.insert(id.clone(), Rc::new(RefCell::new(msg)));
        self.event_queue.push_back(EventQueueItem {
            timestamp: timestamp,
            event: StateEvent::SendMessage(id),
        });
    }

    pub fn process_local_message(
        &mut self,
        timestamp: f64,
        id: String,
        node_id: String,
        data: String,
        is_sent: bool,
    ) {
        let msg_type: LocalMessageType;
        let event: StateEvent;

        if is_sent {
            msg_type = LocalMessageType::Sent;
            event = StateEvent::SendLocalMessage(id.clone());
        } else {
            msg_type = LocalMessageType::Received;
            event = StateEvent::ReceiveLocalMessage(id.clone());
        }
        let msg = StateLocalMessage {
            id: id.clone(),
            timestamp,
            node_id,
            data,
            msg_type,
        };
        self.local_messages.insert(id, msg);

        self.event_queue.push_back(EventQueueItem {
            timestamp: timestamp,
            event: event,
        });
    }

    pub fn process_node_disconnected(&mut self, timestamp: f64, id: String) {
        self.event_queue.push_back(EventQueueItem {
            timestamp: timestamp,
            event: StateEvent::NodeDisconnected(id),
        });
    }

    pub fn process_node_connected(&mut self, timestamp: f64, id: String) {
        self.event_queue.push_back(EventQueueItem {
            timestamp: timestamp,
            event: StateEvent::NodeConnected(id),
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
            let mut mut_msg = msg.borrow_mut();
            mut_msg.update(self.global_speed, self.current_time as f32);
            mut_msg.update_status();
            if mut_msg.is_delivered() && !mut_msg.drop {
                mut_msg
                    .dest
                    .borrow_mut()
                    .messages_received
                    .push(mut_msg.id.clone());
            }
        }
        self.travelling_messages.retain(|_, msg| {
            let msg_borrow = msg.borrow();
            if !msg_borrow.is_delivered()
                && (!msg_borrow.drop && self.current_time < msg_borrow.time_delivered.into())
            {
                return true;
            }
            false
        });
    }

    pub fn draw(&mut self) {
        for (node_id, node) in &self.nodes {
            let show_events = *self.ui_data.show_events_for_node.get(node_id).unwrap();
            node.borrow()
                .draw(show_events, self.current_time, self.ui_data.show_timers);
        }
        for (_, msg) in &self.travelling_messages {
            let msg_borrowed = msg.borrow();
            let src_id = &msg_borrowed.src.borrow().id;
            let dest_id = &msg_borrowed.dest.borrow().id;
            let show_message = *self.ui_data.show_events_for_node.get(src_id).unwrap()
                || *self.ui_data.show_events_for_node.get(dest_id).unwrap();
            if show_message {
                msg_borrowed.draw();
            }
        }
        self.draw_time();
    }

    pub fn draw_time(&self) {
        draw_text_ex(
            &format!("Time: {:.5}", self.current_time),
            screen_width() * 0.03,
            screen_height() * 0.96,
            TextParams {
                font_size: (screen_width() / 18.0).floor() as u16,
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
            self.global_speed = f32::max(0.0, self.global_speed - GLOBAL_SPEED_DELTA);
        }
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = mouse_position();
            if self.ui_data.selected_node.is_none() {
                if let Some(node_id) = self.get_node_by_mouse_pos(mouse_pos) {
                    self.ui_data.selected_node = Some(node_id);
                    self.ui_data.selected_mouse_position = Vec2::new(mouse_pos.0, mouse_pos.1);
                }
            } else {
                let node_id = self.ui_data.selected_node.clone().unwrap();
                let node = self.nodes.get_mut(&node_id).unwrap();
                let drag_direction =
                    Vec2::new(mouse_pos.0, mouse_pos.1) - self.ui_data.selected_mouse_position;
                if !drag_direction.is_nan() {
                    let new_pos = node.borrow().pos + drag_direction;
                    node.borrow_mut().update_pos(new_pos);
                }
                self.ui_data.selected_mouse_position = Vec2::new(mouse_pos.0, mouse_pos.1);
            }

            if let Some(msg_id) = self.get_msg_by_mouse_pos(mouse_pos) {
                self.ui_data.show_msg_windows.insert(msg_id, true);
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            self.ui_data.last_clicked = self.current_time;
        }
        if is_mouse_button_released(MouseButton::Left) {
            if self.current_time - self.ui_data.last_clicked <= SINGLE_CLICK_DELAY
                && self.ui_data.selected_node.is_some()
            {
                self.ui_data
                    .show_node_windows
                    .insert(self.ui_data.selected_node.clone().unwrap(), true);
            }
            self.ui_data.selected_node = None;
        }
        if self.ui_data.show_timers {
            for (_, node) in &self.nodes {
                self.ui_data.hovered_timer = node.borrow().check_for_hovered_timer();
                if self.ui_data.hovered_timer.is_some() {
                    break;
                }
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
            self.draw_ui_config_window(egui_ctx);
            self.draw_ui_hovered_timer(egui_ctx);
            self.draw_ui_node_windows(egui_ctx);
            self.draw_ui_msg_windows(egui_ctx);
        });
    }

    pub fn draw_ui_config_window(&mut self, egui_ctx: &Context) {
        egui::Window::new("Config").show(egui_ctx, |ui| {
            let next_event_at;
            if self.event_queue.is_empty() {
                next_event_at = "--".to_owned();
            } else {
                next_event_at = format!("{:.4}", self.event_queue.front().unwrap().timestamp);
            }
            ui.label(format!("Next event at: {}", next_event_at));
            ui.add(Checkbox::new(&mut self.ui_data.show_timers, "Show timers"));
            ui.add(
                Slider::new(&mut self.global_speed, 0.0000..=1.)
                    .logarithmic(true)
                    .step_by(GLOBAL_SPEED_DELTA as f64)
                    .text("Speed"),
            );
            ui.collapsing("Show events (messages and timers) for a node:", |ui| {
                ui.set_max_height(screen_height() * 0.2);
                ScrollArea::vertical().show(ui, |ui| {
                    for node_id in &self.ui_data.ordered_node_ids {
                        let show_events =
                            self.ui_data.show_events_for_node.get_mut(node_id).unwrap();
                        ui.add(Checkbox::new(show_events, format!("Node {}", node_id)));
                    }
                });
                ui.set_max_height(f32::INFINITY);
            });
        });
    }

    pub fn draw_ui_hovered_timer(&mut self, egui_ctx: &Context) {
        if let Some(timer) = &self.ui_data.hovered_timer {
            egui::Window::new("Timer")
                .default_pos(mouse_position())
                .show(egui_ctx, |ui| {
                    ui.label(format!("Id: {}", timer.id));
                    ui.label(format!("Timer delay: {}", timer.delay));
                    ui.label(format!("Time set: {}", timer.time_set));
                    ui.label(format!("Time removed: {}", timer.time_removed));
                });
        }
    }

    pub fn draw_ui_node_windows(&mut self, egui_ctx: &Context) {
        for (node_id, show_window) in &mut self.ui_data.show_node_windows {
            let node = self.nodes.get(node_id).unwrap().borrow();
            egui::Window::new(format!("Node {}", node_id))
                .open(show_window)
                .show(egui_ctx, |ui| {
                    ui.label(format!(
                        "Status: {}",
                        if node.connected {
                            "Connected"
                        } else {
                            "Disconnected"
                        }
                    ));
                    ui.collapsing("Sent local messages", |ui| {
                        ui.set_max_height(screen_height() * 0.3);
                        ScrollArea::vertical().show(ui, |ui| {
                            for msg in &node.local_messages_sent {
                                ui.label(format!("Message {}", msg.id));
                                ui.label(format!("Sent at: {}", msg.timestamp));
                                ui.label(format!("Data: {}", msg.data));
                                ui.separator();
                            }
                        });
                        ui.set_max_height(f32::INFINITY);
                    });
                    ui.collapsing("Received local messages", |ui| {
                        ui.set_max_height(screen_height() * 0.3);
                        ScrollArea::vertical().show(ui, |ui| {
                            for msg in &node.local_messages_received {
                                ui.label(format!("Message {}", msg.id));
                                ui.label(format!("Received at: {}", msg.timestamp));
                                ui.label(format!("Data: {}", msg.data));
                                ui.separator();
                            }
                        });
                        ui.set_max_height(f32::INFINITY);
                    });
                    ui.collapsing("Sent messages", |ui| {
                        ui.set_max_height(screen_height() * 0.3);
                        ScrollArea::vertical().show(ui, |ui| {
                            for msg_id in &node.messages_sent {
                                let msg = self.messages.get(msg_id).unwrap().borrow();
                                ui.label(format!("Message {}", msg.id));
                                ui.label(format!("To: {}", msg.dest.borrow().id));
                                ui.label(format!("Sent at: {}", msg.time_sent));
                                ui.label(format!("Status: {:?}", msg.status));
                                ui.label(format!("Type: {}", msg.tip));
                                ui.label(format!("Data: {}", msg.data));
                                ui.separator();
                            }
                        });
                        ui.set_max_height(f32::INFINITY);
                    });
                    ui.collapsing("Received messages", |ui| {
                        ui.set_max_height(screen_height() * 0.3);
                        ScrollArea::vertical().show(ui, |ui| {
                            for msg_id in &node.messages_received {
                                let msg = self.messages.get(msg_id).unwrap().borrow();
                                ui.label(format!("Message {}", msg.id));
                                ui.label(format!("From: {}", msg.src.borrow().id));
                                ui.label(format!("Received at: {}", msg.time_delivered));
                                ui.label(format!("Type: {}", msg.tip));
                                ui.label(format!("Data: {}", msg.data));
                                ui.separator();
                            }
                        });
                        ui.set_max_height(f32::INFINITY);
                    });
                    ui.collapsing("Current timers", |ui| {
                        ui.set_max_height(screen_height() * 0.3);
                        ScrollArea::vertical().show(ui, |ui| {
                            for timer in &node.timers {
                                ui.label(format!("Timer {}", timer.id));
                                ui.label(format!("Time set: {}", timer.time_set));
                                ui.label(format!("Delay: {}", timer.delay));
                                ui.separator();
                            }
                        });
                        ui.set_max_height(f32::INFINITY);
                    });
                });
        }
    }

    pub fn draw_ui_msg_windows(&mut self, egui_ctx: &Context) {
        for (msg_id, show_window) in &mut self.ui_data.show_msg_windows {
            if !self.travelling_messages.contains_key(msg_id) {
                continue;
            }
            let msg = self.travelling_messages.get(msg_id).unwrap().borrow();
            egui::Window::new(format!("Message {}", msg_id))
                .open(show_window)
                .show(egui_ctx, |ui| {
                    ui.label(format!("From: {}", msg.src.borrow().id.clone()));
                    ui.label(format!("To: {}", msg.dest.borrow().id.clone()));
                    ui.label(format!("Data: {}", msg.data.clone()));
                });
        }
    }

    pub fn process_event(&mut self, timestamp: f64, event: StateEvent) -> bool {
        if self.current_time < timestamp {
            return false;
        }
        match event {
            StateEvent::AddNode(node_id) => {
                self.nodes.get_mut(&node_id).unwrap().borrow_mut().show = true;
            }
            StateEvent::SendMessage(msg_id) => {
                self.travelling_messages.insert(
                    msg_id.clone(),
                    Rc::clone(self.messages.get(&msg_id).unwrap()),
                );
                let mut msg = self.messages.get_mut(&msg_id).unwrap().borrow_mut();
                msg.update_start_pos();
                msg.set_status(MessageStatus::OnTheWay);
                msg.src.borrow_mut().messages_sent.push(msg.id.clone());
            }
            StateEvent::NodeDisconnected(id) => {
                self.nodes.get_mut(&id).unwrap().borrow_mut().connected = false
            }
            StateEvent::NodeConnected(id) => {
                self.nodes.get_mut(&id).unwrap().borrow_mut().connected = true
            }
            StateEvent::TimerSet(timer) => self
                .nodes
                .get_mut(&timer.node_id)
                .unwrap()
                .borrow_mut()
                .timers
                .push_back(timer),
            StateEvent::SendLocalMessage(id) => {
                let msg = self.local_messages.remove(&id).unwrap();
                self.nodes
                    .get_mut(&msg.node_id)
                    .unwrap()
                    .borrow_mut()
                    .local_messages_sent
                    .push(msg);
            }
            StateEvent::ReceiveLocalMessage(id) => {
                let msg = self.local_messages.remove(&id).unwrap();
                self.nodes
                    .get_mut(&msg.node_id)
                    .unwrap()
                    .borrow_mut()
                    .local_messages_received
                    .push(msg.clone());
            }
        }
        true
    }
}
