use std::{env, sync::Mutex};

use lazy_static::lazy_static;
use redis::Commands;
use serde::Deserialize;

use crate::error::Error;

pub struct Redis {
    connection: redis::Client,
}

impl Redis {
    pub fn new() -> Redis {
        let username = env::var("REDIS_USERNAME").unwrap();
        let password = env::var("REDIS_PASSWORD").unwrap();
        let host = env::var("REDIS_HOST").unwrap();
        let port = env::var("REDIS_PORT").unwrap();
        let database = env::var("REDIS_DB").unwrap();

        let conn_string = format!(
            "redis://{}:{}@{}:{}/{}",
            username, password, host, port, database
        );
        let connection = redis::Client::open(conn_string).unwrap();
        Self { connection }
    }

    pub fn set_ex(&mut self, key: String, seconds: usize, value: String) -> Result<(), Error> {
        self.connection.set_ex(key, value, seconds)?;
        Ok(())
    }

    pub fn get_json<K: for<'a> Deserialize<'a>>(&mut self, key: String) -> Result<K, Error> {
        let data: Option<String> = self.connection.get(key)?;
        match data {
            Some(d) => Ok(serde_json::from_str(&d)?),
            None => Err(Error::new("Value not found in redis cache", 500)),
        }
    }

    pub fn get(&mut self, key: String) -> Result<String, Error> {
        let data: String = self.connection.get(key)?;

        Ok(data)
    }

    pub fn del(&mut self, key: String) -> Result<(), Error> {
        self.connection.del(key)?;
        Ok(())
    }
}

lazy_static! {
    pub static ref REDIS_INSTANCE: Mutex<Redis> = Mutex::new(Redis::new());
}
