use macroquad::prelude::*;

pub const NODE_RADIUS: f32 = 15.;
pub const MESSAGE_RADIUS: f32 = 5.;
pub const CIRCLE_RADIUS: f32 = 150.;
pub const TIMER_RADIUS: f32 = 8.;
pub const FPS: f32 = 60.0;

pub const ALIVE_NODE_COLOR: Color = YELLOW;
pub const DEAD_NODE_COLOR: Color = RED;

pub const TIMER_COLOR: Color = ORANGE;
pub const READY_TIMER_COLOR: Color = GREEN;
pub const CANCELLED_TIMER_COLOR: Color = RED;

pub const GLOBAL_SPEED_DELTA: f32 = 0.02;
pub const MAX_MESSAGE_SPEED: f32 = 30.;

pub const SINGLE_CLICK_DELAY: f64 = 0.3;

pub const TIMERS_MAX_NUMBER: usize = 9;

pub fn calc_dist(a: Vec2, b: Vec2) -> f32 {
    ((a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)).sqrt()
}

pub fn draw_circle_segment(x: f32, y: f32, r: f32, start_angle: f32, end_angle: f32, color: Color) {
    let num_segments = 100;
    let theta = (end_angle - start_angle) / num_segments as f32;
    for i in 0..num_segments {
        let angle_start = start_angle + theta * i as f32;
        let angle_end = start_angle + theta * (i + 1) as f32;
        let start_x = x + r * angle_start.cos();
        let start_y = y + r * angle_start.sin();
        let end_x = x + r * angle_end.cos();
        let end_y = y + r * angle_end.sin();
        draw_line(start_x, start_y, end_x, end_y, 2.0, color);
        draw_line(x, y, start_x, start_y, 2.0, color);
        draw_line(x, y, end_x, end_y, 2.0, color);
    }
}
