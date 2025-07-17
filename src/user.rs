use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct User {
    pub username: String,
    pub password: String,
    pub pos_x: f64,
    pub pox_y: f64,

    // Resources,
    pub fuel: i32,
    pub crystals: i32,
}
