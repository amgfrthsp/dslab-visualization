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

// LOG BUILDER

// use std::fs::OpenOptions;
// use std::io::prelude::*;

// use logs::log_entities::*;

// mod logs;

// fn main() {
//     let mut log = EventLog {
//         node_cnt: 3,
//         events: vec![],
//     };

//     log.events.push(Event::TypeSent(EventSent {
//         timestamp: 1.0,
//         msg: Message {
//             id: String::from("1"),
//             from: String::from("1"),
//             to: String::from("2"),
//             data: String::from("Hello"),
//         },
//     }));

//     let serialized = serde_json::to_string_pretty(&log).unwrap();

//     let mut file = OpenOptions::new()
//         .write(true)
//         .open("src/logs/log.json")
//         .unwrap();
//     file.write_all(serialized.as_bytes()).unwrap();
// }

// use macroquad::prelude::*;
// use macroquad_particles::{self as particles, AtlasConfig, BlendMode, EmitterConfig};

// fn explosion() -> particles::EmitterConfig {
//     particles::EmitterConfig {
//         one_shot: true,
//         emitting: false,
//         lifetime: 0.3,
//         lifetime_randomness: 0.7,
//         explosiveness: 0.95,
//         amount: 30,
//         initial_direction_spread: 2.0 * std::f32::consts::PI,
//         initial_velocity: 200.0,
//         size: 30.0,
//         gravity: vec2(0.0, -1000.0),
//         atlas: Some(AtlasConfig::new(4, 4, 8..)),
//         blend_mode: BlendMode::Additive,
//         ..Default::default()
//     }
// }

// #[macroquad::main("Fountain")]
// async fn main() {
//     let texture = load_texture("./smoke_fire.png").await.unwrap();

//     let mut one_shot_emitter = particles::Emitter::new(EmitterConfig {
//         texture: Some(texture),
//         ..explosion()
//     });

//     loop {
//         clear_background(BLACK);

//         one_shot_emitter.draw(vec2(650.0, 82.0));
//         draw_circle(650.0, 82.0, 15.0, YELLOW);

//         if is_key_pressed(KeyCode::Space) {
//             one_shot_emitter.config.emitting = true;
//         }
//         next_frame().await
//     }
// }
