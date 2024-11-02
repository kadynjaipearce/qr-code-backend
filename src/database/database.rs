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
    pub db: Surreal<Client>,
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

        Ok(Database { db })
    }

    pub async fn insert_user(&self, user: models::User) -> Response<models::User> {
        let mut result = self
            .db
            .query("CREATE user SET id = $id, email = $email")
            .bind(("id", user.auth0_id))
            .bind(("email", user.email))
            .await?;

        let created: Option<models::User> = result.take(0)?;
        Ok(created.unwrap())
    }

    pub async fn select_user(&self, auth0_id: String) -> Response<Option<models::User>> {
        let result: Option<models::User> = self
            .db
            .query("SELECT * FROM users WHERE id = $id")
            .bind(("id", auth0_id))
            .await?
            .take(0)?;

        Ok(result)
    }

    pub async fn view_users(&self) -> Response<Vec<models::UserResult>> {
        let result: Vec<models::UserResult> = self.db.select("user").await?;

        Ok(result)
    }
}
