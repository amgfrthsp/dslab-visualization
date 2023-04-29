use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::visualization::utilities::*;

#[derive(Debug, Clone)]
pub struct StateTimer {
    pub id: String,
    pub name: String,
    pub time_set: f64,
    pub node: String,
    pub delay: f64,
    pub time_removed: f64,
    pub k: i32,
}

impl StateTimer {
    pub fn get_position(&self, node_pos: Vec2) -> Vec2 {
        let angle = (2.0 * PI / (TIMERS_MAX_NUMBER as f32)) * (self.k as f32);
        node_pos + Vec2::from_angle(angle as f32) * (NODE_RADIUS + TIMER_RADIUS + 5.)
    }

    pub fn check_hovered(&self, node_pos: Vec2) -> bool {
        let mouse_pos = Vec2::from(mouse_position());
        calc_dist(self.get_position(node_pos), mouse_pos) <= TIMER_RADIUS
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
