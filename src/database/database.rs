use crate::database::models::{self, format_user_id};
use crate::errors::{ApiError, Response};
use crate::utils::Environments;

use surrealdb::engine::remote::ws::{Client, Wss};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

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
        DEFINE FIELD email ON user TYPE string ASSERT $value != NONE;
        DEFINE FIELD created_at ON user TYPE datetime ASSERT $value != NONE;

        DEFINE TABLE dynamic_url SCHEMAFULL;
        DEFINE FIELD id ON dynamic_url TYPE string ASSERT $value != NONE;
        DEFINE FIELD server_url ON dynamic_url TYPE string ASSERT $value != NONE;
        DEFINE FIELD target_url ON dynamic_url TYPE string ASSERT $value != NONE;
        DEFINE FIELD created_at ON dynamic_url TYPE datetime ASSERT $value != NONE;
        DEFINE FIELD updated_at ON dynamic_url TYPE datetime ASSERT $value != NONE; 
        ",
        )
        .await?;

        // Return a new instance of the Database struct with the established connection.
        Ok(Database { db })
    }

    pub async fn list_user_urls(&self, user_id: &str) -> Response<Vec<models::DynamicUrlResult>> {
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

        let created = result.take::<Vec<models::DynamicUrlResult>>(0)?;

        if created.is_empty() {
            Err(ApiError::InternalServerError(
                "User has no dynamic urls.".to_string(),
            ))
        } else {
            Ok(created)
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
            .query("CREATE type::thing('user', $id) SET email = $email, created_at = time::now();")
            .bind(("id", format_user_id(user.id)))
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
        dynamic_url: models::DynamicUrl,
    ) -> Response<models::DynamicUrlResult> {
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
            .bind((
                "user",
                format_user_id(user_id.to_string()),
            ))
            .bind(("server_url", dynamic_url.server_url))
            .bind(("target_url", dynamic_url.target_url))
            .await?;

        match result.take::<Option<models::DynamicUrlResult>>(0)? {
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
        server_url: &str,
        new_target_url: &str,
    ) -> Response<models::DynamicUrlResult> {
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

        match result.take::<Option<models::DynamicUrlResult>>(0)? {
            Some(updated) => Ok(updated),
            None => Err(ApiError::InternalServerError(
                "No matching URL found.".to_string(),
            )),
        }
    }

    pub async fn delete_dynamic_url(&self, id: &str) -> Response<()> {
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

        match result.take::<Option<models::DynamicUrlResult>>(0)? {
            Some(_) => Ok(()),
            None => Err(ApiError::InternalServerError(
                "Failed to delete url.".to_string(),
            )),
        }
    }
}
