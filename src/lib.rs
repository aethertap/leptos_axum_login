#![allow(unused)]

// Leptos beginners: Consider this file to be the true root of all of your code, even stuff that is only
// for the server. When I started using leptos, I got mixed up and tried to split out server
// code so that it was only `use`d from main.rs, and that was a giant mistake. Put everything here
// and protect it with `cfg_if`. The only place where you'll need to qualify it with the crate
// name is in `main.rs`.

pub mod app;
pub mod error_template;
pub mod auth;
pub mod user;
use cfg_if::cfg_if;
pub mod pages;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod fileserv;
        pub mod sqlite_backend;
    }
}


#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
