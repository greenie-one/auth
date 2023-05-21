use async_once::AsyncOnce;
use lazy_static::lazy_static;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{ClientOptions, FindOneOptions, InsertOneOptions},
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
        let client_options = ClientOptions::parse("mongodb+srv://greenie_backend:98mBkHwq2l03ZIiTpXK665OK8@development.foqafth.mongodb.net/greenie_mvp").await.unwrap();
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
            filter.insert("_id", id.unwrap());
        }

        if email.is_some() {
            filter.insert("email", email.unwrap());
        }

        if mobile.is_some() {
            filter.insert("mobileNumber", mobile.unwrap());
        }

        let found = collection
            .find_one(filter, FindOneOptions::default())
            .await?;

        Ok(found)
    }

    pub async fn create_user(&self, user: UserModel) -> Result<InsertOneResult, Error> {
        let collection: mongodb::Collection<UserModel> = self.connection.collection("users");
        let res = collection
            .insert_one(user, InsertOneOptions::default())
            .await?;

        Ok(res)
    }
}

lazy_static! {
    pub static ref MONGO_DB_INSTANCE: AsyncOnce<MongoDB> = AsyncOnce::new(async {
        let client = MongoDB::new().await;
        client
    });
}
