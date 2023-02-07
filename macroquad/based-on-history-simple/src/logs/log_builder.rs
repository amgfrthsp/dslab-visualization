fn main() {
    let sent = EventSent {
        timestamp: 0.0,
        msg: Message {
            id: String::from("1"),
            from: String::from("1"),
            to: String::from("2"),
            data: String::from("Hello"),
        },
    };

    let receive = EventReceived {
        timestamp: 3.0,
        id: String::from("1"),
    };

    let log = EventLog {
        node_cnt: 2,
        events: vec![Event::TypeSent(sent), Event::TypeReceived(receive)],
    };

    let serialized = serde_json::to_string_pretty(&log).unwrap();
    let deserialized: EventLog = serde_json::from_str(&serialized).unwrap();

    println!("{}", &serialized);
    println!("{:?}", &deserialized.events);
}
