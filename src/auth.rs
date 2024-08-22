use leptos::*;
use sqlx::SqlitePool;
use crate::user::User;
use crate::sqlite_backend::SqliteBackend;

#[server(Login,"auth","Url","login")]
pub async fn login(username: String, password: String) -> Result<User,ServerFnError> {
    let conn:SqlitePool = use_context().expect("no database connection");
    todo!()
}
