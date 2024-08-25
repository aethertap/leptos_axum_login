use leptos::*;
use crate::user::User;
use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(feature="ssr")] {
        use crate::{sqlite_backend::SqliteBackend};
        use axum_login::{AuthSession,AuthnBackend};
        use logging::log;
    }
}

/// get_user tries to retrieve the user from the session. If there is a logged-in user,
/// it will return `Some(user)`, otherwise it returns `None`. This is useful for checking
/// login status in components before rendering stuff that either assumes a user, or shouldn't
/// be accessible to the unauthorized.
  #[server(GetUser,"/api","Url","get_user")]
pub async fn get_user() -> Result<Option<User>,ServerFnError> {
    let session: AuthSession<SqliteBackend> = use_context().expect("session not provided");
    log!("Session user: {:#?}", session.user);
    Ok(session.user.clone())
}

/// Check the credentials and log the user in. This is the central purpose of this example! See the pages/login.rs
/// file for an example of how this one is used.
#[server(Login,"/api","Url","login")]
pub async fn login(username: String, password: String) -> Result<Option<User>,ServerFnError> {
    // Note that you can still use `leptos_axum::extract().await?` if you want, but since we
    // called `provide_context` from the `server_fn_handler` in `main`, we can do it this way
    // and it feels faster. Get the AuthSession.
    let mut auth :AuthSession<SqliteBackend> = use_context().unwrap();
    // If you want access to the actual session, you'll have to extract it separately because I couldn't
    // find a good way to get it out of the auth session.
    let session:tower_sessions::Session = use_context().unwrap();//leptos_axum::extract().await?;
    // Advanced debugging tools
    log!("Logging in user as '{username}'/'{password}'");
    log!("Session id = {:?}",session.id());
    // The SqliteBackend we defined has the `Self::Credential` type set to a `(String,String)` tuple
    // which is meant to be the username/password pair. This is just an example, you probably want
    // something more robust to handle different auth scenarios like Oauth and whatnot. Maybe I'll add
    // those in later if I can figure out how.
    let user = auth.backend.authenticate((username,password)).await?;

    // If the authentication was successful, we actually have to tell the AuthSession that the user
    // is now logged in. This happens when we call `auth.login(user)`. This will also be the first
    // place where you actually get a session id sent back to the browser unless you've done other stuff
    // with your sessions elsewhere.
    if let Some(user) = user.as_ref() {
        auth.login(user).await?;
        Ok(Some(user.clone()))
    } else {
        // If anything else happened other than a successful auth, just return a failure.
        Ok(None)
    }
}


/// Add a user to the database and log them in, because I get annoyed by sites that let me register and then
/// make me log in separately after that. Give me a break! This function is called from the Register component
/// which is in pages/register.rs.
#[server(Register,"/api","Url","register")]
pub async fn register(username: String, password: String) -> Result<Option<User>,ServerFnError> {
    // Extract the auth_session and session. You could also use `leptos_axum::extract().await` here,
    // but this seems nicer.
    let mut auth_session:AuthSession<SqliteBackend> = use_context().expect("auth-session not provided");
    let session:tower_sessions::Session = use_context().unwrap();
    // The backend handles all of the password hashing and whatnot. Just call add_user and then go write
    // the backend, and it's all done!
    let user = auth_session.backend.add_user(username,password).await?;

    log!("get_user returned {user:#?}");
    if let Some(user) = user {
        // Tell the AuthSession that we're logged-in now and it should behave accordingly. This will set the
        // session id and send it to the browser as a side-effect (before now you likely had no session id in the browser).
        auth_session.login(&user).await?;
        log!("AuthSession user after register: {}", auth_session.user.as_ref().unwrap().username);
        log!("Register - session id = {:#?}", session.id());
        Ok(Some(user))
    } else {
        // Something went wrong? Fail silently!
        Ok(None)
    }
}
