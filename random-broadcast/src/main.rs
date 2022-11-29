use std::{time::Instant, sync::Arc};

use druid::{AppLauncher, WindowDesc, TimerToken};

mod nodes_widget;
use crate::nodes_widget::*;

pub fn main() {
    let window = WindowDesc::new( move ||
        NodesWidget{
            timer_id: TimerToken::INVALID,
            broadcast_timer: TimerToken::INVALID,
            last_update: Instant::now(),
        }).title("Random Broadcast");
    AppLauncher::with_window(window)
        .launch(AppData {
            n: 5,  // Here you can customize the number of nodes
            nodes: Arc::new(vec!()),
            messages: Arc::new(vec!()),
        })
        .expect("launch failed");
}