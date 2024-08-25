
use leptos::*;
use leptos_router::*;

use crate::auth::{Login,get_user};


/// This renders a complex and beautiful reactive HTML user interface, consisting of two
/// text boxes and a butto with no styling. However, it does detect when you're logged in
/// and even tells you who you are!
#[component]
pub fn Login() -> impl IntoView {
    // username wil be sent to the server every time it changes so we can get some of that
    // cool reactive feedback to tell us our username is already taken (becase it is, always).
    // This is how sites get you to add 7 random digits to your name that you'll never rememeber.
    let username = create_rw_signal(String::new());
    // This one *could* have some smart password strength stuff that forces you to make
    // impossible-to-type-on-mobile passwords. However, that's not yet implemented.
    let password = create_rw_signal(String::new());
    // For those of us who can't type on mobile, sometimes you want to see what you entered
    // for the password to check your work. This is the signal that will let you do that, but
    // it' not hooked up at the moment.
    let show_password = create_rw_signal(false);
    // When you change `show_password`, you *actually* need to change the `type` field of the
    // password text box. This gives you the type field to use as a derived signal.
    let password_type = move || if show_password() {"text"} else {"password"};
    // when the submit button is pressed, the ActionForm will want to invoke a server action
    // corresponding to a server function that does something. This is where we make that action.
    let login = create_server_action::<Login>();
    // Whenever the server action has a new value to report, we use that to trigger a call to `get_user`
    // so we can see who we are now. This is a `create_resource` because it doesn't need to be used outside
    // of the Suspense. If you want to use a resource without wrapping a Suspense around it, you'll need to use
    // create_local_resource so that it will wait until the app is hydrated before triggering. There's an example
    // of that in login.rs.
    let status = create_resource(login.value(), |_| get_user());
    view! {
        <ActionForm action=login>
            <input type="text"
                name="username" // don't forget the name field, or your server function won't work. Yes, I did.
                on:input = move |e|{ username.set(event_target_value(&e))}
                    prop:value=move ||username.get() // prop:value is reactive-capable, value is not. see the leptos book.
                placeholder="Username"/>
            <input type=password_type
                name="password"
                prop:value = move ||password.get()/>
            <a // when clicked, toggle the show password
                on:click={move |ev| {
                    ev.prevent_default(); // do this, or your page will reload and everything will break.
                    show_password.update(|show| *show = !*show)
                }}>
                {move || if show_password.get() {" Hide "} else {" Show "}}
            </a>
            <input type="submit" value=" Login "/> // the button that make it go

            // Since we have a resource, we need a suspense to show it off. Whenever the
            // login action has a result, it will trigger the status resource with an update, which
            // will show up as a change in the UI because of this suspense. If you don't use a suspense (or similar),
            // you'll get warnings in the browser console about using resources incorrectly.
            <Suspense fallback={move ||view!{<p>"checking stuff..."</p>}}>
                { // We're just going to tell people who they are. They may not know.
                    move || {
                        if let Some(Ok(Some(user))) = status() {
                            view!{<p>"Authenticated as user " {user.username}</p>}
                        } else {
                            view!{<p>"Not authenticated."</p>}
                        }
                    }
                }
            </Suspense>
        </ActionForm>
    }
}
