mod logs;
mod visualization;

use macroquad::prelude::*;
use visualization::{event_controller::EventController, state::State};

#[macroquad::main("Ping Pong based on history")]
async fn main() {
    let mut state = State::new();

    let mut ec = EventController::new();
    ec.parse_log("histories/ping-pong.json");
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

// fn receive_messaged(log: &mut EventLog, id: &str, timestamp: f64) {
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
//         node_cnt: 2,
//         events: vec![],
//     };

//     let mut t: f64 = 1.;

//     for i in 0..20 {
//         let id = i.to_string();
//         if i % 2 == 0 {
//             send_message(&mut log, &id, "0", "1", t);
//         } else {
//             send_message(&mut log, &id, "1", "0", t);
//         }
//         receive_messaged(&mut log, &id, t + 3.);
//         t += 4.;
//     }

//     let serialized = serde_json::to_string_pretty(&log).unwrap();
//     let mut file = OpenOptions::new()
//         .write(true)
//         .open("histories/ping-pong.json")
//         .unwrap();
//     file.write_all(serialized.as_bytes()).unwrap();
// }
