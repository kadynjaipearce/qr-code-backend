use crate::database::models::{self, format_user_id};
use crate::errors::{ApiError, Response};
use crate::utils::Environments;

use surrealdb::engine::remote::ws::{Client, Wss};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use super::models::DynamicQrResult;

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

    pub async fn select_user(&self, id: &str) -> Response<Option<models::UserResult>> {
        /*
           Selects a user from the database with a id.

           Params:
               id (string): The user's Auth0 ID.

            Returns:
               Response<Option<models::UserResult>>: The selected user object, or None if no user was found.
        */

        let mut result = self
            .db
            .query("SELECT * FROM type::thing('user', $id);")
            .bind(("id", id.to_string()))
            .await?;

        match result.take::<Option<models::UserResult>>(0)? {
            Some(user) => Ok(Some(user)),
            None => Ok(None),
        }
    }

    // Dynamic URL CRUD operations.

    pub async fn insert_dynamic_url(
        &self,
        user_id: &str,
        dynamic_url: models::DynamicQr,
    ) -> Response<models::DynamicQrResult> {
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
        RELATE type::thing('user', $user)->created->CREATE type::thing('dynamic_url', uuid()) 
        SET server_url = $server_url, 
        target_url = $target_url, 
        created_at = time::now(), updated_at = time::now()",
            )
            .bind(("user", user_id.to_string()))
            .bind(("server_url", dynamic_url.server_url))
            .bind(("target_url", dynamic_url.target_url))
            .await?;

        match result.take::<Option<models::DynamicQrResult>>(0)? {
            Some(created) => Ok(created),
            None => Err(ApiError::InternalServerError(
                "Failed to create dynamic URL.".to_string(),
            )),
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
            .query("SELECT target_url FROM dynamic_url WHERE server_url = $server_url")
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
        qrcode_id: &str,
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
            .query("UPDATE dynamic_url SET target_url = $target_url, updated_at = time::now() WHERE id = $id")
            .bind(("id", qrcode_id.to_string()))
            .bind(("target_url", new_target_url.to_string()))
            .await?;

        match result.take::<Option<models::DynamicQrResult>>(0)? {
            Some(updated) => Ok(updated),
            None => Err(ApiError::InternalServerError(
                "No matching URL found.".to_string(),
            )),
        }
    }

    pub async fn delete_dynamic_url(&self, id: &str) -> Response<DynamicQrResult> {
        /*
            Deletes a dynamic URL from the database.

            Params:
                id (string): The ID of the dynamic URL to delete.

        */

        let mut result = self
            .db
            .query("DELETE dynamic_url WHERE id = $id")
            .bind(("id", id.to_string()))
            .await?;

        match result.take::<Option<models::DynamicQrResult>>(0)? {
            Some(deleted) => Ok(deleted),
            None => Err(ApiError::InternalServerError(
                "Failed to delete url.".to_string(),
            )),
        }
    }

    pub async fn lookup_subscription_id(&self, user_id: &str) -> Response<Option<String>> {
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

        match result.take::<Option<models::UserSubscriptionResult>>(0)? {
            Some(subscription) => Ok(Some(subscription.id.to_string())),
            None => Ok(None),
        }
    }

    pub async fn insert_subscription(
        &self,
        user_id: &str,
        subscription: models::UserSubscription,
    ) -> Response<models::UserSubscriptionResult> {
        /*
            Inserts a new user subscription into the database.

            Params:
                user_id (string): The user's Auth0 ID.
                subscription (models::UserSubscription): Contains:
                    - `tier`: The subscription tier.
                    - `usage`: The usage count.
                    - `start_date`: The start date of the subscription.
                    - `end_date`: The end date of the subscription.
                    - `subscription_status`: The status of the subscription.

            Returns:
                Response<models::UserSubscription>: The inserted user subscription object.

        */
        let mut result = self.db.query(
            "RELATE type::thing('user', $user)->subscribed->CREATE type::thing('subscription', $user) 
            SET subscription_id = $subscription_id,
            tier = $tier, 
            usage = 0, 
            start_date = time::now(), 
            end_date = NONE, 
            subscription_status = 'active'",
        )
        .bind(("subscription_id", subscription.id))
        .bind(("user", user_id.to_string()))
        .bind(("tier", subscription.tier)).await?;

        match result.take::<Option<models::UserSubscriptionResult>>(0)? {
            Some(created) => Ok(created),
            None => Err(ApiError::InternalServerError(
                "Failed to create subscription.".to_string(),
            )),
        }
    }

    pub async fn update_subscription(
        &self,
        user_id: &str,
        subscription: models::UpdateUserSubscription,
    ) -> Response<models::UserSubscriptionResult> {
        /*
            Updates a user subscription in the database.

            Params:
                user_id (string): The user's Auth0 ID.
                subscription (models::UserSubscription): Contains:
                    - `tier`: The subscription tier.
                    - `usage`: The usage count.
                    - `start_date`: The start date of the subscription.
                    - `end_date`: The end date of the subscription.
                    - `subscription_status`: The status of the subscription.

            Returns:
                Response<models::UserSubscription>: The updated user subscription object.

        */

        let mut result = self
            .db
            .query(
                "UPDATE subscription SET tier = $tier, usage = $usage, start_date = $start_date, end_date = $end_date, subscription_status = $subscription_status WHERE subscription_id = $subscription_id",
            )
            .bind(("subscription_id", subscription.id))
            .bind(("tier", subscription.tier))
            .bind(("usage", subscription.usage))
            .bind(("start_date", subscription.start_date))
            .bind(("end_date", subscription.end_date))
            .bind(("subscription_status", subscription.subscription_status))
            .await?;

        match result.take::<Option<models::UserSubscriptionResult>>(0)? {
            Some(updated) => Ok(updated),
            None => Err(ApiError::InternalServerError(
                "Failed to update subscription.".to_string(),
            )),
        }
    }
}
