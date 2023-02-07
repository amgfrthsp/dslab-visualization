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
//     log.events.push(Event::TypeReceived(EventReceived {
//         timestamp: 3.0,
//         id: String::from("1"),
//     }));

//     log.events.push(Event::TypeSent(EventSent {
//         timestamp: 4.0,
//         msg: Message {
//             id: String::from("2"),
//             from: String::from("2"),
//             to: String::from("1"),
//             data: String::from("Hi"),
//         },
//     }));

//     log.events.push(Event::TypeReceived(EventReceived {
//         timestamp: 7.0,
//         id: String::from("2"),
//     }));

//     let serialized = serde_json::to_string_pretty(&log).unwrap();

//     let mut file = OpenOptions::new()
//         .write(true)
//         .open("src/logs/log.json")
//         .unwrap();
//     file.write_all(serialized.as_bytes()).unwrap();
// }
