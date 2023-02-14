mod logs;
mod visualization;

use std::env;

use macroquad::prelude::*;
use visualization::{event_controller::EventController, state::State};

#[macroquad::main("Based on history")]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let mut state = State::new();
    let default_history = "examples/broadcast.json".to_string();

    let mut ec = EventController::new();
    ec.parse_log(args.get(1).unwrap_or(&default_history));
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

// fn send_message(log: &mut EventLog, id: &str, from: &str, to: &str, timestamp: f64) {
//     log.events.push(Event::TypeSent(EventSent {
//         timestamp: timestamp,
//         msg: Message {
//             id: id.to_string(),
//             from: from.to_string(),
//             to: to.to_string(),
//             data: String::from("Hello"),
//         },
//     }));
// }

// fn receive_message(log: &mut EventLog, id: &str, timestamp: f64) {
//     log.events.push(Event::TypeReceived(EventReceived {
//         timestamp: timestamp,
//         id: id.to_string(),
//     }));
// }

// fn crash_node(log: &mut EventLog, id: &str, timestamp: f64) {
//     log.events.push(Event::TypeNodeDown(EventNodeDown {
//         timestamp: timestamp,
//         id: id.to_string(),
//     }));
// }

// fn up_node(log: &mut EventLog, id: &str, timestamp: f64) {
//     log.events.push(Event::TypeNodeUp(EventNodeUp {
//         timestamp: timestamp,
//         id: id.to_string(),
//     }));
// }

// fn main() {
//     let mut log = EventLog {
//         node_cnt: 5,
//         events: vec![],
//     };

//     send_message(&mut log, "0", "0", "1", 1.);
//     send_message(&mut log, "1", "0", "2", 1.);
//     send_message(&mut log, "2", "0", "3", 1.);
//     send_message(&mut log, "3", "0", "4", 1.);

//     receive_message(&mut log, "0", 3.);
//     receive_message(&mut log, "1", 4.);
//     receive_message(&mut log, "2", 5.);
//     receive_message(&mut log, "3", 6.);

//     send_message(&mut log, "4", "3", "1", 4.);
//     send_message(&mut log, "5", "3", "2", 4.);
//     send_message(&mut log, "6", "3", "0", 4.);
//     send_message(&mut log, "7", "3", "4", 4.);

//     receive_message(&mut log, "4", 6.);
//     receive_message(&mut log, "5", 7.);
//     receive_message(&mut log, "6", 8.);
//     receive_message(&mut log, "7", 9.);

//     let serialized = serde_json::to_string_pretty(&log).unwrap();
//     let mut file = OpenOptions::new()
//         .write(true)
//         .open("examples/broadcast.json")
//         .unwrap();
//     file.write_all(serialized.as_bytes()).unwrap();
// }
