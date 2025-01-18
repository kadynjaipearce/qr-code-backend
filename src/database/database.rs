use crate::database::models::{self, format_user_id};
use crate::errors::{ApiError, Response};
use crate::utils::Environments;

use surrealdb::engine::remote::ws::{Client, Wss};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use super::models::UserResult;

pub struct Database {
    db: Surreal<Client>, //  Holds a private instance of the SurrealDB connection to restrict query access.
}

impl Database {
    /*
        Initializes the database connection and defines two tables: `user` and `dynamic_url`.

        - `user` table:
            - `id` (string): Unique identifier for the user.
            - `email` (string): The user's email address.
            - `created_at` (datetime): Timestamp of when the user was created.

        - `dynamic_url` table:
            - `id` (string): Unique identifier for the dynamic URL.
            - `server_url` (string): The server URL that will be shortened or dynamic.
            - `target_url` (string): The original destination URL that the dynamic URL points to.
            - `created_at` (datetime): Timestamp of when the dynamic URL was created.
            - `updated_at` (datetime): Timestamp of the last update to the dynamic URL.
    */

    pub async fn new(secrets: &Environments) -> Response<Self> {
        // Establish a connection to the database using the provided URL.
        let db = Surreal::new::<Wss>(&secrets.get("DATABASE_URL")).await?;

        // Sign in using the provided credentials.
        db.signin(Root {
            username: &secrets.get("DATABASE_USERNAME").as_str(),
            password: &secrets.get("DATABASE_PASSWORD").as_str(),
        })
        .await?;

        // Set the namespace and database to use.
        db.use_ns("ns").use_db("db").await?;

        db.query(
            "
        DEFINE TABLE user SCHEMAFULL;
        DEFINE FIELD id ON user TYPE string ASSERT $value != NONE;
        DEFINE FIELD username ON user TYPE string ASSERT $value != NONE;
        DEFINE FIELD email ON user TYPE string ASSERT $value != NONE;
        DEFINE FIELD created_at ON user TYPE datetime ASSERT $value != NONE;

        DEFINE TABLE session SCHEMAFULL;
        DEFINE FIELD id ON session TYPE string ASSERT $value != NONE;
        DEFINE FIELD session_id ON session TYPE string ASSERT $value != NONE;
        DEFINE FIELD tier ON session TYPE string ASSERT $value != NONE;
        DEFINE FIELD created_at ON session TYPE datetime ASSERT $value != NONE; 

        DEFINE TABLE subscription SCHEMAFULL;
        DEFINE FIELD subscription_id ON subscription TYPE string ASSERT $value != NONE;
        DEFINE FIELD tier ON subscription TYPE string ASSERT $value != NONE;
        DEFINE FIELD start_date ON subscription TYPE datetime ASSERT $value != NONE;
        DEFINE FIELD end_date ON subscription TYPE datetime;
        DEFINE FIELD usage ON subscription TYPE int ASSERT $value != NONE;
        DEFINE FIELD subscription_status ON subscription TYPE string ASSERT $value != NONE;

        DEFINE TABLE dynamic_url SCHEMAFULL;
        DEFINE FIELD id ON dynamic_url TYPE string ASSERT $value != NONE;
        DEFINE FIELD server_url ON dynamic_url TYPE string ASSERT $value != NONE;
        DEFINE FIELD target_url ON dynamic_url TYPE string ASSERT $value != NONE;
        DEFINE FIELD access_count ON dynamic_url TYPE int ASSERT $value != NONE;
        DEFINE FIELD last_accessed ON dynamic_url TYPE datetime ASSERT $value != NONE;
        DEFINE FIELD created_at ON dynamic_url TYPE datetime ASSERT $value != NONE;
        DEFINE FIELD updated_at ON dynamic_url TYPE datetime ASSERT $value != NONE; 
        ",
        )
        .await?;

        // Return a new instance of the Database struct with the established connection.
        Ok(Database { db })
    }

