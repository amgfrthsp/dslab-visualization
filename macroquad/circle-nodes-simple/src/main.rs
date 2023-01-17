use macroquad::prelude::*;
use std::f64::consts::PI;

const NODE_RADIUS: f32 = 10.;
const CIRCLE_RADIUS: f32 = 100.;

#[macroquad::main("Circle")]
async fn main() {
    let n: usize = 5;
    let center = Vec2::new(screen_width() / 2., screen_height() / 2.);

    loop {
        for k in 0..n {
            let angle = (2.0 * PI / (n as f64)) * (k as f64);
            let node_center = center + Vec2::from_angle(angle as f32) * CIRCLE_RADIUS;
            draw_circle(node_center.x, node_center.y, NODE_RADIUS, YELLOW);
        }
        next_frame().await;
        continue;
    }
}
