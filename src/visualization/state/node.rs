use std::collections::VecDeque;

use macroquad::prelude::*;

use crate::visualization::utilities::*;

use super::{local_message::StateLocalMessage, timer::*};

#[derive(Debug, Clone)]
pub struct StateNode {
    pub id: String,
    pub pos: Vec2,
    pub alive: bool,
    pub local_messages_sent: Vec<StateLocalMessage>,
    pub local_messages_received: Vec<StateLocalMessage>,
    pub messages_sent: Vec<String>,
    pub messages_received: Vec<String>,
    pub timers: VecDeque<StateTimer>,
    pub free_timer_slots: VecDeque<usize>,
    pub show: bool,
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

    pub fn draw(&self, show_events: bool, current_time: f64, show_timers: bool) {
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

        if show_events && show_timers {
            for i in 0..self.timers.len() {
                if self.timers[i].k == -1 {
                    break;
                }
                self.timers[i].draw(self.pos, current_time);
            }
        }
    }

    pub fn make_alive(&mut self) {
        self.alive = true;
    }

    pub fn make_dead(&mut self) {
        self.alive = false;
    }
}
