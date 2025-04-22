# Database API Documentation

This document outlines the public database endpoints available in the QR backend system.

## Table of Contents
- [User Management](#user-management)
- [Dynamic URL Management](#dynamic-url-management)
- [Subscription Management](#subscription-management)
- [Session Management](#session-management)

## User Management

### Insert User
```rust
pub async fn insert_user(&self, user: models::User) -> Response<models::UserResult>
```
Inserts a new user into the database after Auth0 post-registration.

**Parameters:**
- `user`: User object containing:
  - `id`: Auth0 user ID
  - `email`: User's email

**Returns:**
- `Response<models::UserResult>`: The inserted user object with generated fields

### Select User
```rust
pub async fn select_user(&self, user_id: &str) -> Response<Option<models::UserResult>>
```
Retrieves a user from the database by their ID.

**Parameters:**
- `user_id`: The user's Auth0 ID

**Returns:**
- `Response<Option<models::UserResult>>`: The user object if found, None otherwise

### Delete User Data
```rust
pub async fn delete_user_data(&self, user_id: &str) -> Response<bool>
```
Deletes all user data from the database.

**Parameters:**
- `user_id`: The user's Auth0 ID

**Returns:**
- `Response<bool>`: True if successful

## Dynamic URL Management

### List User URLs
```rust
pub async fn list_user_urls(&self, user_id: &str) -> Response<Vec<models::DynamicQrResult>>
```
Lists all dynamic URLs created by a user.

**Parameters:**
- `user_id`: The user's Auth0 ID

**Returns:**
- `Response<Vec<models::DynamicQrResult>>`: List of dynamic URLs

### Insert Dynamic URL
```rust
pub async fn insert_dynamic_url(&self, user_id: &str, dynamic_url: models::DynamicQr) -> Response<Vec<models::DynamicQrResult>>
```
Creates a new dynamic URL in the database.

**Parameters:**
- `user_id`: The user's Auth0 ID
- `dynamic_url`: Dynamic URL object containing:
  - `target_url`: The original destination URL

**Returns:**
- `Response<Vec<models::DynamicQrResult>>`: The created dynamic URL object

### Lookup Dynamic URL
```rust
pub async fn lookup_dynamic_url(&self, server_url: &str) -> Response<String>
```
Looks up a dynamic URL and returns its target URL.

**Parameters:**
- `server_url`: The server URL to look up

**Returns:**
- `Response<String>`: The target URL

### Update Dynamic URL
```rust
pub async fn update_dynamic_url(&self, server_url: &str, new_target_url: &str) -> Response<models::DynamicQrResult>
```
Updates the target URL of a dynamic URL.

**Parameters:**
- `server_url`: The server URL to update
- `new_target_url`: The new target URL

**Returns:**
- `Response<models::DynamicQrResult>`: The updated dynamic URL object

### Delete Dynamic URL
```rust
pub async fn delete_dynamic_url(&self, server_url: &str) -> Response<bool>
```
Deletes a dynamic URL from the database.

**Parameters:**
- `server_url`: The server URL to delete

**Returns:**
- `Response<bool>`: True if successful

## Subscription Management

### Get Subscription ID
```rust
pub async fn get_subscription_id(&self, user_id: &str) -> Response<Option<String>>
```
Retrieves a user's subscription ID.

**Parameters:**
- `user_id`: The user's Auth0 ID

**Returns:**
- `Response<Option<String>>`: The subscription ID if found

### Insert Subscription
```rust
pub async fn insert_subscription(&self, user_id: &str, subscription: models::UserSubscription) -> Response<models::UserSubscriptionResult>
```
Creates a new subscription for a user.

**Parameters:**
- `user_id`: The user's Auth0 ID
- `subscription`: Subscription object containing:
  - `sub_id`: Subscription ID
  - `tier`: Subscription tier
  - `status`: Subscription status

**Returns:**
- `Response<models::UserSubscriptionResult>`: The created subscription object

### Get Subscription
```rust
pub async fn get_subscription(&self, user_id: &str) -> Response<models::UserSubscriptionResult>
```
Retrieves a user's subscription details.

**Parameters:**
- `user_id`: The user's Auth0 ID

**Returns:**
- `Response<models::UserSubscriptionResult>`: The subscription object

### Override Subscription
```rust
pub async fn override_subscription(&self, user_id: &str, subscription_id: &str, new_tier: &str) -> Response<models::UserSubscriptionResult>
```
Updates a user's subscription tier.

**Parameters:**
- `user_id`: The user's Auth0 ID
- `subscription_id`: The subscription ID
- `new_tier`: The new subscription tier

**Returns:**
- `Response<models::UserSubscriptionResult>`: The updated subscription object

### Set Subscription Status
```rust
pub async fn set_subscription_status(&self, user_id: &str, status: &str) -> Response<models::UserSubscriptionResult>
```
Updates a user's subscription status.

**Parameters:**
- `user_id`: The user's Auth0 ID
- `status`: The new subscription status

**Returns:**
- `Response<models::UserSubscriptionResult>`: The updated subscription object

### Validate Subscription Status
```rust
pub async fn validate_subscription_status(&self, user_id: &str) -> Response<bool>
```
Checks if a user's subscription is valid.

**Parameters:**
- `user_id`: The user's Auth0 ID

**Returns:**
- `Response<bool>`: True if subscription is valid

### Increment Usage
```rust
pub async fn increment_usage(&self, user_id: &str) -> Response<models::UserSubscriptionResult>
```
Increments a user's subscription usage count.

**Parameters:**
- `user_id`: The user's Auth0 ID

**Returns:**
- `Response<models::UserSubscriptionResult>`: The updated subscription object

### Decrement Usage
```rust
pub async fn decrement_usage(&self, user_id: &str) -> Response<models::UserSubscriptionResult>
```
Decrements a user's subscription usage count.

**Parameters:**
- `user_id`: The user's Auth0 ID

**Returns:**
- `Response<models::UserSubscriptionResult>`: The updated subscription object

## Session Management

### Insert Session
```rust
pub async fn insert_session(&self, user_id: &str, session: models::PaymentSession) -> Response<models::PaymentSessionResult>
```
Creates a new payment session.

**Parameters:**
- `user_id`: The user's Auth0 ID
- `session`: Session object containing:
  - `session_id`: The session ID
  - `tier`: The session's tier

**Returns:**
- `Response<models::PaymentSessionResult>`: The created session object

### Get User from Session
```rust
pub async fn get_user_from_session(&self, session_id: &str) -> Response<UserResult>
```
Retrieves user information from a session ID.

**Parameters:**
- `session_id`: The session ID

**Returns:**
- `Response<UserResult>`: The user object

### Get User from Subscription
```rust
pub async fn get_user_from_subscription(&self, subscription_id: &str) -> Response<UserResult>
```
Retrieves user information from a subscription ID.

**Parameters:**
- `subscription_id`: The subscription ID

**Returns:**
- `Response<UserResult>`: The user object 