    pub async fn list_user_urls(&self, user_id: &str) -> Response<Vec<models::DynamicQrResult>> {
        /*
           Lists all dynamic URLs created by a user.

           Params:
               user_id (string): The user's Auth0 ID.

           Returns:
               Response<Vec<models::DynamicUrlResult>>: A list of dynamic URLs created by the user.

        */

        let mut result = self
            .db
            .query("RETURN SELECT * FROM type::thing('user', $user)->created->dynamic_url")
            .bind(("user", user_id.to_string()))
            .await?;

        let created = result.take::<Vec<models::DynamicQrResult>>(0)?;

        match created.is_empty() {
            true => Err(ApiError::InternalServerError("No URLs found.".to_string())),
            false => Ok(created),
        }
    }

    pub async fn insert_user(&self, user: models::User) -> Response<models::UserResult> {
        /*
            Inserts a new user into the database after Auth0 post-registration.

            Params:
                user (models::User): Contains:
                    - `id`: Auth0 user ID.
                    - `email`: User's email.

            Returns:
                Response<models::UserResult>: The inserted user object, including any generated fields like `created_at`.
        */

        let mut result = self
            .db
            .query("CREATE type::thing('user', $id) SET username = $username, email = $email, created_at = time::now();")
            .bind(("id", format_user_id(user.id)))
            .bind(("username", user.username))
            .bind(("email", user.email))
            .await?;

        match result.take::<Option<models::UserResult>>(0)? {
            Some(created) => Ok(created),
            None => Err(ApiError::InternalServerError(
                "Failed to create user.".to_string(),
            )),
        }
    }

    pub async fn select_user(&self, user_id: &str) -> Response<Option<models::UserResult>> {
        /*
           Selects a user from the database with a id.

           Params:
               id (string): The user's Auth0 ID.

            Returns:
               Response<Option<models::UserResult>>: The selected user object, or None if no user was found.
        */

        let mut result = self
            .db
            .query("SELECT * FROM type::thing('user', $user_id);")
            .bind(("user_id", user_id.to_string()))
            .await?;

        match result.take::<Option<models::UserResult>>(0)? {
            Some(user) => Ok(Some(user)),
            None => Ok(None),
        }
    }

    pub async fn insert_dynamic_url(
        &self,
        user_id: &str,
        dynamic_url: models::DynamicQr,
    ) -> Response<Vec<models::DynamicQrResult>> {
        /*
           Inserts a new dynamic URL into the database.

           Params:
               user_id (string): The user's Auth0 ID.
               dynamic_url (models::DynamicUrl): Contains:
                   - `server_url`: The server URL that will be shortened.
                   - `target_url`: The original destination URL that the dynamic URL points to.

           Returns:
               Response<models::DynamicUrlResult>: The inserted dynamic URL object, including any generated fields like `created_at`.

        */

        let mut result = self
            .db
            .query(
                "
                LET $user = type::thing('user', $user_id);
                LET $url = type::thing('dynamic_url', rand::ulid());
                
        RELATE $user->created->CREATE $url 
        SET server_url = rand::ulid(), 
        access_count = 0,
        last_accessed = time::now(),
        target_url = $target_url, 
        access_count = 0,
        last_accessed = time::now(),
        created_at = time::now(), 
        updated_at = time::now();
        
        SELECT * FROM $user->created->dynamic_url;",
            )
            .bind(("user_id", user_id.to_string()))
            .bind(("target_url", dynamic_url.target_url))
            .await?;

        let created = result.take::<Vec<models::DynamicQrResult>>(3)?;

        if created.is_empty() {
            Err(ApiError::InternalServerError(
                "Failed to create dynamic URL.".to_string(),
            ))
        } else {
            Ok(created)
        }
    }

