use actix::prelude::*;
use polars::prelude::*;

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct QAUnionData {
    pub data: Arc<DataFrame>,
    pub types: String,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct RequestData {
    start: String,
    end: String,
    orderbookids: Vec<String>,
    market: String,
}
