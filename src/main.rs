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
        state.draw_ui();

        egui_macroquad::draw();

        state.update();
        state.draw();

        next_frame().await;
    }
}

// LOG BUILDER

// use core::time;
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

// fn set_timer(log: &mut EventLog, id: &str, timestamp: f64, node_id: &str, delay: f64) {
//     let timer = Timer {
//         id: id.to_string(),
//         node_id: node_id.to_string(),
//         delay,
//     };
//     log.events
//         .push(Event::TypeTimerSet(EventTimerSet { timestamp, timer }));
// }

// fn removed_timer(log: &mut EventLog, id: &str, timestamp: f64) {
//     log.events.push(Event::TypeTimerRemoved(EventTimerRemoved {
//         timestamp,
//         id: id.to_string(),
//     }));
// }

// fn main() {
//     let mut log = EventLog {
//         node_cnt: 1,
//         events: vec![],
//     };

//     set_timer(&mut log, "0", 3.0, "0", 10.);
//     removed_timer(&mut log, "0", 13.0);

//     set_timer(&mut log, "1", 3.0, "0", 10.);
//     removed_timer(&mut log, "1", 13.0);

//     set_timer(&mut log, "2", 3.0, "0", 10.);
//     removed_timer(&mut log, "2", 13.0);

//     set_timer(&mut log, "3", 3.0, "0", 10.);
//     removed_timer(&mut log, "3", 13.0);

//     set_timer(&mut log, "4", 3.0, "0", 10.);
//     removed_timer(&mut log, "4", 13.0);

//     set_timer(&mut log, "5", 3.0, "0", 10.);
//     removed_timer(&mut log, "5", 13.0);

//     set_timer(&mut log, "6", 3.0, "0", 10.);
//     removed_timer(&mut log, "6", 13.0);

//     set_timer(&mut log, "7", 3.0, "0", 10.);
//     removed_timer(&mut log, "7", 13.0);

//     set_timer(&mut log, "8", 3.0, "0", 10.);
//     removed_timer(&mut log, "8", 13.0);

//     set_timer(&mut log, "10", 6.0, "0", 10.);
//     removed_timer(&mut log, "10", 9.0);

//     let serialized = serde_json::to_string_pretty(&log).unwrap();
//     let mut file = OpenOptions::new()
//         .write(true)
//         .open("examples/one_timer.json")
//         .unwrap();
//     file.write_all(serialized.as_bytes()).unwrap();
// }
