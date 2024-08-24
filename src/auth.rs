use leptos::*;
use crate::{error_template::AppError, user::User};
use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(feature="ssr")] {
        use sqlx::SqlitePool;
        use crate::{sqlite_backend::SqliteBackend};
        use axum_login::{AuthSession,AuthnBackend};
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
    let mut auth :AuthSession<SqliteBackend> = use_context().unwrap();//leptos_axum::extract().await?;
    let mut session:tower_sessions::Session = use_context().unwrap();//leptos_axum::extract().await?;
    log!("Logging in user as '{username}'/'{password}'");
    log!("Session id = {:?}",session.id());
    if username.is_empty() {
        return Ok(None)
    }
    let user = auth.backend.authenticate((username,password)).await?;
    if let Some(user) = user.as_ref() {
        auth.login(user).await;
        Ok(Some(user.clone()))
    } else {
        Ok(None)
    }
}

#[server(Register,"/api","Url","register")]
pub async fn register(username: String, password: String) -> Result<Option<User>,ServerFnError> {
    let mut auth_session:AuthSession<SqliteBackend> = use_context().expect("auth-session not provided");
    let session:tower_sessions::Session = use_context().unwrap();
    let pool:SqlitePool = use_context().expect("no database connection");
    let argon = argon2::Argon2::default();
    let pass_salt = SaltString::generate(&mut OsRng);
    let pass_hash = argon.hash_password(password.as_bytes(), &pass_salt)
        .map_err(|e| AppError::Internal(format!("Password hash failed: {e}")))?.to_string();
    let pass_salt_str = pass_salt.as_str().to_owned();
    // check to make sure the user doesn't exist, then insert.
    let user = auth_session.backend.add_user(username,password).await?;
//    let user = User{
//        id:new_id.id,
//        username,
//        session_auth_hash:pass_hash.as_bytes().to_owned(),
//    };
    log!("get_user returned {user:#?}");
    if let Some(user) = user {
        auth_session.login(&user).await;
        log!("AuthSession user after register: {}", auth_session.user.as_ref().unwrap().username);
        log!("Register - session id = {:#?}", session.id());
        Ok(Some(user))
    } else {
        Ok(None)
    }
}
