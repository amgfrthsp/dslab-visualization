use std::collections::VecDeque;

use macroquad::prelude::*;

use crate::visualization::utilities::*;

use super::{local_message::StateLocalMessage, state::State, timer::*};

#[derive(Debug, Clone)]
pub struct StateNode {
    pub name: String,
    pub id: u32,
    pub relative_pos: Vec2,
    pub connected: bool,
    pub state: String,
    pub local_messages_sent: Vec<StateLocalMessage>,
    pub local_messages_received: Vec<StateLocalMessage>,
    pub messages_sent: Vec<String>,
    pub messages_received: Vec<String>,
    pub timers: VecDeque<StateTimer>,
    pub free_timer_slots: VecDeque<usize>,
    pub color: Color,
    pub show: bool,
}

impl StateNode {
    pub fn new(name: String, id: u32, pos: Vec2, color: Color) -> Self {
        let relative_pos = get_relative_pos(pos);
        Self {
            name,
            id,
            relative_pos,
            color,
            connected: true,
            state: String::from(""),
            local_messages_sent: Vec::new(),
            local_messages_received: Vec::new(),
            messages_sent: Vec::new(),
            messages_received: Vec::new(),
            timers: VecDeque::new(),
            free_timer_slots: (0..TIMERS_MAX_NUMBER).collect(),
            show: false,
        }
    }

    pub fn update_pos(&mut self, new_pos: Vec2) {
        self.relative_pos = get_relative_pos(new_pos);
    }

    pub fn get_pos(&self) -> Vec2 {
        get_absolute_pos(self.relative_pos)
    }

    pub fn update(
        &mut self,
        current_time: f64,
        node_radius: f32,
        timer_radius: f32,
        show_timers: bool,
    ) -> Option<StateTimer> {
        let pos = self.get_pos();
        let mut hovered_timer: Option<StateTimer> = None;
        for timer in &mut self.timers {
            if timer.k == -1 {
                if !self.free_timer_slots.is_empty() {
                    timer.k = *self.free_timer_slots.front().unwrap() as i32;
                    self.free_timer_slots.pop_front();
                }
            } else if current_time >= timer.time_removed {
                self.free_timer_slots.push_back(timer.k as usize);
            }
            if show_timers && timer.check_hovered(pos, node_radius, timer_radius) {
                hovered_timer = Some(timer.clone());
            }
        }
        self.timers
            .retain(|timer| current_time < timer.time_removed);

        hovered_timer
    }

    pub fn draw(&self, state: &State) {
        let pos = self.get_pos();
        draw_circle(
            pos.x,
            pos.y,
            state.get_node_radius(),
            if self.connected {
                self.color
            } else {
                DEAD_NODE_COLOR
            },
        );

        let font_size = (state.get_node_radius() * 2.0).floor() as u16;
        let text_size = measure_text(&self.name, None, font_size, 1.0);
        let text_position = Vec2::new(
            pos.x - text_size.width / 2.0,
            pos.y + text_size.height / 2.0,
        );

        let show_events = *state.ui_data.show_events_for_node.get(&self.name).unwrap();

        if show_events && state.ui_data.show_timers {
            for i in 0..self.timers.len() {
                if self.timers[i].k == -1 {
                    break;
                }
                self.timers[i].draw(pos, state);
            }
        }

        draw_text_ex(
            &self.name,
            text_position.x,
            text_position.y,
            TextParams {
                font_size,
                color: WHITE,
                ..Default::default()
            },
        );
    }
}
