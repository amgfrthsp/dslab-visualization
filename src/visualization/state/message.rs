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
    pub drop: bool,
}

impl StateMessage {
    pub fn update(&mut self, global_speed: f32, current_time: f32) {
        let direction = self.dest.borrow().pos - self.pos;
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
        let overall_dist = calc_dist(self.src.borrow().pos, self.dest.borrow().pos);
        let color = if self.drop && calc_dist(self.src.borrow().pos, self.pos) >= overall_dist * 0.4
        {
            RED
        } else {
            BLUE
        };
        draw_circle(self.pos.x, self.pos.y, MESSAGE_RADIUS, color);
    }

    pub fn is_delivered(&self) -> bool {
        if !self.drop {
            calc_dist(self.pos, self.dest.borrow().pos) < 5.0
        } else {
            let overall_dist = calc_dist(self.src.borrow().pos, self.dest.borrow().pos);
            calc_dist(self.src.borrow().pos, self.pos) >= overall_dist * 0.7
        }
    }

    pub fn update_start_pos(&mut self) {
        self.pos = self.src.borrow().pos;
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

#[derive(Clone, Debug)]
pub enum MessageStatus {
    Queued,
    OnTheWay,
    Dropped,
    Delivered,
}
