use std::f64::consts::PI;
use std::sync::Arc;
use std::time::{Instant, Duration};

use druid::kurbo::{Circle, Vec2, Point};
use druid::widget::prelude::*;
use druid::{Color, Data, Lens, TimerToken};

const NODE_RADIUS: f64 = 10.;
const MESSAGE_RADIUS: f64 = 5.;
const CIRCLE_RADIUS: f64 = 100.;

#[derive(Clone, Data, Lens)]
pub struct Node {
    pub id: String,
    pub pos: Point,
}

impl Node {
    pub fn new(id: String, pos: Point) -> Node {
        Node {
            id: id,
            pos: pos,
        }
    }
}

#[derive(Clone, Data, Lens)]
pub struct Message {
    pub from: Node,
    pub to: Node,
    pub pos: Point,
}

impl Message {
    pub fn new(from: Node, to: Node) -> Message {
        Message {
            from: from.clone(),
            to: to.clone(),
            pos: from.pos,
        }
    }
}

#[derive(Clone, Data, Lens)]
pub struct AppData {
    pub n: u32,
    pub nodes: Arc<Vec<Node>>,
    pub messages: Arc<Vec<Message>>,
}

impl AppData {
    // allows time interval in the range [100ms, 5000ms]
    // equivalently, 0.2 ~ 20ups
    pub fn iter_interval(&self) -> u64 {
        (1000. / 100.) as u64
    }
}

pub struct NodesWidget {
    pub timer_id: TimerToken,
    pub last_update: Instant,
}

impl Widget<AppData> for NodesWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::WindowConnected => {
                data.n = 2;
                let center = Vec2::new(200.0, 200.0);

                for k in 0..data.n {
                    let angle = (2.0 * PI / (data.n as f64)) * (k as f64);
                    let node_center = (center + Vec2::from_angle(angle) * CIRCLE_RADIUS).to_point();
                    Arc::make_mut(&mut data.nodes).push(Node::new(k.to_string(), node_center));
                }
                Arc::make_mut(&mut data.messages).push(Message::new(data.nodes[0].clone(), data.nodes[1].clone()));

                ctx.request_paint();
                let deadline = Duration::from_millis(data.iter_interval());
                self.last_update = Instant::now();
                self.timer_id = ctx.request_timer(deadline);
            }
            Event::Timer(id) => {
                if *id == self.timer_id {
                    let mut new_messages: Vec<Message> = vec!();
                    for msg in Arc::make_mut(&mut data.messages).iter_mut() {
                        let vec = (msg.to.pos.to_vec2() - msg.pos.to_vec2()).normalize();
                        msg.pos = (msg.pos.to_vec2() + vec).to_point();
                        if calc_dist(msg.pos, msg.to.pos) > NODE_RADIUS - MESSAGE_RADIUS {
                            new_messages.push(msg.clone());
                        } else {
                            new_messages.push(Message::new(msg.to.clone(), msg.from.clone()));
                        }
                    }
                    data.messages = Arc::new(new_messages);

                    ctx.request_paint();

                    let deadline = Duration::from_millis(data.iter_interval());
                    self.last_update = Instant::now();
                    self.timer_id = ctx.request_timer(deadline);
                }
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AppData,
        _env: &Env,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppData, data: &AppData, _env: &Env) {
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppData,
        _env: &Env,
    ) -> Size {
        let max_size = bc.max();
        let min_side = max_size.height.min(max_size.width);
        Size {
            width: min_side,
            height: min_side,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, _env: &Env) {
        for k in 0..data.n {
            ctx.fill(Circle::new(data.nodes[k as usize].pos, NODE_RADIUS), &Color::YELLOW);
        }
        for msg in data.messages.iter() {
            ctx.fill(Circle::new(msg.pos, MESSAGE_RADIUS), &Color::BLUE);
        }
    }
}

fn calc_dist(a: Point, b: Point) -> f64 {
    return ((a.x - b.x) * (a.x - b.x) - (a.y - b.y) * (a.y - b.y)).sqrt();
}
