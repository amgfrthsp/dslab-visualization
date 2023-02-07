use macroquad::prelude::Vec2;

pub const NODE_RADIUS: f32 = 15.;
pub const MESSAGE_RADIUS: f32 = 5.;
pub const CIRCLE_RADIUS: f32 = 150.;
pub const FPS: f32 = 60.0;

pub fn calc_dist(a: Vec2, b: Vec2) -> f32 {
    ((a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)).sqrt()
}
