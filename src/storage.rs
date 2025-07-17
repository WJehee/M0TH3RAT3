use std::{fs::File, io::{Read, Write}};

use serde::{Deserialize, Serialize};
use color_eyre::Result;

use crate::login::User;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Storage {
    pub path: String,
    pub users: Vec<User>,
}

impl Storage {
    pub fn new(path: String) -> Storage {
        Storage {
            path: path,
            users: Vec::new(),
        }
    }

    pub fn load(storage_path: String) -> Result<Storage> {
        let mut file = File::open(storage_path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let result: Storage = serde_json::from_str(&buffer)?;

        Ok(result)
    }

    pub fn save(self) -> Result<()> {
        let mut file = File::create(&self.path)?;

        file.write_all(&serde_json::to_vec_pretty(&self)?)?;
        Ok(())
    }
}



