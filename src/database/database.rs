use rocket::futures::future::ok;
use rocket::http::Status;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

use crate::database::models;
use crate::errors::{ApiError, Response};
use crate::utils::Environments;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status: Status,
    pub data: Option<T>,
}

pub struct Database {
    db: Surreal<Client>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: Thing,
}

impl Database {
    pub async fn new(secrets: &Environments) -> Response<Self> {
        let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;

        db.signin(Root {
            username: &secrets.get("DATABASE_USERNAME").as_str(),
            password: &secrets.get("DATABASE_PASSWORD").as_str(),
        })
        .await?;
        db.use_ns("ns").use_db("db").await?;

        db.query("
        DEFINE TABLE user SCHEMAFULL;
        DEFINE FIELD id ON user TYPE string ASSERT $value != NONE;
        DEFINE FIELD email ON user TYPE string ASSERT $value != NONE;
        DEFINE FIELD created_at ON user TYPE datetime ASSERT $value != NONE;
        ").await?;

        Ok(Database { db })
    }

    pub async fn insert_user(&self, user: models::User) -> Response<models::UserResult> {
        let mut result = self
            .db
            .query("CREATE type::thing('user', $id) SET email = $email, created_at = time::now();")
            .bind(("id", user.id))
            .bind(("email", user.email))
            .await?;

        let created: Option<models::UserResult> = result.take(0)?;
        Ok(created.unwrap())
    }

    pub async fn select_user(&self, id: String) -> Response<Option<models::UserResult>> {
        let result: Option<models::UserResult> = self
            .db
            .query("SELECT * FROM type::thing('user', $id);")
            .bind(("id", id))
            .await?
            .take(0)?;

        Ok(result)
    }

    pub async fn validate_user(&self, email: String) -> Response<bool> {
        let exists: Option<models::UserResult> = self
            .db
            .query("SELECT * FROM user WHERE email = $email")
            .bind(("email", email))
            .await?
            .take(0)?;

        Ok(exists.is_some())
    }
}
