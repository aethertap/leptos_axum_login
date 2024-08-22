use leptos::*;
use crate::user::User;
use cfg_if::cfg_if;
cfg_if!{
    if #[cfg(feature="ssr")] {
        use sqlx::SqlitePool;
    }
}


#[server(Login,"api","Url","login")]
pub async fn login(username: String, password: String) -> Result<User,ServerFnError> {
    let conn:SqlitePool = use_context().expect("no database connection");
    todo!()
}
