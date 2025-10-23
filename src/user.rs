
use serde::{Serialize,Deserialize};


pub type DatabaseId = i64;

use cfg_if::cfg_if;
cfg_if!{
    if #[cfg(feature="ssr")] {
        //use leptos::prelude::*;
        use axum_login::AuthUser;
        use leptos::logging::log;
        use crate::error_template::AppError;
        use sqlx::prelude::FromRow;
        use password_hash::PasswordHash;

        // This trait is used by axum_login to keep track of whether
        // a user is authenticated or not.
        impl AuthUser for User {
            type Id=String;
            /// id needs to return something that can uniquely identify the user.
            /// I could use the id field, but I'm using the username so I can spot it
            /// in data to see where it gets sent.
            fn id(&self) -> Self::Id {
                self.username.clone()
            }

            /// I *think* this is what is used to generate the actual session id that
            /// makes it way into the cookie. One thing for sure: it needs to be the same
            /// every time you call get_user, or it will invalidate your session. I had a
            /// bug in which I was storing and loading the whole password hash, but only
            /// checking the actual hash data, and it would log a user in and work as long
            /// as they stayed on the same page. As soon as they navigated away from the page,
            /// it would load the buggy id from the database and kill the session.
            fn session_auth_hash(&self) -> &[u8] {
                self.session_auth_hash.as_ref()
            }
        }

        /// SqlUser represents a row from the users table. It's only used by the
        /// SqliteBackend in order to support `get_user` and `authenticate`
        #[derive(Clone,PartialEq,Debug,FromRow)]
        pub struct SqlUser {
            pub id: DatabaseId,
            pub username: String,
            pub pass_hash: String,
        }

        impl SqlUser {

            /// Convert the database row into a user object that the AuthSession
            /// can use.
            pub fn to_user(self) -> Result<User,AppError> {
                // parse the hash data out of the string representation that we kept in the database
                let PasswordHash{hash,..} = PasswordHash::parse(&self.pass_hash,password_hash::Encoding::B64)
                    .map_err(|e| AppError::InternalError(format!("Decode password: {e}")))?;
                // This is where we dig into the password hash data structure and pull out just
                // the actual hash bytes that came out of argon2. These are used to identify the session
                // so that this user always gets the same session data.
                let hash:Vec<u8> = hash.map(|output| {
                    output.as_bytes().to_owned()
                }).ok_or_else(||AppError::InternalError("Badly formatted password hash".into()))?;
                
                log!("Got user {self:?}");
                Ok(User {
                    id: self.id,
                    username: self.username,
                    session_auth_hash: hash,
                }
            )
            }
        }
    }
}


/// This is used by AuthSession to keep track of a user's authentication
/// status. If the user is authenticated, AuthSession.user will be Some(User).
/// If not, the AuthSession.user will be None.

#[derive(Clone,PartialEq,Debug,Serialize,Deserialize)]
pub struct User {
    /// The database id for this user
    pub id: DatabaseId,

    /// User-facing username, has a unique constraint in the db so we can use it to id users
    pub username: String,

    /// This is computed with Argon2id, but it's only a *piece* of the entire thing returned
    /// by the hash function. You should be able to use whatever you want here as long as you
    /// can keep it stable between page loads. Personally, I don't like using the password hash
    /// but that's how they do it in the example so it's probably fine.
    pub session_auth_hash: Vec<u8>,
}

