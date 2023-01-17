use macroquad::prelude::*;

const NODE_RADIUS: f32 = 10.;
const MESSAGE_RADIUS: f32 = 5.;

struct Node {
    pos: Vec2,
}

struct Message {
    pos: Vec2,
    from: Vec2,
    to: Vec2,
}

#[macroquad::main("Ping Pong")]
async fn main() {
    let center = Vec2::new(screen_width() / 2., screen_height() / 2.);
    let nodes: Vec<Node> = vec![
        Node {pos: Vec2::new(center.x - 100.0, center.y)}, 
        Node {pos: Vec2::new(center.x + 100.0, center.y)}
    ];

    let mut messages: Vec<Message> = vec![Message {pos: nodes[0].pos, from: nodes[0].pos, to: nodes[1].pos}];

    loop {
        draw_circle(nodes[0].pos.x, nodes[0].pos.y, NODE_RADIUS, YELLOW);
        draw_circle(nodes[1].pos.x, nodes[1].pos.y, NODE_RADIUS, YELLOW);
        messages[0].pos = messages[0].pos + (messages[0].to - messages[0].pos).normalize(); // * speed
        if calc_dist(messages[0].pos, messages[0].to) < NODE_RADIUS - MESSAGE_RADIUS {
            messages[0].pos = messages[0].to;
            messages[0].to = messages[0].from;
            messages[0].from = messages[0].pos;
        }
        draw_circle(messages[0].pos.x, messages[0].pos.y, MESSAGE_RADIUS, BLUE);
        next_frame().await;
        continue;
    }
}

fn calc_dist(a: Vec2, b: Vec2) -> f32 {
    ((a.x - b.x) * (a.x - b.x) - (a.y - b.y) * (a.y - b.y)).sqrt()
}
