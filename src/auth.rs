use leptos::*;
use sqlx::SqlitePool;
use crate::user::User;


#[server(Login,"api","Url","login")]
pub async fn login(username: String, password: String) -> Result<User,ServerFnError> {
    let conn:SqlitePool = use_context().expect("no database connection");
    todo!()
}
