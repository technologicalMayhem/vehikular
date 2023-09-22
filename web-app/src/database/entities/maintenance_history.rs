use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Model {
    pub author: Option<String>,
    pub date_time: NaiveDateTime,
    pub subject: String,
    pub body: String,
    pub mileage: Option<i32>,
}
