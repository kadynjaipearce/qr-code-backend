use core::fmt;
use serde::{Deserialize, Serialize};
use surrealdb::{sql::Datetime, RecordId};

pub enum SubscriptionStatus {
    Active,
    Inactive,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Lite,
    Pro,
}

impl SubscriptionTier {
    // Define the max usage for each tier
    pub fn max_usage(&self) -> i32 {
        match self {
            SubscriptionTier::Lite => 5,
            SubscriptionTier::Pro => 25,
        }
    }

    // Convert a string to a SubscriptionTier enum
    pub fn from_str(tier_str: &str) -> Option<Self> {
        match tier_str {
            "Lite" => Some(SubscriptionTier::Lite),
            "Pro" => Some(SubscriptionTier::Pro),
            _ => None, // Invalid tier string
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResult {
    pub id: RecordId,
    pub username: String,
    pub email: String,
    pub created_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentSession {
    pub session_id: String,
    pub tier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentSessionResult {
    pub session_id: String,
    pub tier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSubscription {
    pub sub_id: String,
    pub tier: String,
    pub status: String,
}

pub struct UpdateUserSubscription {
    pub id: String,
    pub tier: String,
    pub usage: i32,
    pub start_date: Datetime,
    pub end_date: Datetime,
    pub subscription_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSubscriptionResult {
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
