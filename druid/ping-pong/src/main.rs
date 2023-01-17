use std::{time::Instant, sync::Arc};

use druid::{AppLauncher, WindowDesc, TimerToken};

mod nodes_widget;
use crate::nodes_widget::*;

pub fn main() {
    let window = WindowDesc::new( move ||
        NodesWidget{
            timer_id: TimerToken::INVALID,
            last_update: Instant::now(),
        }).title("Ping-Pong");
    AppLauncher::with_window(window)
        .launch(AppData {
            n: 2,
            nodes: Arc::new(vec!()),
            messages: Arc::new(vec!()),
        })
        .expect("launch failed");
}