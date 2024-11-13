
use leptos::prelude::*;
use leptos::either::Either;
use crate::auth::get_user;
use crate::server::LoginUser;


/// Render a styled login form adapted from the tailwindui.com simple login form. It provides
/// feedback about current login status (informs you if you are already logged in, and as whom).
/// Currently, no navigation happens after a successful login, it should take a query:next to 
/// indicate where the user wants to go.
#[component]
pub fn Login() -> impl IntoView {
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
    view! {
        <leptos_meta::Title text="Log in"></leptos_meta::Title>
        <ActionForm action=login>
            <div class="flex min-h-full flex-col justify-center px-6 py-12 lg:px-8">
                <div class="sm:mx-auto sm:w-full sm:max-w-sm">
                    // tailwindui.com/img/logos/mark.svg?color=indigo&shade=600" alt="your company"/>*/
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

