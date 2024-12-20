use core::fmt;
use serde::{Deserialize, Serialize};
use surrealdb::{
    sql::{Datetime, Thing},
    RecordId,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResult {
    id: RecordId,
    username: String,
    email: String,
    created_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSubscription {
    pub id: RecordId,
    pub tier: String,
    pub usage: i32,
    pub start_date: Datetime,
    pub end_date: Datetime,
    pub subscription_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicQr {
    pub server_url: String,
    pub target_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicQrResult {
    id: RecordId,
    server_url: String,
    target_url: String,
    access_count: i32,
    last_accessed: Datetime,
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
    // removes the | encoded in auth_ids e.g. auth_0|id becomes auth_0_id
    auth0_id.replace(&['|', '-'], "_") 
}
