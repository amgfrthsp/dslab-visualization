use std::{cell::RefCell, rc::Rc};

use crate::visualization::utilities::*;
use macroquad::prelude::*;

use super::node::*;

#[derive(Debug, Clone)]
pub struct StateMessage {
    pub id: String,
    pub pos: Vec2,
    pub src: Rc<RefCell<StateNode>>,
    pub dest: Rc<RefCell<StateNode>>,
    pub tip: String,
    pub data: String,
    pub status: MessageStatus,
    pub time_sent: f32,
    pub time_delivered: f32,
    pub copies_received: u64,
    pub last_color_change: f64,
    pub drop_color: Color,
}

impl StateMessage {
    pub fn get_direction(&self) -> Vec2 {
        self.dest.borrow().pos - self.pos
    }

    pub fn get_own_speed(&self, current_time: f32) -> f32 {
        let direction = self.get_direction();
        let travel_time_left = self.time_delivered - current_time;
        let mut own_speed = if !self.is_dropped() {
            1.0 / (FPS * travel_time_left / direction.length())
        } else {
            1.0 / (FPS * 3.0 / direction.length())
        };
        if own_speed < 0. {
            own_speed = MAX_MESSAGE_SPEED;
        }
        own_speed
    }

    pub fn update(&mut self, global_speed: f32, current_time: f32) {
        let direction = self.get_direction();
        let own_speed = self.get_own_speed(current_time);
        self.pos += direction.normalize() * own_speed * global_speed;
    }

    pub fn update_with_jump(&mut self, global_speed: f32, current_time: f32, delta: f32) {
        let direction = self.get_direction();
        let own_speed = self.get_own_speed(current_time);
        let jump_dist = own_speed * global_speed * delta;
        self.pos += direction.normalize() * jump_dist;
    }

    pub fn draw(&mut self) {
        let time = get_time();
        if self.is_dropped() && time - self.last_color_change >= 0.3 {
            self.drop_color = if self.drop_color == BLACK {
                self.src.borrow().color
            } else {
                BLACK
            };
            self.last_color_change = time;
        }
        let color = if self.is_dropped() {
            self.drop_color
        } else {
            self.src.borrow().color
        };
        draw_circle(self.pos.x, self.pos.y, MESSAGE_RADIUS, color);
        if self.is_duplicated() {
            let font_size = (MESSAGE_RADIUS * 2.0).floor() as u16;
            let text = self.copies_received.to_string();
            let text_size = measure_text(&text, None, font_size, 1.0);
            let text_position = Vec2::new(
                self.pos.x - text_size.width / 2.0,
                self.pos.y + text_size.height / 2.0,
            );

            draw_text_ex(
                &text,
                text_position.x,
                text_position.y,
                TextParams {
                    font_size,
                    color: BLACK,
                    ..Default::default()
                },
            );
        }
    }

    pub fn is_dropped(&self) -> bool {
        self.copies_received == 0
    }

    pub fn is_duplicated(&self) -> bool {
        self.copies_received > 1
    }

    pub fn is_delivered(&self, current_time: f32) -> bool {
        if !self.is_dropped() {
            calc_dist(self.pos, self.dest.borrow().pos) < 5.0 || current_time >= self.time_delivered
        } else {
            let overall_dist = calc_dist(self.src.borrow().pos, self.dest.borrow().pos);
            calc_dist(self.src.borrow().pos, self.pos) >= overall_dist * 0.25
        }
    }

    pub fn update_start_pos(&mut self) {
        self.pos = self.src.borrow().pos;
    }

    pub fn update_status(&mut self, current_time: f32) {
        if self.is_delivered(current_time) {
            self.status = if self.is_dropped() && self.src.borrow().id != self.dest.borrow().id {
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

#[derive(Clone, Debug, PartialEq)]
pub enum MessageStatus {
    Queued,
    OnTheWay,
    Dropped,
    Delivered,
}
