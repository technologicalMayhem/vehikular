use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Model {
    pub version: i32,
    pub discription: String,
    pub applied_at: NaiveDateTime,
}