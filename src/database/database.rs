use crate::database::models;
use crate::errors::Response;
use crate::utils::Environments;

use surrealdb::engine::remote::ws::{Client, Wss};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use super::models::DynamicUrl;

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
            .bind(("id", user.id))
            .bind(("email", user.email))
            .await?;

        let created: Option<models::UserResult> = result.take(0)?;
        Ok(created.unwrap())
    }

    pub async fn select_user(&self, id: String) -> Response<Option<models::UserResult>> {
        /*
           Selects a user from the database with a id.

           Params:

        */
        let result: Option<models::UserResult> = self
            .db
            .query("SELECT * FROM type::thing('user', $id);")
            .bind(("id", id))
            .await?
            .take(0)?;

        Ok(result)
    }

    // todo: implement dynamic_url database interactions.

    pub async fn insert_dynamic_url(
        &self,
        dynamic_url: DynamicUrl,
    ) -> Response<models::DynamicUrlResult> {
        let mut result = self.db.query("
        CREATE type::thing('dynamic_url', uuid()) 
        SET server_url = $server_url, 
        target_url = $target_url, 
        created_at = time::now(), updated_at = time::now()")
        .bind(("server_url", dynamic_url.server_url))
        .bind(("target_url", dynamic_url.target_url))
        .await?;
        
        let created: Option<models::DynamicUrlResult> = result.take(0)?;
        Ok(created.unwrap())
    }
}
