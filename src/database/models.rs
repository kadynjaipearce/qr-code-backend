use core::fmt;
use serde::{Deserialize, Serialize};
use surrealdb::{sql::Datetime, RecordId};

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionStatus {
    pub subscription_status: String,
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
pub struct UserDetails {
    pub user: UserResult,
    pub subscription: UserSubscriptionResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionId {
    pub subscription_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]  // This will map the enum variants to lowercase snake_case in JSON
pub enum SubscriptionAction {
    Cancel,
    Upgrade,
    Downgrade,
    Resume,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRequest {
    pub action: SubscriptionAction,
    pub subscription_id: String,
    pub new_tier: String,

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
pub struct NewSubscription {
    pub new_tier: String,
    pub new_price_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSubscription {
    pub sub_id: String,
    pub tier: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSubscriptionResult {
    pub id: RecordId,
    pub subscription_id: String,
    pub tier: String,
    pub usage: i32,
    pub start_date: Datetime,
    pub end_date: Datetime,
    pub subscription_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicQr {
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
