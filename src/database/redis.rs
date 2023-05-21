use std::sync::Mutex;

use lazy_static::lazy_static;
use redis::Commands;
use serde::Deserialize;

use crate::error::Error;

pub struct Redis {
    connection: redis::Client,
}

impl Redis {
    pub fn new() -> Redis {
        let connection =
            redis::Client::open("redis://greenie_mvp:J4g0eugG6JeEaUVpy@redis.greenie.one:6379")
                .unwrap();
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
