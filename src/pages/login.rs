
use leptos::*;
use leptos_router::*;
use leptos_dom::logging::console_log;

use crate::auth::{Login,GetUser};


#[component]
pub fn Login() -> impl IntoView {
    use crate::auth::get_user;
    let username = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let show_password = create_rw_signal(false);
    let password_type = move || if show_password() {"text"} else {"password"};
    let login = create_server_action::<Login>();
    let status = create_resource(login.value(), |_| get_user());
    view! {
        <ActionForm action=login>
            <input type="text"
                name="username"
                on:input = move |e|{ username.set(event_target_value(&e))}
                prop:value=move ||username.get()
                placeholder="Username"/>
            <input type=password_type
                name="password"
                prop:value = move ||password.get()/>
            <input type="submit" value=" Login "/>
            <Suspense fallback={move ||view!{<p>"checking stuff..."</p>}}>
                {
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
