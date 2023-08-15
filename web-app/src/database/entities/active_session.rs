use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Model {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub idle_timeout: NaiveDateTime,
    pub absolute_timeout: NaiveDateTime,
}