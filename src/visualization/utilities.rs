use macroquad::prelude::*;

pub const NODE_RADIUS: f32 = 15.;
pub const MESSAGE_RADIUS: f32 = 5.;
pub const CIRCLE_RADIUS: f32 = 150.;
pub const FPS: f32 = 60.0;

pub const ALIVE_NODE_COLOR: Color = YELLOW;
pub const DEAD_NODE_COLOR: Color = RED;

pub const SPEED_DELTA: f32 = 0.02;

pub fn calc_dist(a: Vec2, b: Vec2) -> f32 {
    ((a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)).sqrt()
}