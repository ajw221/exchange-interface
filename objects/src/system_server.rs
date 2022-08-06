use serde::{
    Deserialize,
    Serialize,
};



#[derive(Debug, Serialize, Deserialize)]
pub struct ServerTime {
    pub unixtime: i64,
    pub rfc1123: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    status: String,
    timestamp: String,
}

