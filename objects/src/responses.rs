use crate::{
    orders::OpenOrders,
    system_server::{
        ServerTime,
        SystemStatus,
    },
    trades::TradingPairs,
};

use serde::{
    Deserialize,
    Serialize,
};



#[derive(Debug, Serialize, Deserialize)]
pub struct TradingPairResponse {
    pub error: Vec<String>,
    pub result: TradingPairs,
}

/* Result Enum */
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseResult {
    OpenOrdersResponse(OpenOrders),
    ServerTimeResponse(ServerTime),
    SystemStatusResponse(SystemStatus),
}

/* Response Object */
#[derive(Debug, Serialize, Deserialize)]
pub struct APIResponse {
    pub error: Vec<String>,
    pub result: ResponseResult,
}