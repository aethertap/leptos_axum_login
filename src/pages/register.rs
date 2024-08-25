use leptos::*;
use leptos_router::*;
use leptos_dom::logging::console_log;

use crate::auth::Register;


/// This function returns true if the name provided is not already a username in the database, and isn't
/// "asdf", which should obviously never be allowed.
#[server(CheckUsername,"/api")]
pub async fn check_username(name: String) -> Result<bool,ServerFnError> {
    use sqlx::{query_as,FromRow,SqlitePool};
    use serde::{Deserialize,Serialize};
    use logging::log;

    /// This struct just serves as a pattern to lay the returned data over so that I can use
    /// `query_as!`. All I actually want to know is whether that username exists.
    #[derive(Clone,Deserialize,Serialize,Debug,FromRow)]
    struct Q{id:i64}
    log!("(Server) Checking for username {name}");
    // Burst into flames if there's no database pool. This should probably be more robust in a real
    // application.
    let pool:SqlitePool = use_context().expect("No connection pool provided");
    let count:Option<Q> = query_as!(Q,"select id from users where username=$1 limit 1", name)
        .fetch_optional(&pool).await?;
    log!("Fetched {count:#?}");
    Ok(count.is_none() &&
        (name != "asdf") // ban it!
    )
}

/// This function shows a clean, modern ui with no styling and no layout. It looks suspiciously
/// similar to the login page, but the action is different. This will check to see if a username
/// is taken, and if not it will allow you to register with that name. If not, it will still allow
/// you to try, but the server will complain.
#[component]
pub fn Register() -> impl IntoView {
    let username = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let show_password = create_rw_signal(false);
    let password_type = move || if show_password() {"text"} else {"password"};
    // I'm creating this with `create_local_resource` so that it can be used outside of a suspense
    // without causing hydration bugs. `create_local_resource` waits to do anything until the app is
    // hydrated on the client in order to prevent hydration mismatch errors. This would be a performance
    // problem normally, but in this case this resource is only ever triggered after user input, which means
    // the app is definitely hydrated.
    let valid_username = create_local_resource(
        move || username.get(),
        move |name| async move {
            console_log(&format!("Checking username {name}"));
            let res = check_username(name).await;
            console_log(&format!("Result: {res:?}"));
            res.unwrap_or(true)
        });
    let register = create_server_action::<Register>();
    create_effect(move |_| {
        if let Some(Ok(Some(_))) = register.value()() {
            console_log("Got a user logged in");
        }
    });
    view! {
        <ActionForm action=register>
            <input type="text"
                name="username"
                on:input = move |e|{ username.set(event_target_value(&e))}
                prop:value=move ||username.get() // prop:value lets us react to changes, value wouldn't. See leptos book.
                placeholder="Username"/>
            <input type=password_type
                name="password"
                prop:value = move ||password.get()/>
            <a on:click=move |ev|{
                    ev.prevent_default();
                    show_password.update(|s| *s = !*s);
            }>{move ||if show_password() {" Hide "} else {" Show "}}</a>
            // I can use valid_username here because it was made with create_local_resource. If it was
            // just create_resource, it would throw warning messages in the browser console.
            <input type="submit" disabled=move||{valid_username().map(|b| !b)} value=" Register "/>
            <Suspense fallback={move ||view!{<p>"checking username..."</p>}}>
                // We have a resource to tell us whether we can have a username or not, so we need to
                // use it in a suspense
                {
                    move || {
                        if Some(true) == valid_username() {
                            view!{<p>"That'll work"</p>}
                        } else {
                            view!{<p>"Nope, can't have that one"</p>}
                        }
                    }
                }
            </Suspense>
        </ActionForm>
    }
}
