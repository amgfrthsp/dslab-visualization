use druid::{AppLauncher, WindowDesc};

mod nodes_widget;
use crate::nodes_widget::*;

pub fn main() {
    let window = WindowDesc::new(move || NodesWidget{ n: 8 }).title("Node circle");
    AppLauncher::with_window(window)
        .launch(())
        .expect("launch failed");
}