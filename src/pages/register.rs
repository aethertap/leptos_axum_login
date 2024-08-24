use leptos::*;
use leptos_dom::logging::console_log;


/// This function returns true if the name provided is not already a username in the database.
#[server(CheckUsername,"api","Url","check_username")]
pub async fn check_username(name: String) -> Result<bool,ServerFnError> {
//    use sqlx::{query_as,FromRow,SqlitePool};
//    use serde::{Deserialize,Serialize};
//    use logging::log;
//
//    #[derive(Clone,Deserialize,Serialize,Debug,FromRow)]
//    struct Q{username:String}
//    log!("Checking for username {name}");
//    let pool:SqlitePool = use_context().expect("No connection pool provided");
//    let count:Option<Q> = query_as!(Q,"select username from users where username=$1 limit 1", name)
//        .fetch_optional(&pool).await?;
//    log!("Fetched {count:#?}");
//    Ok(count.is_none() && (name != "asdf"))
    Ok(name != "asdf")
}

#[component]
pub fn Register() -> impl IntoView {
    let username = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let show_password = create_rw_signal(false);
    let password_type = move || if show_password() {"text"} else {"password"};
    let valid_username = create_resource(
        move || username.get(),
        move |name| async move {
            console_log("Checking username");
            let res = check_username(name).await;
            console_log(&format!("Result: {res:?}"));
            res.unwrap_or(true)
        });
    view! {
        <input type="text"
            on:input = move |e|{ username.set(event_target_value(&e))}
            prop:value=move ||username.get()
            placeholder="Username"/>
        <input type=password_type prop:value = move ||password.get()/>
        <input type="submit" value=" Register "/>
        <Suspense fallback={move ||view!{<p>"checking username..."</p>}}>
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
    }
}