
cfg_if::cfg_if!{
    if #[cfg(feature="ssr")] {
        use axum::extract::FromRef;
        use leptos::prelude::*;
        use sqlx::SqlitePool;
        use crate::config::ServerConfig;
        
        /// This holds stuff I need to pass through to my server-side handler functions. YOU
        /// HAVE TO DERIVE `FromRef` ON THIS!!!! If you get an error message about LeptosOptions
        /// and Handlers in your Router, it's because you need this derived. I spent a long time
        /// cursing before I remembered seeing something about that in one of the docs.
        #[derive(Clone,Debug,FromRef)]
        pub struct AppState {
            pub pool: SqlitePool,
            pub leptos_options: LeptosOptions,
            pub server_config: ServerConfig,
        }
    }
}
