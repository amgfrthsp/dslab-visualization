mod logs;
mod visualization;

use macroquad::prelude::*;
use visualization::{event_controller::EventController, state::State};

#[macroquad::main("Ping Pong based on history")]
async fn main() {
    let mut state = State::new();

    let mut ec = EventController::new();
    ec.parse_log("src/logs/log.json");
    ec.send_commands(&mut state);

    loop {
        state.update();
        state.draw();

        next_frame().await;
    }
}
