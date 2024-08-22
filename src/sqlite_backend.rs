
use cfg_if::cfg_if;

use axum_login::{AuthnBackend, UserId};
use sqlx::SqlitePool;
use async_trait::async_trait;
use crate::user::*;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use crate::error_template::{self, AppError};

pub static DB_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"),"/db/database.sqlite3");
pub static MIGRATIONS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"),"/db/migrations");

/// This is a barebones example of an authentication backend using sqlite3.
#[derive(Clone)]
pub struct SqliteBackend {
    pub pool: SqlitePool,
}

impl SqliteBackend {
    /// Create a new instance of the backend. The database path is statically given here
    /// for convenience but should probably come from a config in a real app. If it manages
    /// to open the db, it returns `Ok(Self)`. Don't forget to call `.migrate()` on the backend
    /// before you use it to make sure any changes to the db are reflected here.
    pub async fn new() -> Result<Self,AppError> {
        let pool = SqlitePool::connect(DB_PATH).await
            .map_err(|e| AppError::Internal(format!("{e}")))?;
        Ok(SqliteBackend{pool})
    }

    /// Run `sqlx::migrate!` to make sure the database is up to date with the expected
    /// schema.
    pub async fn migrate(&self) -> Result<(),AppError> {
        Ok(sqlx::migrate!("db/migrations")
            .run(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("In migrations: {e}")))?)
    }

    /// Insert a new user into the database. Success only if the user doesn't already exist
    /// and the data meets criteria (which are *very* weak in this example!).
    pub async fn add_user(&self, username: String, password: String) -> Result<Option<User>, AppError> {
        // First validate the data. You must do better than this.
        if username.len() < 2 || password.len() < 2 {
            return Err(AppError::Invalid("Username and password have to be at least 2 characters each!".into()));
        }
        // Hash the password and insert the new user.
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let salt64 = salt.as_str();
        let hash:PasswordHash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Password hashing error: {e}")))?;
        let hash_str = hash.to_string();
        sqlx::query!("insert into users (username,pass_hash,pass_salt) values ($1,$2,$3)",
            username,
            hash_str,
            salt64
        ).execute(&self.pool).await
        .map_err(|e| AppError::Internal(format!("Error inserting user: {e}")))?;
        todo!();
    }
}

#[async_trait]
impl AuthnBackend for SqliteBackend {
    type User = crate::user::User;
    type Credentials = (String,String);
    type Error = crate::error_template::AppError;


    /// `authenticate` looks up the user by name, then checks the given password against the
    /// salted hash in the database to see if it matches. If so, you get the user back. If not,
    /// you get Ok(None). An Err value means something went wrong with the process, not that
    /// the authentication failed.
    async fn authenticate(&self, (username,password): Self::Credentials)
    -> Result<Option<Self::User>,Self::Error> {
        let mut user:Option<SqlUser> =  sqlx::query_as!(SqlUser,
                "select * from users where username = $1", username)
            .fetch_optional(&self.pool).await
            .map_err(|e| AppError::Internal(format!("Fetch user: {e}")))?;
        if let Some(user) = user.take() {
            let argon2 = Argon2::default();
            let salt = SaltString::from_b64(&user.pass_salt)
                .map_err(|e| AppError::Internal(format!("Invalid Salt: {e}")))?;
            let verify_hash = argon2.hash_password(password.as_bytes(),&salt)
                .map_err(|e| AppError::Internal(format!("Failed hash: {e}")))?
                .to_string();
            if verify_hash == user.pass_hash {
                Ok(Some(user.to_user()?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Return Some(user) if the user exists, otherwise return None. Only return an Err value if
    /// something actually goes wrong in the process. The user object returned from this will be
    /// sent to the authenticate function along with credentials later, so it is most convenient
    /// if the User has the password validation data attached to it in order to avoid another trip
    /// to the database.
    async fn get_user(&self, user_id: &UserId<Self>)
    -> Result<Option<Self::User>,Self::Error> {
        // The stored type in the database isn't the same as what the app uses, so I have a
        // separate query type (SqlUser) that gets converted.
        let mut user:Option<SqlUser> = sqlx::query_as!(SqlUser,
            "select * from users where username = $1", user_id
        ).fetch_optional(&self.pool).await
        .map_err(|e| AppError::Internal(format!("Fetch user: {e}")))?;
        // If there is something here, then the user exists and needs to be converted to the
        // version that the app can work with. Otherwise, no such user exists.
        if let Some(user) = user.take() {
            Ok(Some(user.to_user()?))
        } else {
            Ok(None)
        }
    }
}
