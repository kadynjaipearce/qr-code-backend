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

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkResult {
    pub target_url: String,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User {{ id: {}, email: {} }}", self.id, self.email)
    }
}

pub fn format_user_id(auth0_id: String) -> String {
    auth0_id.replace(&['|', '-'], "_") // removes the | encoded in auth_ids e.g. auth_0|id becomes auth_0_id
}
