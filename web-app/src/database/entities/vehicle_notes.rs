use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Model {
    pub id: i32,
    pub car_id: i32,
    pub body: String,
}