use std::{fs::File, io::{Read, Write}};

use serde::{Deserialize, Serialize};
use color_eyre::Result;

use crate::{objects::SolarSystem, user::User};

#[derive(Deserialize, Serialize, Clone)]
pub struct Storage {
    pub path: String,
    pub users: Vec<User>,
    pub components: i32,
    pub map: Vec<SolarSystem>,
}

impl Storage {
    pub fn new(path: String) -> Storage {
        Storage {
            path: path,
            users: Vec::new(),
            map: Vec::new(),
            components: 0,
        }
    }

    pub fn load(storage_path: String) -> Result<Storage> {
        let mut file = File::open(storage_path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let result: Storage = serde_json::from_str(&buffer).expect("JSON to be valid");

        Ok(result)
    }

    pub fn save(&self) -> Result<()> {
        let mut file = File::create(&self.path)?;

        file.write_all(&serde_json::to_vec_pretty(&self)?)?;
        Ok(())
    }

    pub fn find_user(&self, username: &str) -> Option<&User> {
        self.users.iter().find(|u| u.username == username)
    }

    /// The login puzzle. Users start without a password; the game master
    /// hands out a code that starts with `password_start`. Attempts with the
    /// right prefix are counted, and once `password_attempts_max` is reached
    /// the next such attempt becomes the user's password. After that it is a
    /// plain password check. Callers should save afterwards because attempts
    /// are part of the state.
    pub fn try_login(&mut self, username: &str, password: &str) -> Option<User> {
        for user in self.users.iter_mut() {
            if user.username != username {
                continue;
            }
            if user.password.is_empty() {
                let prefix = password.split('-').next().unwrap_or_default();
                if prefix == user.password_start {
                    if user.password_attempts >= user.password_attempts_max {
                        user.password = password.to_string();
                    } else {
                        user.password_attempts += 1;
                    }
                }
            }
            if !user.password.is_empty() && user.password == password {
                return Some(user.clone());
            }
        }
        None
    }

    pub fn update_user(&mut self, user: &User) {
        for u in self.users.iter_mut() {
            if u.username == user.username {
                *u = user.clone();
            }
        }
    }
}

