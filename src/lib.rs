// allow this because it only comes up with todo!(), which should be gone before production.
#![allow(dependency_on_unit_never_type_fallback)]
#![recursion_limit = "256"]
#![feature(try_blocks)]

pub mod app;
pub mod auth;
pub mod user;
pub mod error_template;
pub mod state;
pub mod config;
pub mod pages;
pub mod prelude;
pub mod server;

cfg_if::cfg_if! {
    if #[cfg(feature="ssr")] {
        pub mod fallback;
        pub mod sqlite_backend;
    }
}


/// For a CSR mode app, this function needs to have
/// `#[cfg_attr(feature="csr",wasm_bindgen::prelud::wasm_bindgen)]`
#[cfg_attr(feature="csr", wasm_bindgen::prelude::wasm_bindgen)]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

