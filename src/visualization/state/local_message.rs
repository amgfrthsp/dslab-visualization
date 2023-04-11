#[derive(Debug, Clone)]
pub struct StateLocalMessage {
    pub id: String,
    pub timestamp: f64,
    pub node_id: String,
    pub data: String,
    pub msg_type: LocalMessageType,
}

#[derive(Clone, Debug)]
pub enum LocalMessageType {
    Sent,
    Received,
}
