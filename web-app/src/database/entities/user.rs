use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Model {
    pub id: i32,
    pub display_name: String,
    pub email: String,
    pub password_hash: String,
}