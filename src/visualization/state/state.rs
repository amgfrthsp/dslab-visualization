use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    f32::consts::PI,
    rc::Rc,
};

use egui::{Checkbox, Context, ScrollArea, Slider};
use macroquad::prelude::*;
use serde_json::Value;

use crate::visualization::utilities::*;

use super::local_message::*;
use super::message::*;
use super::node::*;
use super::timer::*;

#[derive(Clone, Debug)]
pub enum StateEvent {
    NodeStarted(String),
    MessageSent(String),
    LocalMessageSent(String),
    LocalMessageReceived(String),
    NodeConnected(String),
    NodeDisconnected(String),
    TimerSet(StateTimer),
    LinkDisabled((String, String)),
    LinkEnabled((String, String)),
    DropIncoming(String),
    PassIncoming(String),
    DropOutgoing(String),
    PassOutgoing(String),
    NetworkPartition((Vec<String>, Vec<String>)),
    NetworkReset(),
    NodeStateUpdated((String, String)),
}

#[derive(Clone)]
pub struct EventQueueItem {
    time: f64,
    event: StateEvent,
}

#[derive(Clone)]
pub struct UIData {
    ordered_nodes: Vec<String>,
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
    node_colors: VecDeque<Color>,
    drop_outgoing: HashSet<String>,
    drop_incoming: HashSet<String>,
    disabled_links: HashSet<(String, String)>,
    partition: Option<(Vec<String>, Vec<String>)>,
    prev_screen_size: (f32, f32),
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
            global_speed: DEFAULT_GLOBAL_SPEED,
            ui_data: UIData {
                ordered_nodes: Vec::new(),
                show_events_for_node: HashMap::new(),
                show_node_windows: HashMap::new(),
                show_msg_windows: HashMap::new(),
                last_clicked: -1.,
                selected_node: None,
                selected_mouse_position: Vec2::new(0., 0.),
                hovered_timer: None,
                show_timers: false,
            },
            drop_outgoing: HashSet::new(),
            drop_incoming: HashSet::new(),
            disabled_links: HashSet::new(),
            partition: None,
            prev_screen_size: (0., 0.),
            node_colors: VecDeque::from([
                ORANGE, YELLOW, GREEN, SKYBLUE, BLUE, PURPLE, GOLD, LIGHTGRAY, PINK, LIME, VIOLET,
                WHITE, MAGENTA,
            ]),
        }
    }

    pub fn process_node_started(&mut self, time: f64, name: String, id: u32, pos: Vec2) {
        let color = self.node_colors.pop_front().unwrap_or(DEFAULT_NODE_COLOR);
        let node = StateNode {
            name,
            id,
            pos,
            connected: true,
            state: String::from(""),
            local_messages_sent: Vec::new(),
            local_messages_received: Vec::new(),
            messages_sent: Vec::new(),
            messages_received: Vec::new(),
            timers: VecDeque::new(),
            free_timer_slots: (0..TIMERS_MAX_NUMBER).collect(),
            color,
            show: false,
        };
        self.ui_data
            .show_events_for_node
            .insert(node.name.clone(), true);
        self.ui_data.ordered_nodes.push(node.name.clone());
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::NodeStarted(node.name.clone()),
        });
        self.nodes
            .insert(node.name.clone(), Rc::new(RefCell::new(node)));
    }

    pub fn process_message_sent(
        &mut self,
        id: String,
        time: f64,
        src: &str,
        dest: &str,
        tip: String,
        data: String,
        duration: f32,
        copies_received: u64,
    ) {
        if self.global_speed == DEFAULT_GLOBAL_SPEED && duration > 0. {
            self.global_speed = duration / 10.;
        }

        let src_node = self.nodes.get(src).unwrap();
        let msg = StateMessage::new(
            id.clone(),
            Rc::clone(src_node),
            Rc::clone(self.nodes.get(dest).unwrap()),
            tip,
            data,
            MessageStatus::Queued,
            time as f32,
            time as f32 + duration,
            copies_received,
        );
        self.messages.insert(id.clone(), Rc::new(RefCell::new(msg)));
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::MessageSent(id),
        });
    }

    pub fn process_local_message(
        &mut self,
        time: f64,
        id: String,
        node: String,
        data: String,
        is_sent: bool,
    ) {
        let msg_type: LocalMessageType;
        let event: StateEvent;

        if is_sent {
            msg_type = LocalMessageType::Sent;
            event = StateEvent::LocalMessageSent(id.clone());
        } else {
            msg_type = LocalMessageType::Received;
            event = StateEvent::LocalMessageReceived(id.clone());
        }
        let msg = StateLocalMessage {
            id: id.clone(),
            time,
            node,
            data,
            msg_type,
        };
        self.local_messages.insert(id, msg);

        self.event_queue.push_back(EventQueueItem { time, event });
    }

    pub fn process_node_disconnected(&mut self, time: f64, node: String) {
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::NodeDisconnected(node),
        });
    }

    pub fn process_node_connected(&mut self, time: f64, node: String) {
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::NodeConnected(node),
        });
    }

    pub fn process_timer_set(
        &mut self,
        id: String,
        name: String,
        time_set: f64,
        node: String,
        delay: f64,
        time_removed: f64,
    ) {
        let timer = StateTimer::new(id, name, time_set, node, delay, time_removed);
        self.event_queue.push_back(EventQueueItem {
            time: time_set,
            event: StateEvent::TimerSet(timer),
        });
    }

    pub fn process_link_disabled(&mut self, time: f64, from: String, to: String) {
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::LinkDisabled((from, to)),
        });
    }

    pub fn process_link_enabled(&mut self, time: f64, from: String, to: String) {
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::LinkEnabled((from, to)),
        });
    }

    pub fn process_drop_incoming(&mut self, time: f64, node: String) {
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::DropIncoming(node),
        });
    }

    pub fn process_pass_incoming(&mut self, time: f64, node: String) {
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::PassIncoming(node),
        });
    }

    pub fn process_drop_outgoing(&mut self, time: f64, node: String) {
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::DropOutgoing(node),
        });
    }

    pub fn process_pass_outgoing(&mut self, time: f64, node: String) {
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::PassOutgoing(node),
        });
    }

    pub fn process_network_partition(
        &mut self,
        time: f64,
        group1: Vec<String>,
        group2: Vec<String>,
    ) {
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::NetworkPartition((group1, group2)),
        });
    }

    pub fn process_network_reset(&mut self, time: f64) {
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::NetworkReset(),
        });
    }

    pub fn process_state_updated(&mut self, time: f64, node: String, node_state: String) {
        self.event_queue.push_back(EventQueueItem {
            time,
            event: StateEvent::NodeStateUpdated((node, node_state)),
        });
    }

    pub fn update(&mut self) {
        let screen_size = (screen_width(), screen_height());
        if screen_size != self.prev_screen_size {
            for (_, msg) in &mut self.travelling_messages {
                let old_pos = msg.borrow().src.borrow().pos;
                msg.borrow_mut().pos -= old_pos;
            }
            if self.partition.is_some() {
                self.partition_nodes();
            } else {
                let center = Vec2::from((screen_width() / 2., screen_height() / 2.));
                self.make_node_circle(self.ui_data.ordered_nodes.clone(), center, CIRCLE_RADIUS);
            }
            for (_, msg) in &mut self.travelling_messages {
                let new_pos = msg.borrow().src.borrow().pos;
                msg.borrow_mut().pos += new_pos;
            }
        }
        self.prev_screen_size = screen_size;
        self.check_keyboard_events();

        if self.paused {
            self.last_updated = get_time();
            return;
        } else {
            self.current_time += (get_time() - self.last_updated) * (self.global_speed as f64);
            self.last_updated = get_time();
        }

        while let Some(event) = self.event_queue.front() {
            if self.process_event(event.time, event.event.clone()) {
                self.event_queue.pop_front();
            } else {
                break;
            }
        }
        for (_, node) in &mut self.nodes {
            node.borrow_mut().update(self.current_time);
        }
        self.travelling_messages.retain(|id, msg_ref| {
            let mut msg = msg_ref.borrow_mut();
            msg.update(self.global_speed, self.current_time as f32);
            msg.update_status(self.current_time as f32);
            if msg.status == MessageStatus::Delivered {
                for _ in 0..msg.copies_received {
                    msg.dest.borrow_mut().messages_received.push(id.clone());
                }
                false
            } else if msg.status == MessageStatus::Dropped {
                false
            } else {
                true
            }
        });
    }

    pub fn draw(&mut self) {
        if self.partition.is_some() {
            let start = (screen_width() / 2., 0.);
            let end = (screen_width() / 2., screen_height());
            draw_line(start.0, start.1, end.0, end.1, 5., LIGHTGRAY);
        }
        for (node_name, node) in &self.nodes {
            let show_events = *self.ui_data.show_events_for_node.get(node_name).unwrap();
            node.borrow()
                .draw(show_events, self.current_time, self.ui_data.show_timers);
        }
        for (_, msg) in &mut self.travelling_messages {
            let mut msg_borrowed = msg.borrow_mut();
            let src = &msg_borrowed.src.borrow().name.clone();
            let dest = &msg_borrowed.dest.borrow().name.clone();
            let show_message = *self.ui_data.show_events_for_node.get(src).unwrap()
                || *self.ui_data.show_events_for_node.get(dest).unwrap();
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
        if is_key_pressed(KeyCode::Right) && !self.event_queue.is_empty() {
            let new_current_time = self.event_queue.front().unwrap().time - 0.01;
            let delta = self.current_time - new_current_time;
            for (_, msg) in &self.travelling_messages {
                msg.borrow_mut().update_with_jump(
                    self.global_speed,
                    self.current_time as f32,
                    delta as f32,
                );
            }
            self.current_time = new_current_time;
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
                if let Some(node) = self.get_node_by_mouse_pos(mouse_pos) {
                    self.ui_data.selected_node = Some(node);
                    self.ui_data.selected_mouse_position = Vec2::new(mouse_pos.0, mouse_pos.1);
                }
            } else {
                let node = self.ui_data.selected_node.clone().unwrap();
                let node = self.nodes.get_mut(&node).unwrap();
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
        None
    }

    pub fn get_node_by_mouse_pos(&mut self, mouse_pos: (f32, f32)) -> Option<String> {
        for (_, node) in &self.nodes {
            if calc_dist(Vec2::new(mouse_pos.0, mouse_pos.1), node.borrow().pos) < NODE_RADIUS {
                return Some(node.borrow().name.clone());
            }
        }
        None
    }

    pub fn draw_ui(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            self.draw_ui_config_window(egui_ctx);
            self.draw_ui_hovered_timer(egui_ctx);
            self.draw_ui_node_windows(egui_ctx);
            self.draw_ui_msg_windows(egui_ctx);
            self.draw_ui_network_window(egui_ctx);
        });
    }

    pub fn draw_ui_config_window(&mut self, egui_ctx: &Context) {
        egui::Window::new("Config").show(egui_ctx, |ui| {
            let next_event_at;
            if self.event_queue.is_empty() {
                next_event_at = "--".to_owned();
            } else {
                next_event_at = format!("{:.4}", self.event_queue.front().unwrap().time);
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
                    for node in &self.ui_data.ordered_nodes {
                        let show_events = self.ui_data.show_events_for_node.get_mut(node).unwrap();
                        let node = self.nodes.get(node).unwrap().borrow().name.clone();
                        ui.add(Checkbox::new(show_events, format!("Node {}", node)));
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
                    ui.label(format!("Name: {}", timer.name));
                    ui.label(format!("Timer delay: {}", timer.delay));
                    ui.label(format!("Time set: {}", timer.time_set));
                    ui.label(format!("Time removed: {}", timer.time_removed));
                });
        }
    }

    pub fn draw_ui_node_windows(&mut self, egui_ctx: &Context) {
        for (node, show_window) in &mut self.ui_data.show_node_windows {
            let node = self.nodes.get(node).unwrap().borrow();
            egui::Window::new(format!("Node {}", node.name))
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
                    ui.collapsing("State", |ui| {
                        ui.set_max_height(screen_height() * 0.3);
                        ScrollArea::vertical().show(ui, |ui| {
                            ui.label(format!("{}", node.state));
                        });
                        ui.set_max_height(f32::INFINITY);
                    });
                    ui.collapsing("Sent local messages", |ui| {
                        ui.set_max_height(screen_height() * 0.3);
                        ScrollArea::vertical().show(ui, |ui| {
                            for msg in &node.local_messages_sent {
                                ui.label(format!("Message {}", msg.id));
                                ui.label(format!("Sent at: {}", msg.time));
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
                                ui.label(format!("Received at: {}", msg.time));
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
                                ui.label(format!("To: {}", msg.dest.borrow().name));
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
                                ui.label(format!("From: {}", msg.src.borrow().name));
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
                                ui.label(format!("Timer {}", timer.name));
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
                    ui.label(format!("From: {}", msg.src.borrow().id));
                    ui.label(format!("To: {}", msg.dest.borrow().id));
                    if msg.is_duplicated() {
                        ui.label(format!("Duplicated {} times", msg.copies_received));
                    }
                    ui.label(format!("Type: {}", msg.tip));
                    ui.label(format!("Data: {}", msg.data.clone()));
                });
        }
    }

    pub fn draw_ui_network_window(&mut self, egui_ctx: &Context) {
        egui::Window::new("Network")
            .default_pos((screen_width() * 0.8, 15.))
            .show(egui_ctx, |ui| {
                ui.set_max_height(screen_height() * 0.5);
                ScrollArea::vertical().show(ui, |ui| {
                    ui.strong("\nDrop incoming:");
                    ui.label(format!("{:?}", self.drop_incoming));
                    ui.strong("Drop outgoing:");
                    ui.label(format!("{:?}", self.drop_outgoing));
                    ui.strong("Partition:");
                    if self.partition.is_none() {
                        ui.label("---");
                    } else {
                        let pair = self.partition.clone().unwrap();
                        ui.label(format!("{:?} -x- {:?}", pair.0, pair.1));
                    }
                    ui.strong("Disabled links:");
                    let mut shown: HashSet<(String, String)> = HashSet::new();
                    for (from, to) in &self.disabled_links {
                        let pair = (to.to_string(), from.to_string());
                        if self.disabled_links.contains(&pair) && !shown.contains(&pair) {
                            shown.insert(pair);
                            ui.label(format!("{} <-xx-> {}", from, to));
                        } else {
                            ui.label(format!("{} -xx-> {}", from, to));
                        }
                    }
                });
                ui.set_max_height(f32::INFINITY);
            });
    }

    pub fn process_event(&mut self, time: f64, event: StateEvent) -> bool {
        if self.current_time < time {
            return false;
        }
        match event {
            StateEvent::NodeStarted(node) => {
                self.nodes.get_mut(&node).unwrap().borrow_mut().show = true;
            }
            StateEvent::MessageSent(msg_id) => {
                let msg = self.messages.get(&msg_id).unwrap().clone();

                msg.borrow()
                    .src
                    .borrow_mut()
                    .messages_sent
                    .push(msg_id.clone());
                let src = msg.borrow().src.borrow().name.clone();
                let dest = msg.borrow().dest.borrow().name.clone();
                if self.drop_incoming.contains(&dest)
                    || self.drop_incoming.contains(&src)
                    || self.disabled_links.contains(&(src, dest))
                {
                    msg.borrow_mut().set_status(MessageStatus::Dropped);
                } else {
                    msg.borrow_mut().update_start_pos();
                    msg.borrow_mut().set_status(MessageStatus::OnTheWay);
                    self.travelling_messages.insert(msg_id, msg);
                }
            }
            StateEvent::NodeDisconnected(node) => {
                self.nodes.get_mut(&node).unwrap().borrow_mut().connected = false
            }
            StateEvent::NodeConnected(node) => {
                self.nodes.get_mut(&node).unwrap().borrow_mut().connected = true
            }
            StateEvent::TimerSet(timer) => self
                .nodes
                .get_mut(&timer.node)
                .unwrap()
                .borrow_mut()
                .timers
                .push_back(timer),
            StateEvent::LocalMessageSent(id) => {
                let msg = self.local_messages.remove(&id).unwrap();
                self.nodes
                    .get_mut(&msg.node)
                    .unwrap()
                    .borrow_mut()
                    .local_messages_sent
                    .push(msg);
            }
            StateEvent::LocalMessageReceived(id) => {
                let msg = self.local_messages.remove(&id).unwrap();
                self.nodes
                    .get_mut(&msg.node)
                    .unwrap()
                    .borrow_mut()
                    .local_messages_received
                    .push(msg.clone());
            }
            StateEvent::LinkDisabled((from, to)) => {
                self.disabled_links.insert((from, to));
            }
            StateEvent::LinkEnabled((from, to)) => {
                self.disabled_links.remove(&(from, to));
            }
            StateEvent::DropIncoming(node) => {
                self.drop_incoming.insert(node);
            }
            StateEvent::PassIncoming(node) => {
                self.drop_incoming.remove(&node);
            }
            StateEvent::DropOutgoing(node) => {
                self.drop_outgoing.insert(node);
            }
            StateEvent::PassOutgoing(node) => {
                self.drop_outgoing.remove(&node);
            }
            StateEvent::NetworkPartition((group1, group2)) => {
                for node1 in &group1 {
                    for node2 in &group2 {
                        self.disabled_links.insert((node1.clone(), node2.clone()));
                        self.disabled_links.insert((node2.clone(), node1.clone()));
                    }
                }
                self.partition = Some((group1, group2));
                self.partition_nodes();
            }
            StateEvent::NetworkReset() => {
                self.partition = None;
                self.drop_incoming.clear();
                self.drop_outgoing.clear();
                self.disabled_links.clear();
                let center = Vec2::new(screen_width() / 2., screen_height() / 2.);
                self.make_node_circle(self.ui_data.ordered_nodes.clone(), center, CIRCLE_RADIUS);
            }
            StateEvent::NodeStateUpdated((node, node_state)) => {
                let value: Value = serde_json::from_str(&node_state).unwrap();
                let pretty_state = serde_json::to_string_pretty(&value).unwrap();
                self.nodes.get_mut(&node).unwrap().borrow_mut().state = pretty_state;
            }
        }
        true
    }

    pub fn partition_nodes(&mut self) {
        if self.partition.is_none() {
            return;
        }
        let (group1, group2) = self.partition.clone().unwrap();
        let left = Vec2::new(screen_width() / 4., screen_height() / 2.);
        let right = Vec2::new(screen_width() * 3. / 4., screen_height() / 2.);
        self.make_node_circle(group1, left, PARTITIONED_CIRCLE_RADIUS);
        self.make_node_circle(group2, right, PARTITIONED_CIRCLE_RADIUS);
    }

    pub fn make_node_circle(&mut self, nodes: Vec<String>, center: Vec2, circle_radius: f32) {
        for i in 0..nodes.len() {
            let angle = (2.0 * PI / (nodes.len() as f32)) * (i as f32);
            let pos = center + Vec2::from_angle(angle as f32) * circle_radius;
            self.nodes
                .get_mut(&nodes[i])
                .unwrap()
                .borrow_mut()
                .update_pos(pos);
        }
    }
}
