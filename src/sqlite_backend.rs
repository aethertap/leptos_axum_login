
use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(feature="ssr")] {
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

    }
}
use crate::error_template::AppError;

pub static DB_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"),"/db/database.sqlite3");
pub static MIGRATIONS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"),"/db/migrations");

/// This is a barebones example of an authentication backend using sqlite3.
#[derive(Clone,Debug)]
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
        // This does the hashing
        let argon2 = Argon2::default();
        // The salt is used to prevent certain attacks against stored passwords (see the Internet for more)
        let salt = SaltString::generate(&mut OsRng);
        // I need to actually *store* the salt in the database. Well, at least I want to. The salt is actually
        // included in the password hash string representation so it's probably not strictly necessary, but old
        // habits.
        let pass_salt_str = salt.as_str();
        // This gives back a data structure with various parts, which can be encoded using
        // a standard format into a string that's suitable for use in plain-text environments. Argon2id is the
        // recommended hashing algorithm at the time of this code being published (2024)
        let pass_hash:PasswordHash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Password hashing error: {e}")))?;
        // Now *this* part is what will be put directly into the database as the user's password hash. This is not just
        // the 32-byte hash function output, it also has other data attached (like the salt). It has to have
        // a let-binding outside of the macro or the compiler complains.
        let pass_hash_str = pass_hash.to_string();
        /// This struct lets the query_as! macro return the new rowid to me.
        #[derive(Debug)]
        struct InsertUser{
            /// The row_id from sqlite. Other databases will have other ways of returning this to you.
            pub id:i64
        }
        let new_id:InsertUser = sqlx::query_as!(InsertUser, "insert into users (username,pass_hash,pass_salt) values ($1,$2,$3) returning id",
            username,
            pass_hash_str,
            pass_salt_str,
        ).fetch_one(&self.pool).await
        .map_err(|e| AppError::Internal(format!("Error inserting user: {e}")))?;

        // Now we need to make sure we can make a good session key. In this case, we're using the raw bytes
        // that were output from the password hash (in this case, 32 bytes). This does *not* include the salt
        // or other associated data that's bulit into the pass_hash_str
        let hash_bytes = pass_hash.hash.unwrap().as_bytes().to_owned();
        Ok(Some(User{
            id:new_id.id,
            username,
            session_auth_hash: hash_bytes,
        }))
    }
}

/// The `AuthnBackend` is the part that handles autheNtication (proving that a user's identity
/// is valid). The `AuthzBackend` handles authoriZation (permissions granted to a user whose
/// identity is already known). I'm only doing authentication for this example. At least for now.
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
            let hasher = Argon2::default();
            let hash = PasswordHash::parse(user.pass_hash.as_ref(),password_hash::Encoding::B64)
                .map_err(|e| AppError::Internal(format!("Corrupted password hash: {e}")))?;
            // Use the existing implementation to verify the password. I was doing this myself until
            // I noticed that there is a PasswordVerifier trait, so this is better in every way.
            if let Ok(()) = hasher.verify_password(password.as_bytes(), &hash) {
                return Ok(Some(user.to_user()?))
            }
        }
        Ok(None)
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
