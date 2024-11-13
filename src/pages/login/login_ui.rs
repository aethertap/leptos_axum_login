
use leptos::prelude::*;
use leptos::either::Either;
use leptos_router::hooks::{use_navigate, use_query_map};
use crate::auth::get_user;
use crate::server::LoginUser;


/// Render a styled login form adapted from the tailwindui.com simple login form. It provides
/// feedback about current login status (informs you if you are already logged in, and as whom).
/// Currently, no navigation happens after a successful login, it should take a query:next to 
/// indicate where the user wants to go.
#[component]
pub fn Login() -> impl IntoView {
    let qmap = use_query_map();
    // This will call auth::login_user
    let login:ServerAction<LoginUser> = ServerAction::new();
    let show_pass = RwSignal::new(false);
    // based on the state of show_pass, this provides the `type=` attribute for the password
    // input.
    let pass_type = move || show_pass.get().then_some("text").or(Some("password")).unwrap();
    // If the user is logged in, return it.
    let logged_in_user = Resource::new(
        move || login.version().get(), 
        move |_user|  async move {
            if let Ok(Some(user)) = get_user().await {
                Some(user)
            } else {
                None
            }
        });
    // Create HTML to display the user's login status below the form.
    let login_status = move || Suspend::new( async move {
        match logged_in_user.await {
            Some(user) => Either::Left(view! { <p>"Logged in as " {user.username}</p> }),
            None => Either::Right(view! { <p>"Not logged in"</p> }),
        }
    });

    // Only set up a redirect if we were given a continuation url ("c") in the query map. c stands
    // for continue.
    // This part does the redirect. Effect creates something that gets attached to the reactive
    // system for this function, so it does not need to be explicitly stored in a variable here.
    // This one will be poked whenever logged_in_user changes, and if it gets a valid user it will
    // naviate to whatever the continue parameter holds.
    Effect::new(move || {
        use urlencoding::decode;
        // Make sure that you're using qmap.get() *inside* some kind of reactive context (like this
        // Effect). If you just want the value without the signal to update, use get_untracked
        // instead. You'll see a message in the console on the browser if you mess this up.
        if let Some(next) = qmap.get().get("c") {
            // Have to unwrap two layers of option here, first to see if we got anything from
            // logged_in_user (as a signal), then to see if the thing we got was something or not.
            if let Some(Some(_)) = logged_in_user.get() {
                // use_navigate is how you change your location from within the code.
                let nav = use_navigate();
                // remember: the URL here is going to be encoded, so it has to be decoded before we
                // try to go there. Probably should have some kind of error message if this fails,
                // but then that would only happen if the user is messing with the address bar. Let
                // them learn the consequences of their actions.
                if let Ok(next) = decode(&next) {
                    nav(&next,Default::default());
                } 
                // you can make a default place to go after login as well, but I didn't want to
                // make it go home by default because then it would be going to the same place no
                // matter what. So, if you directly go to the login page without a 'c' in the query
                // map, you just stay on the login page. 
                // else {
                //    nav("/dashboard",Default::default());
                // }
            }
        }
    });

    view! {
        <leptos_meta::Title text="Log in"></leptos_meta::Title>
        <ActionForm action=login>
            <div class="flex min-h-full flex-col justify-center px-6 py-12 lg:px-8">
                <div class="sm:mx-auto sm:w-full sm:max-w-sm">
                    <h2 class="mt-10 text-center text-2xl font-bold leading-9 tracking-tight text-gray-900">
                        Sign in to your account
                    </h2>
                </div>

                <div class="mt-10 sm:mx-auto sm:w-full sm:max-w-sm space-y-4">
                    <div>
                        <label
                            for="username"
                            class="flex self-start block text-sm font-medium leading-6 text-gray-900"
                        >
                            Username
                        </label>
                        <div class="mt-2">
                            <input
                                id="username"
                                name="username"
                                type="text"
                                autocomplete="username"
                                required
                                class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
                            />
                        </div>
                    </div>

                    <div>
                        <div class="flex items-center justify-between">
                            <label
                                for="password"
                                class="block text-sm font-medium leading-6 text-gray-900"
                            >
                                Password
                            </label>
                            <div class="text-sm">
                                <a
                                    href="#"
                                    class="font-semibold text-indigo-600 hover:text-indigo-500"
                                >
                                    Forgot password?
                                </a>
                            </div>
                        </div>
                        <div class="mt-2 flex flex-row items-center">
                            <input
                                id="password"
                                name="password"
                                type=pass_type
                                autocomplete="current-password"
                                required
                                class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
                            />
                            <a on:click=move |ev| {
                                ev.prevent_default();
                                show_pass.update(|s| *s = !*s);
                            }>{move || if show_pass.get() { " hide " } else { " show " }}</a>
                        </div>
                    </div>

                    <div>
                        <input
                            type="submit"
                            class=r#"flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold
                               leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline 
                               focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"#
                            value="sign in"
                        />
                    </div>

                    <p class="mt-10 text-center text-sm text-gray-500">
                        not a member?
                        <a
                            href="/register"
                            class="font-semibold leading-6 text-indigo-600 hover:text-indigo-500"
                        >
                            register for an account
                        </a>
                    </p>
                </div>
            </div>
        </ActionForm>
        <Transition fallback=move || view! { <p>Checking login...</p> }>{login_status}</Transition>
    }
}

