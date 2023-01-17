use macroquad::prelude::*;
use std::f32::consts::PI;

const NODE_RADIUS: f32 = 10.;
const MESSAGE_RADIUS: f32 = 5.;
const CIRCLE_RADIUS: f32 = 100.;
const BROADCAST_TIMER: f64 = 1.0; // in secs

struct State {
    n: usize,
    nodes: Vec<Node>,
    messages: Vec<Message>,
    last_update: f64,
    broadcast_timer: f64,
}

impl State {
    fn new(n: usize) -> Self {
        let center = Vec2::new(screen_width() / 2., screen_height() / 2.);

        let mut nodes: Vec<Node> = vec![];
        for i in 0..n {
            let angle = (2.0 * PI / (n as f32)) * (i as f32);
            let node_center = center + Vec2::from_angle(angle) * CIRCLE_RADIUS;
            nodes.push(Node { pos: node_center });
        }

        let messages: Vec<Message> = vec![];
        let time = get_time();

        Self {
            n,
            nodes,
            messages,
            last_update: time,
            broadcast_timer: 0.0,
        }
    }

    pub fn update(&mut self) {
        for msg in &mut self.messages {
            msg.update();
        }
        self.messages.retain(|msg| !msg.is_delivered());

        let time = get_time();
        let elapsed_time = time - self.last_update;
        self.broadcast_timer += elapsed_time;

        if self.broadcast_timer > BROADCAST_TIMER {
            let node = rand::gen_range(0, self.n);
            for i in 0..self.n {
                if i == node {
                    continue;
                }
                self.messages.push(Message {
                    pos: self.nodes[node].pos,
                    from: self.nodes[node].pos,
                    to: self.nodes[i].pos,
                });
            }
            self.broadcast_timer = 0.0;
        }
        self.last_update = time;
    }

    pub fn draw(&self) {
        for node in &self.nodes {
            node.draw();
        }
        for msg in &self.messages {
            msg.draw();
        }
    }
}

struct Node {
    pos: Vec2,
}

impl Node {
    pub fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, NODE_RADIUS, YELLOW);
    }
}

struct Message {
    pos: Vec2,
    from: Vec2,
    to: Vec2,
}

impl Message {
    pub fn update(&mut self) {
        self.pos += (self.to - self.from).normalize() * 0.5;
    }

    pub fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, MESSAGE_RADIUS, BLUE);
    }

    pub fn is_delivered(&self) -> bool {
        calc_dist(self.pos, self.to) < NODE_RADIUS - MESSAGE_RADIUS
    }
}

#[macroquad::main("Random broadcast")]
async fn main() {
    let mut state = State::new(5);

    loop {
        state.update();
        state.draw();

        next_frame().await;
    }
}

fn calc_dist(a: Vec2, b: Vec2) -> f32 {
    ((a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)).sqrt()
}
