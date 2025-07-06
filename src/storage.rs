use std::{fs::File, io::{Read, Write}};

use bincode::{Decode, Encode};
use color_eyre::Result;

use crate::components::login::User;

#[derive(Decode, Encode, Debug, Clone)]
pub struct Storage {
    pub path: String,
    pub users: Vec<User>,
}

impl Storage {
    pub fn new() -> Storage {
        let mut users = Vec::new();
        users.push(User {
            username: String::from("groep1"),
            password: String::from("groep1"),
        });
        Storage {
            path: String::from("default.bin"),
            users
        }
    }

    pub fn try_login(&self, username: &str, password: &str) -> Option<User> {
        for user in &self.users {
            if user.username == username && user.password == password {
                return Some(user.clone());
            }
        }
        None
    }

    pub fn load(storage_path: String) -> Result<Storage> {
        let mut file = File::open(storage_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let config = bincode::config::standard();
        let (result, _bytes_read) = bincode::decode_from_slice(&buffer, config)?;
        Ok(result)
    }

    pub fn save(self) -> Result<()> {
        let mut file = File::create(&self.path)?;

        let config = bincode::config::standard();
        let encoded = bincode::encode_to_vec(self, config)?;

        file.write_all(&encoded)?;
        Ok(())
    }
}