    pub async fn lookup_dynamic_url(&self, server_url: &str) -> Response<String> {
        /*
           Looks up a dynamic URL in the database and returns the target URL.

           Params:
               server_url (string): The server URL to look up.

           Returns:
               Response<String>: The target URL that the server URL points to.

        */

        let mut result = self
            .db
            .query("SELECT target_url FROM dynamic_url WHERE server_url = $server_url;
                    UPDATE dynamic_url SET access_count = access_count + 1, last_accessed = time::now() WHERE server_url = $server_url;")
            .bind(("server_url", server_url.to_string()))
            .await?;

        match result.take::<Option<models::LinkResult>>(0)? {
            Some(created) => Ok(created.target_url),
            None => Err(ApiError::InternalServerError(
                "Url doesn't exist.".to_string(),
            )),
        }
    }

    pub async fn update_dynamic_url(
        &self,
        server_url: &str,
        new_target_url: &str,
    ) -> Response<models::DynamicQrResult> {
        /*
             Updates the target URL of a dynamic URL in the database.

             Params:
               server_url (string): The server URL to update.
               new_target_url (string): The new target URL to set.

             Returns:
               Response<models::DynamicUrlResult>: The updated dynamic URL object, including any generated fields like `updated_at`.

        */

        let mut result = self
            .db
            .query("UPDATE dynamic_url SET target_url = $target_url, updated_at = time::now() WHERE server_url = $server_url")
            .bind(("server_url", server_url.to_string()))
            .bind(("target_url", new_target_url.to_string()))
            .await?;

        match result.take::<Option<models::DynamicQrResult>>(0)? {
            Some(updated) => Ok(updated),
            None => Err(ApiError::InternalServerError(
                "No matching URL found.".to_string(),
            )),
        }
    }

    pub async fn delete_dynamic_url(&self, server_url: &str) -> Response<bool> {
        /*
            Deletes a dynamic URL from the database.

            Params:
                id (string): The ID of the dynamic URL to delete.

        */

        let _ = self
            .db
            .query("DELETE dynamic_url WHERE server_url = $server_url")
            .bind(("server_url", server_url.to_string()))
            .await?;

        Ok(true)
    }

    pub async fn get_subscription_id(&self, user_id: &str) -> Response<Option<String>> {
        /*
            Looks up a user's subscription in the database.

            Params:
                user_id (string): The user's Auth0 ID.

            Returns:
                Response<Option<models::UserSubscription>>: The user's subscription object, or None if no subscription was found.

        */

        let mut result = self
            .db
            .query(
                "SELECT subscription_id FROM type::thing('user', $user)->subscribed->subscription",
            )
            .bind(("user", user_id.to_string()))
            .await?;

        match result.take::<Option<models::SubscriptionId>>(0)? {
            Some(id) => Ok(Some(id.subscription_id)),
            None => Ok(None),
        }
    }

    pub async fn delete_user_data(&self, user_id: &str) -> Response<bool> {
        /*
            Deletes a user's data from the database.

            Params:
                user_id (string): The user's Auth0 ID.

        */

        let _ = self
            .db
            .query("
                    LET $user = type::thing('user', $user_id);
            
                    DELETE $user->subscribed->subscription;
                    DELETE $user->created->dynamic_url;")
            .bind(("user_id", user_id.to_string()))
            .await?;

        Ok(true)
    }

    pub async fn get_user_from_subscription(
        &self,
        subscription_id: &str,
    ) -> Response<UserResult> {
        /*
            Looks up a user's Auth0 ID from a subscription ID in the database.

            Params:
                subscription_id (str): The subscription ID to look up.

            Returns:
                Response<UserResult>: The user's data

        */

        let mut result = self
            .db
            .query(
                "
                LET $subscription = type::thing('subscription', $subscription_id);
                LET $user = SELECT in FROM subscribed WHERE out = $subscription;
                SELECT * FROM $user.in;",
            )
            .bind(("subscription_id", subscription_id.to_string()))
            .await?;

        match result.take::<Option<models::UserResult>>(2)? {
            Some(subscription) => Ok(subscription),
            None => Err(ApiError::InternalServerError("No user found.".to_string())),
        }
    }

    pub async fn get_user_from_session(&self, session_id: &str) -> Response<UserResult> {
        /*
            Looks up a user's Auth0 ID from a session ID in the database.

            Params:
                session_id (string): The session ID to look up.

            Returns:
                Response<Option<String>>: The user's Auth0 ID, or None if no user was found.

        */

        let mut result = self
            .db
            .query(
                "
                LET $payment = type::thing('session', $session_id);
                LET $user = SELECT in FROM payment WHERE out = $payment;
                SELECT * FROM $user.in;",
            )
            .bind(("session_id", session_id.to_string()))
            .await?;

        match result.take::<Option<models::UserResult>>(2)? {
            Some(subscription) => Ok(subscription),
            None => Err(ApiError::InternalServerError("No user found.".to_string())),
        }
    }

    pub async fn insert_session(
        &self,
        user_id: &str,
        session: models::PaymentSession,
    ) -> Response<models::PaymentSessionResult> {
        /*
            Inserts a new session into the database.

            Params:
                session_id (string): The session ID.
                tier (string): The session's tier.

        */

        let mut result = self.db
            .query("

            LET $user = type::thing('user', $user_id);
            
            RELATE $user->payment->CREATE type::thing('session', $session_id) SET session_id = $session_id, tier = $tier, created_at = time::now();
            
            SELECT * FROM $user->payment->session ORDER BY created_at DESC LIMIT 1;")
            .bind(("user_id", user_id.to_string()))
            .bind(("session_id", session.session_id))
            .bind(("tier", session.tier))
            .await?;

        match result.take::<Option<models::PaymentSessionResult>>(2)? {
            Some(created) => Ok(created),
            None => Err(ApiError::InternalServerError(
                "Failed to create session.".to_string(),
            )),
        }
    }

    pub async fn insert_subscription(
        &self,
        user_id: &str,
        subscription: models::UserSubscription,
    ) -> Response<models::UserSubscriptionResult> {
        /*
            Inserts a new subscription into the database.

            Params:
                subscription_id (string): The subscription ID.
                tier (string): The subscription's tier.
                start_date (datetime): The subscription's start date.
                end_date (datetime): The subscription's end date.
                usage (int): The subscription's usage.
                subscription_status (string): The subscription's status.

        */

        let mut result = self
            .db
            .query("

            LET $user = type::thing('user', $user_id);
            
            RELATE $user->subscribed->CREATE type::thing('subscription', $subscription_id) 
            SET subscription_id = $subscription_id, tier = $tier, start_date = time::now(), end_date = time::now(), usage = 0, subscription_status = $subscription_status;
            
            SELECT * FROM $user->subscribed->subscription LIMIT 1;")
            .bind(("user_id", user_id.to_string()))
            .bind(("subscription_id", subscription.sub_id))
            .bind(("tier", subscription.tier))
            .bind(("subscription_status", subscription.status))
            .await?;

        match result.take::<Option<models::UserSubscriptionResult>>(2)? {
            Some(created) => Ok(created),
            None => Err(ApiError::InternalServerError(
                "Failed to create subscription.".to_string(),
            )),
        }
    }

    pub async fn get_subscription(
        &self,
        user_id: &str,
    ) -> Response<models::UserSubscriptionResult> {
        /*
            Gets a user's subscription from the database.

            Params:
                user_id (string): The user's Auth0 ID.

            Returns:
                Response<Option<models::UserSubscriptionResult>>: The user's subscription object, or None if no subscription was found.

        */

        let mut result = self
            .db
            .query("SELECT * FROM type::thing('user', $user_id)->subscribed->subscription;")
            .bind(("user_id", user_id.to_string()))
            .await?;

        match result.take::<Option<models::UserSubscriptionResult>>(0)? {
            Some(subscription) => Ok(subscription),
            None => Err(ApiError::InternalServerError(
                "No subscription found.".to_string(),
            )),
        }
    }

    pub async fn override_subscription(
        &self,
        user_id: &str,
        subscription_id: &str,
        new_tier: &str,
    ) -> Response<models::UserSubscriptionResult> {
        /*
            Overrides a user's subscription in the database.

            Params:
                user_id (string): The user's Auth0 ID.
                subscription (models::UserSubscription): The new subscription object.

            Returns:
                Response<models::UserSubscriptionResult>: The updated subscription object.

        */

        let mut result = self
            .db
            .query("LET $user = type::thing('user', $user_id);
            
            UPDATE subscription SET tier = $tier, start_date = time::now(), end_date = time::now() WHERE subscription_id = $subscription_id;
            
            SELECT * FROM subscription WHERE subscription_id = $subscription_id LIMIT 1;")
            .bind(("user_id", user_id.to_string()))
            .bind(("tier", new_tier.to_string()))
            .bind(("subscription_id", subscription_id.to_string() ))
            .await?;

        match result.take::<Option<models::UserSubscriptionResult>>(0)? {
            Some(updated) => Ok(updated),
            None => Err(ApiError::InternalServerError(
                "Failed to update subscription.".to_string(),
            )),
        }
    }

    pub async fn set_subscription_status(
        &self,
        user_id: &str,
        status: &str,
    ) -> Response<models::UserSubscriptionResult> {
        /*
            Sets a user's subscription status to inactive.

            Params:
                user_id (string): The user's Auth0 ID.

            Returns:
                Response<models::UserSubscriptionResult>: The updated subscription object.

        */

        let mut result = self
            .db
            .query("UPDATE type::thing('user', $user_id)->subscribed->subscription SET subscription_status = $status;")
            .bind(("user_id", user_id.to_string()))
            .bind(("status", status.to_string()))
            .await?;

        match result.take::<Option<models::UserSubscriptionResult>>(0)? {
            Some(updated) => Ok(updated),
            None => Err(ApiError::InternalServerError(
                "Failed to update subscription.".to_string(),
            )),
        }
    }

    pub async fn validate_subscription_status(&self, user_id: &str) -> Response<bool> {
        /*
            Checks the status of a user's subscription.

            Params:
                user_id (string): The user's Auth0 ID.

            Returns:
                Response<String>: The user's subscription status.

        */

        let mut result = self
            .db
            .query("SELECT subscription_status FROM type::thing('user', $user_id)->subscribed->subscription;")
            .bind(("user_id", user_id.to_string()))
            .await?;

        match result.take::<Option<models::SubscriptionStatus>>(0)? {
            Some(status) => {
                if status.subscription_status == "complete" {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Ok(false),
        }
    }

    pub async fn increment_usage(&self, user_id: &str) -> Response<models::UserSubscriptionResult> {
        /*
            Updates the usage of a user's subscription.

            Params:
                user_id (string): The user's Auth0 ID.
                usage (int): The new usage value.

            Returns:
                Response<models::UserSubscriptionResult>: The updated subscription object.

        */

        let mut result = self
            .db
            .query("UPDATE type::thing('user', $user_id)->subscribed->subscription SET usage = usage + 1;")
            .bind(("user_id", user_id.to_string()))
            .await?;

        match result.take::<Option<models::UserSubscriptionResult>>(0)? {
            Some(updated) => Ok(updated),
            None => Err(ApiError::InternalServerError(
                "Failed to update usage.".to_string(),
            )),
        }
    }

    pub async fn decrement_usage(&self, user_id: &str) -> Response<models::UserSubscriptionResult> {
        /*
            Updates the usage of a user's subscription.

            Params:
                user_id (string): The user's Auth0 ID.
                usage (int): The new usage value.

            Returns:
                Response<models::UserSubscriptionResult>: The updated subscription object.

        */

        let mut result = self
            .db
            .query("UPDATE type::thing('user', $user_id)->subscribed->subscription SET usage = usage - 1;")
            .bind(("user_id", user_id.to_string()))
            .await?;

        match result.take::<Option<models::UserSubscriptionResult>>(0)? {
            Some(updated) => Ok(updated),
            None => Err(ApiError::InternalServerError(
                "Failed to update usage.".to_string(),
            )),
        }
    }
}
