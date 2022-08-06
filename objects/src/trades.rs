use serde::{
    Deserialize,
    Serialize,
};

use std::collections::HashMap;



pub type TradingPairs = HashMap<String, TradingPair>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TradingPair {
    pub altname: String,
    pub wsname: String,
    pub aclass_base: String,
    pub base: String,
    pub aclass_quote: String,
    pub quote: String,
    pub lot: String,
    pub pair_decimals: i64,
    pub lot_decimals: i64,
    pub lot_multiplier: i64,
    pub leverage_buy: Vec<i64>,
    pub leverage_sell: Vec<i64>,
    pub fees: Vec<Vec<f64>>,
    pub fees_maker: Vec<Vec<f64>>,
    pub fee_volume_currency: String,
    pub margin_call: i64,
    pub margin_stop: i64,
    pub ordermin: String,
}