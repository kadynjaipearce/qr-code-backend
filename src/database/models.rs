use core::fmt;
use std::fmt::write;

use serde::{Deserialize, Serialize};
use surrealdb::{sql::Datetime, sql::Thing, Response};

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


impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User {{ id: {}, email: {} }}", self.id, self.email)
    }
}
