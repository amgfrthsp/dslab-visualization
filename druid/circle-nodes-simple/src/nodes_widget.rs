use std::f64::consts::PI;

use druid::kurbo::{Circle, Vec2};
use druid::widget::prelude::*;
use druid::{Color};

pub struct NodesWidget {
    pub n: u32,
}

const NODE_RADIUS: f64 = 10.;
const CIRCLE_RADIUS: f64 = 100.;

impl Widget<()> for NodesWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut (), _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &(), _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &(), _data: &(), _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &(),
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &(), _env: &Env) {
        let n = self.n;
        let center = Vec2::new(200.0, 200.0);

        for k in 0..n {
            let angle = (2.0 * PI / (n as f64)) * (k as f64);
            let node_center = (center + Vec2::from_angle(angle) * CIRCLE_RADIUS).to_point();
            ctx.fill(Circle::new(node_center, NODE_RADIUS), &Color::YELLOW);
        }
    }
}
