use leptos::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(feature="ssr")] {
        use leptos::logging::log;
        use crate:: state::AppState ;
        //use axum_extra::extract::cookie::{Cookie,CookieJar};
    }
}


    
// Select a user with the given username from the db. If they exist, return true. Otherwise,
// return false. In otherwords, return true if the username is in the databse. 
#[server(UserExists, "/api","Url","user_exists")]
pub async fn user_exists(user:String) -> Result<bool, ServerFnError> {
    use sqlx::{query_as,FromRow};

    log!("checking username {user}");
    let pbox:AppState = use_context().expect("No database pool provided in context");
    
    #[derive(Clone,FromRow)]
    struct Uid {
        // this holds the user id, but it's never used in here. I promised myself I'd kill all the
        // unused stuff, but this one just seems like it's useful as an example so I'm keeping it.
        #[allow(unused)]
        pub id: i64,
    }

    let exists:Option<Uid> = query_as!(Uid, "select id from users where username=$1", user)
        .fetch_optional(&pbox.pool).await?;
    Ok(exists.is_some())
}
