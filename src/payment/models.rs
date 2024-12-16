use serde::{Deserialize, Serialize};


use crate::database::models::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub user: User,
    pub tier: String,
}
