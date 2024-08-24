use leptos::*;
use crate::user::User;
use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(feature="ssr")] {
        use sqlx::SqlitePool;
        use crate::{sqlite_backend::SqliteBackend};
        use axum_login::AuthSession;
        use leptos_axum;
        use logging::log;
    }
}


#[server(GetUser,"api","Url","get_user")]
pub async fn get_user() -> Result<Option<User>,ServerFnError> {
    let session: AuthSession<SqliteBackend> = use_context().expect("session not provided");
    log!("Session user: {:#?}", session.user);
    Ok(session.user.clone())
}


#[server(Login,"api","Url","login")]
pub async fn login(username: String, password: String) -> Result<User,ServerFnError> {
    let conn:SqlitePool = use_context().expect("no database connection");
    todo!()
}
