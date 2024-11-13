use leptos::prelude::*;
use crate::auth::*;
use crate::server::RegisterNewUser;
use super::user_exists;


/// Compute a rough password strength measure for the given password. The result is proportional to
/// the square of the overall length, with extra bonuses added for numbers and symbols. The result
/// will be between 0 and 100, so it can be used as a percentage for an HTML element width. This is
/// not a smart password strength meter, but it's fun. Don't use it unless you do your own analysis
/// and decide it's good enough. It probably isn't.

pub fn password_strength(passwd: &str) -> f32 {
    let mut special:f32 = 0.5;
    let mut num:f32 = 0.5;
    let mut plain:f32 = 0.5;
    for ch in passwd.chars() {
        if "!@#$%^&*()_+-=<>?,./\\[]}{:;\"'".contains(ch) {
            special = 1.2;
        } else if "0123456789".contains(ch) {
            num = 1.1;
        } else {
            plain=1.3;
        }
    } 
    let len = passwd.len() as f32;
    let strength = plain*num*special*len*len/4.0;
    if strength > 100. {
        100.
    } else {
        strength
    }
}

/// Display a styled registration form with password confirmation and password strength meter. The
/// user can show the password as plaintext or hide it. If it's shown, the confirmation input is
/// disabled.
///
/// This will query the following server functions;
/// - `get_user` to check whether the user is already logged in
/// - `user_exists` to check whether a user name is already taken
/// - `register_new_user` (`RegisterNewUser`) to add the user to the database.
///
/// Currently, it doesn't do anything with errors.
///

#[component]
pub fn Register() -> impl IntoView {
    use leptos::either::Either;
    use leptos_router::hooks::use_navigate;

    // this will call auth::register_new_user
    let register:ServerAction<RegisterNewUser> = ServerAction::new();
    // These are just information holders for the edit box. Note that in leptos 0.7, the signals
    // are created with methods on the struct rather than the old `create_rw_signal`.
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    // If this is true, then the password is shown as text. Otherwise it's obscured as a
    // password-entry input.
    let show_pass = RwSignal::new(false);
    // This determins what kind of input is rendered later in the form, based on the state of the
    // `show_pass` RwSignal. Using the `with` method avoids cloning overhead.
    let pass_type = move || show_pass.with(|show| if *show { "text" } else { "password" });
    // This returns Some(user) if the user has an active, logged-in session
    let logged_in = Resource::new(move || register.version().get(), move |ver| async move {
        if let Some(user) = get_user().await.ok().and_then(|u| u) {
            if ver > 0 { // only redirect if a *new* registration happened
                let nav = use_navigate();
                nav("/", Default::default());
            }
            Some(user)          
        } else {
            None
        }
    });

    // Return true if the username provided is already taken, false if it is available
    let name_taken = Resource::new(username, move |name| async move {user_exists(name).await});

    // Show a piece of text to inform the user of whether or not their chosen username is already
    // taken. A Transition is used here instead of a Suspense so that it can switch back and forth
    // every time the name is edited. Suspend::new is a new way to turn async stuff into events in
    // the reactive system. 
    let available_ui = move || view! {
        <Transition fallback=|| {
            view! { "..." }
        }>
            {move || Suspend::new(async move {
                match name_taken.await {
                    Ok(true) => Either::Left(view! { " Sorry, that one's taken. " }),
                    _ => Either::Right(view! { " Available! " }),
                }
            })}
        </Transition>
    };
    // Inform the user if they are logged in already, and who they are (people forget these things
    // from time to time) This uses the Either component, which has a ton of relatives for
    // different numbers of options. If you have 7 things, for example, try the Either7 version.
    let login_status = move || Suspend::new(async move {
        match logged_in.await {
            Some(user) => Either::Left(view! { <p>"Logged in as " {user.username}</p> }),
            None => Either::Right(view! { <p>"Not logged in yet!"</p> })
        } 
    });

    // Dumb password strength calculation just to have a strength meter. It may be dumb but I still 
    // like it.
    let pass_strength = move || password.with(move |pw| password_strength(pw) );

    // This is adapted from the tailwindui.com simple registration form component.
    view! {
        <leptos_meta::Title text="Register"></leptos_meta::Title>
        // ActionForm is a form of action! It calls the server function you give it with the data
        // the user provides when the form is submitted
        <ActionForm action=register>
            <div class="flex min-h-full flex-col justify-center px-6 py-12 lg:px-8">
                <div class="sm:mx-auto sm:w-full sm:max-w-sm">
                    <h2 class="mt-10 text-center text-2xl font-bold leading-9 tracking-tight text-gray-900">
                        Register for a new account
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
                                on:input=move |ev| username.set(event_target_value(&ev))
                                class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
                            />
                        </div>
                        {available_ui}
                    </div>

                    <div>
                        <div class="flex items-center justify-between">
                            <label
                                for="password"
                                class="block text-sm font-medium leading-6 text-gray-900"
                            >
                                Password
                            </label>
                        </div>
                        <div class="mt-2 flex flex-row items-center">
                            <input
                                id="password"
                                name="password"
                                type=pass_type
                                required
                                on:input=move |ev| { password.set(event_target_value(&ev)) }

                                class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
                            />
                            // This is a plain anchor tag, but it has a click handler event attached to
                            // change the show_pass signal. Note that you need to prevent_default
                            // on this otherwise you'll end up scrolling to the top of the page or possibly
                            // jumping off to the end of the Internet.
                            <a
                                class="w-20 block font-bold cursor-pointer select-none"
                                on:click=move |ev| {
                                    ev.prevent_default();
                                    show_pass.update(|s| *s = !*s);
                                }
                            >
                                {move || if show_pass.get() { " Hide " } else { " Show " }}
                            </a>
                        </div> 
                        // This is the very cool strength meter, which is pretty advanced
                        // technology. Might be even more advanced if I did something with the
                        // color, but I don't want to overwhelm people with the awesomeness.
                        <div class="w-full h-2 flex items-start">
                            <div
                                class="rounded h-2 bg-green-300"
                                style=move || format!("width: {}%", pass_strength())
                            ></div>
                        </div>
                    </div>
                    // Ask for the password twice, but only if the show_password option is off. No
                    // reason to repeat it if you can just read it to see if it's right. right?
                    <div>
                        <div class="flex items-center justify-between">
                            <label
                                for="password2"
                                class="block text-sm font-medium leading-6 text-gray-900"
                            >
                                Confirm Password
                            </label>
                        </div>
                        <div class="mt-2 flex flex-row items-center">
                            <input
                                id="password2"
                                disabled=show_pass // hide the confirmation box if the password is human-readable
                                name="password2"
                                type=pass_type
                                required
                                class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
                            />
                        </div>
                    </div>

                    <div>
                        <input
                            type="submit"
                            class=r#"flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold
                               leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline 
                               focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"#
                            value="Register"
                        />
                    </div>
                </div>
            </div>
        </ActionForm>
        // Show the user what the state of their login is, if there's nowhere to redirect to after
        // this.
        <Transition fallback=|| view! { "Checking login status..." }>{login_status}</Transition>
    }
}
