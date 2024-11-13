use leptos::prelude::*;
use leptos::logging::log;
use crate::user::User;
use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(feature="ssr")] {
        use super::{sqlite_backend::SqliteBackend};
        use axum_login::{AuthSession,AuthnBackend};
    }
}


/// require_login returns Some(user) if the user is logged in, and returns None otherwise. As a
/// side-effect, it redirects the user to `/login` so that access can be authorized. By default,
/// the login will return the user to the location where this happened. You can override this by
/// providing Some(return_url) as the argument. To see this amazing function in action, look at
/// app::HomePage.
#[allow(unused)] // not sure why I have to put this here, but rustc complains about it if I don't.
// This function is used by both the lib and main.
pub async fn require_login(mut next:Option<String>) -> Result<Option<User>,ServerFnError> {
    use leptos_router::hooks::use_location;
    use leptos_router::location::Location;
    use urlencoding::encode;
    // Figure out what needs to be given as the next url if the user successfully logs in (or gets
    // registered)
    let return_to = match next.take() {
        Some(s) => s,
        None => {
            // Extract the stuff I need from the current requested location. I need this in order to
            // reconstruct the path so that the login_user can redirect them to it after a successful
            // login.
            let Location{pathname,search,hash,..} = use_location();
            // ret_string is the path that the user was trying to reach before being smacked down by
            // security. If they get past the guardian, then I'll send them there aferward.
            format!("{}{}{}",pathname.get_untracked(),search.get_untracked(),hash.get_untracked())
        }
    };
    if let Some(user) = get_user().await? {
        log!("require_login found user {}", user.username);
        return Ok(Some(user))
    } else {
        use leptos_router::hooks::use_navigate;
        log!("require_login: no logged-in user, redirecting to {return_to}");
        // I want to be able to redirect, this is the way. 
        let nav = use_navigate();
        // 'c' in this stands for "next". Or maybe "continue", something like that... This call
        // sends the user to the login page.
        nav(&format!("/login?c={}",encode(&return_to)),Default::default());
        return Ok(None);
    }
}
    
/// get_user tries to retrieve the user from the session. If there is a logged-in user,
/// it will return `Some(user)`, otherwise it returns `None`. This is useful for checking
/// login status in components before rendering stuff that either assumes a user, or shouldn't
/// be accessible to the unauthorized.
#[server(GetUser,"/api","Url","get_user")]
pub async fn get_user() -> Result<Option<User>,ServerFnError> {
    let session: AuthSession<SqliteBackend> = use_context().expect("session not provided");
    //log!("Session user: {:#?}", session.user.as_ref().map(|u| u.username));
    Ok(session.user.clone())
}

/// Check the credentials and log the user in. This is the central purpose of this example! See the pages/login.rs
/// file for an example of how this one is used.
#[server(LoginUser,"/api","Url","login")]
pub async fn login_user(username: String, password: String) -> Result<Option<User>,ServerFnError> {
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
#[server(RegisterNewUser,"/api","Url","register")]
pub async fn register_new_user(username: String, password: String) -> Result<Option<User>,ServerFnError> {
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
        log!("calling auth_session.login(user)");
        dbg!(auth_session.login(&user).await)?;
        log!("AuthSession user after register: {}", auth_session.user.as_ref().unwrap().username);
        log!("Register - session id = {:#?}", session.id());
        Ok(Some(user))
    } else {
        // Something went wrong? Fail silently!
        Ok(None)
    }
}


