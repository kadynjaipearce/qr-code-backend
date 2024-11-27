use core::fmt;
use serde::{Deserialize, Serialize};
use surrealdb::{sql::Datetime, sql::Thing};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResult {
    id: Thing,
    email: String,
    created_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicUrl {
    pub server_url: String,
    pub target_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicUrlResult {
    id: String,
    server_url: String,
    target_url: String,
    created_at: Datetime,
    updated_at: Datetime,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User {{ id: {}, email: {} }}", self.id, self.email)
    }
}
