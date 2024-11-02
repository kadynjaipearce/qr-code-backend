use core::fmt;
use std::fmt::write;

use serde::{Deserialize, Serialize};
use surrealdb::{sql::Thing, Response};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub auth0_id: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResult {
    
    id: String,
    email: String
}

impl TryFrom<UserResult> for User {
    type Error = ();

    fn try_from(value: UserResult) -> Result<Self, Self::Error> {
        let user = User {
            auth0_id: value.id,
            email: value.email
        };

        Ok(user)
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "User {{ auth0_id: {}, email: {} }}",
            self.auth0_id, self.email
        )
    }
}
