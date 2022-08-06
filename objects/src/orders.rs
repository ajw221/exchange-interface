use serde::{
    Deserialize,
    Serialize
};

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrders {
    pub open: HashMap<String, Order>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub refid: String,
    pub userref: String,
    pub status: String,
    pub opentm: i64,
    pub start_tm: i64,
    pub expire_tm: i64,
    pub descr: OrderInfo,
    pub vol: String,
    pub vol_exec: String,
    pub cost: String,
    pub fee: String,
    pub price: String,
    pub stopprice: String,
    pub limitprice: String,
    pub trigger: String,
    pub misc: String,
    pub oflags: String,
    pub trades: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderInfo {
    pub pair: String,
    pub r#type: String,
    pub ordertype: String,
    pub price: String,
    pub price2: String,
    pub leverage: String,
    pub order: String,
    pub close: String,
}
