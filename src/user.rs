
use serde::{Serialize,Deserialize};
use crate::error_template::AppError;

use cfg_if::cfg_if;
cfg_if!{
    if #[cfg(feature="ssr")] {
        use axum_login::AuthUser;
        use sqlx::prelude::FromRow;
        use password_hash::PasswordHash;

        impl AuthUser for User {
            type Id=String;
            fn id(&self) -> Self::Id {
                self.username.clone()
            }

            fn session_auth_hash(&self) -> &[u8] {
                self.session_auth_hash.as_ref()
            }
        }

        #[derive(Clone,PartialEq,Debug,FromRow)]
        pub struct SqlUser {
            pub username: String,
            pub pass_hash: String,
            pub pass_salt: String,
        }

        impl SqlUser {
            pub fn to_user(self) -> Result<User,AppError> {
                let PasswordHash{hash,..} = PasswordHash::parse(&self.pass_hash,password_hash::Encoding::B64)
                    .map_err(|e| AppError::Internal(format!("Decode password: {e}")))?;
                let hash:Vec<u8> = hash.map(|output| {
                    output.as_bytes().to_owned()
                }).ok_or_else(||AppError::Internal("Badly formatted password hash".into()))?;

                Ok(User {
                    username: self.username,
                    session_auth_hash: hash,
                }
            )
            }
        }
    }
}


#[derive(Clone,PartialEq,Debug,Serialize,Deserialize)]
pub struct User {
    /// User-facing username, probably unique
    pub username: String,

    /// This is computed with Argon2id
    pub session_auth_hash: Vec<u8>,
}
