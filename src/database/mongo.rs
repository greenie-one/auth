use std::{env, str::FromStr};

use async_once::AsyncOnce;
use lazy_static::lazy_static;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::ClientOptions,
    results::InsertOneResult,
    Client, Database,
};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct UserModel {
    pub _id: Option<ObjectId>,
    pub email: Option<String>,

    #[serde(rename = "mobileNumber")]
    pub mobile_number: Option<String>,
    pub password: Option<String>,
    pub roles: Vec<String>,
}

pub struct MongoDB {
    connection: Database,
}

impl MongoDB {
    pub async fn new() -> MongoDB {
        let username = env::var("DB_USER").unwrap();
        let password = env::var("DB_PASSWORD").unwrap();
        let host = env::var("DB_HOST").unwrap();
        let database = env::var("DB_DATABASE").unwrap();

        let conn_string = format!(
            "mongodb+srv://{}:{}@{}/{}",
            username, password, host, database
        );
        let client_options = ClientOptions::parse(conn_string).await.unwrap();
        let client = Client::with_options(client_options).unwrap();

        let database = client.database("greenie_mvp");

        Self {
            connection: database,
        }
    }

    pub async fn find_user(
        &self,
        email: Option<String>,
        mobile: Option<String>,
        id: Option<String>,
    ) -> Result<Option<UserModel>, mongodb::error::Error> {
        let collection: mongodb::Collection<UserModel> = self.connection.collection("users");
        let mut filter = doc! {};

        if id.is_some() {
            filter.insert("_id", ObjectId::from_str(&id.unwrap()).unwrap());
        }

        if email.is_some() {
            filter.insert("email", email.unwrap());
        }

        if mobile.is_some() {
            filter.insert("mobileNumber", mobile.unwrap());
        }

        let found = collection.find_one(filter, None).await?;

        Ok(found)
    }

    pub async fn update_password(
        &self,
        user_id: String,
        password: String,
    ) -> Result<Option<UserModel>, Error> {
        let collection: mongodb::Collection<UserModel> = self.connection.collection("users");

        println!("Got new hash {}", password);

        let filter = doc! {
            "_id": user_id
        };
        collection
            .find_one_and_update(filter, doc! { "$set" : { "password": password } }, None)
            .await
            .map_err(|e| e.into())
    }

    pub async fn create_user(&self, user: UserModel) -> Result<InsertOneResult, Error> {
        let collection: mongodb::Collection<UserModel> = self.connection.collection("users");
        let res = collection.insert_one(user, None).await?;

        Ok(res)
    }
}

lazy_static! {
    pub static ref MONGO_DB_INSTANCE: AsyncOnce<MongoDB> = AsyncOnce::new(async {
        let client = MongoDB::new().await;
        client
    });
}
