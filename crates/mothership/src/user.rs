use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct User {
    pub username: String,
    pub password_start: String,
    pub password_attempts: u8,
    pub password_attempts_max: u8,
    pub password: String,
    pub pos_x: f64,
    pub pos_y: f64,

    // Resources,
    pub fuel: i32,
    pub crystals: i32,
    pub reputation: i32,
}
