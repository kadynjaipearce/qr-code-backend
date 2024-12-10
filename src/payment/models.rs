use core::fmt;
use serde::{Deserialize, Serialize};
use surrealdb::{sql::Datetime, RecordId};

use crate::database::models::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub user: User,
    pub tier: String,
}
