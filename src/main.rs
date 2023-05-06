mod logs;
mod visualization;

use std::env;

use macroquad::prelude::*;
use visualization::{event_controller::EventController, state::state::State};

fn window_conf() -> Conf {
    Conf {
        window_title: "Dist sys cartoon".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as _);

    let args: Vec<String> = env::args().collect();

    let mut state = State::new();
    let default_history = "examples/broadcast.json".to_string();

    let mut ec = EventController::new();
    ec.parse_log(args.get(1).unwrap_or(&default_history));
    ec.send_commands(&mut state);

    loop {
        state.draw_ui();

        egui_macroquad::draw();

        state.update();
        state.draw();

        next_frame().await;
    }
}
