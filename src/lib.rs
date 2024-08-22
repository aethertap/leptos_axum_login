pub mod app;
pub mod error_template;
pub mod auth;
pub mod user;
use cfg_if::cfg_if;

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
