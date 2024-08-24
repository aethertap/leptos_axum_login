use leptos::*;
use crate::{error_template::AppError, user::User};
use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(feature="ssr")] {
        use sqlx::SqlitePool;
        use crate::{sqlite_backend::SqliteBackend};
        use axum_login::AuthSession;
        use leptos_axum;
        use logging::log;

        use argon2::{
            password_hash::{
                rand_core::OsRng,
                PasswordHash, PasswordHasher, PasswordVerifier, SaltString
            },
            Argon2
        };

    }
}


#[server(GetUser,"/api","Url","get_user")]
pub async fn get_user() -> Result<Option<User>,ServerFnError> {
    let session: AuthSession<SqliteBackend> = use_context().expect("session not provided");
    log!("Session user: {:#?}", session.user);
    Ok(session.user.clone())
}


#[server(Login,"/api","Url","login")]
pub async fn login(username: String, password: String) -> Result<Option<User>,ServerFnError> {
    let conn:SqlitePool = use_context().expect("no database connection");
    todo!()
}

#[server(Register,"/api","Url","register")]
pub async fn register(username: String, password: String) -> Result<Option<User>,ServerFnError> {
    let pool:SqlitePool = use_context().expect("no database connection");
    let argon = argon2::Argon2::default();
    let pass_salt = SaltString::generate(&mut OsRng);
    let pass_hash = argon.hash_password(password.as_bytes(), &pass_salt)
        .map_err(|e| AppError::Internal(format!("Password hash failed: {e}")))?.to_string();
    let pass_salt_str = pass_salt.as_str().to_owned();
    // check to make sure the user doesn't exist, then insert.
    struct InsertUser{
        pub id:i64
    };
    let new_id:InsertUser = sqlx::query_as!(InsertUser, "insert into users (username,pass_hash,pass_salt) values ($1,$2,$3) returning id",
        username,
        pass_hash,
        pass_salt_str,
    ).fetch_one(&pool).await?;
    log!("Created user {username} with id {}",new_id.id);
    Ok(Some(User{
        id:new_id.id,
        username,
        session_auth_hash:pass_hash.as_bytes().to_owned(),
    }))
}
