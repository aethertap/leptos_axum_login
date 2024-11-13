use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router_macro::path;
use crate::pages::{Login,Register};
use leptos_router::components::{Router,Routes,Route};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    //provide_meta_context();
    
    // I'm not sure where the `head` content comes from, or how to alter it.
    view! {
        <Router>
            <Title formatter=move |text| format!("{text} - Login stuff")/>
            <Routes fallback=|| "Page not found.".into_view()>
            <Route path=path!("/") view=HomePage/>
            <Route path=path!("/register") view=Register/>
            <Route path=path!("/login") view=Login/>
            </Routes>
        </Router>
    }
}

#[server(Ping)]
pub async fn ping() -> Result<String,ServerFnError> {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    Ok("Pong".into())
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    use leptos::either::Either;
    let ping = Resource::new(||(),move|_|ping());
    // Here's a demo of how require_login works. It will send the user to the login page if they
    // aren't already authorized, then when they get signed in they get sent back here. If they
    // aren't registered, the redirect doesn't currently survive the shuffle, sorry.
    let user = Resource::new_blocking(||(), move|_| crate::server::require_login(None));
    // This view would be rendered if the redirect didn't happen in a timely manner for some
    // reason. It might be rendered if it's done server-side, but I'm honestly not sure.
    let no_user = move || view! {
        <Title text="Home"/>
        <h1>"Aaah, no place like home."</h1>
    };
    view! {
        // The Suspense component hangs out with a fallback view until all of the async things come
        // in, then it switches over to the children view. In this case, it shows
        <Suspense fallback=no_user>
            {move || Suspend::new(async move {
                match user.await {
                    Ok(Some(user)) => {
                        Either::Left(
                            view! {
                                <Title text=move || format!("This is home.")/>
                                <h1>"Welcome home, " {user.username}</h1>
                            },
                        )
                    }
                    _ => Either::Right(no_user),
                }
            })}

        </Suspense>

        <Suspense fallback=move || view! { <p>"I said 'ping'....."</p> }>
            <p>"server says: '" <b>{move || Suspend::new(async move { ping.await })}</b> "'"</p>
        </Suspense>
    }
}


