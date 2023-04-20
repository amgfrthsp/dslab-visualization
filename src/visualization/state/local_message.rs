#[derive(Debug, Clone)]
pub struct StateLocalMessage {
    pub id: String,
    pub time: f64,
    pub node_name: String,
    pub data: String,
    pub msg_type: LocalMessageType,
}

#[derive(Clone, Debug)]
pub enum LocalMessageType {
    Sent,
    Received,
}
