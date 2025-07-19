use std::{fs::File, io::{Read, Write}};

use serde::{Deserialize, Serialize};
use color_eyre::Result;

use crate::{objects::SolarSystem, user::User};

#[derive(Deserialize, Serialize, Clone)]
pub struct Storage {
    pub path: String,
    pub users: Vec<User>,
    pub map: Vec<SolarSystem>,
    pub components: i32,
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

    pub fn save(self) -> Result<()> {
        let mut file = File::create(&self.path)?;

        file.write_all(&serde_json::to_vec_pretty(&self)?)?;
        Ok(())
    }

    pub fn update_user(&mut self, user: &User) {
        for u in self.users.iter_mut() {
            if u.username == user.username {
                *u = user.clone();
            }
        }
    }
}

