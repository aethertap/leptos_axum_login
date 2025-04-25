#![allow(unused_imports)]
use cfg_if::cfg_if;
cfg_if!{
    if #[cfg(feature="ssr")] {
        use leptos_axum_login::sqlite_backend::{self,*};
        use leptos_axum_login::server::{self,*};
        use leptos_axum_login::user::{self,*};
        
        pub type AuthSession = axum_login::AuthSession<SqliteBackend>;
    }
}


// The module handling is a bit confusing with this part of leptos. Here's what I've figured out: 
// - Put everything that's part of the app into the lib
// - Import it from the lib into main. Think of main as just launching the server and *nothing
// else*
// - Don't make any mod declarations in main. If the word `mod` shows up in this file, it's
// probably wrong!
 
cfg_if::cfg_if! {
    if #[cfg(feature="ssr")]{
        use tower_sessions::{
            Expiry, 
            ExpiredDeletion,
            SessionManagerLayer
        };
        use tower::ServiceBuilder;
        use axum::{
            body::Body as AxumBody,
            extract::{Path, State},
            http::Request,
            response::{Html, IntoResponse, Response},
            routing::{get, post},
            Router,
        };
        use axum_login::AuthManagerLayerBuilder;
        use tower_sessions_sqlx_store::SqliteStore;
        /// This is the default location of the configuration file that tells the server where to
        /// find the database file. 
        static CONFIG_PATH:&str = concat!(env!("CARGO_MANIFEST_DIR"), "/server_config.toml");

        use structopt::StructOpt;
        /// Command line args for the server. Well, arg. You can specify a different config file if
        /// you don't like the default path
        #[derive(StructOpt,Clone,Debug)]
        pub struct ServerOpts {
            /// Specify a different place to find the config file.
            #[structopt(short="c", long="conf", about="Specify the path to the configuration file", default_value=CONFIG_PATH)]
            config_path: String,
        }
        use leptos::prelude::*;
        use leptos_axum::{generate_route_list, LeptosRoutes,handle_server_fns_with_context};
        use leptos_axum_login::{
            fallback::file_or_index_handler, *,
            auth::*,
            state::AppState,
        };
    }
}

/// This function gets called on the way to all of the server functions we'll define. Its purpose
/// is to fill up the context with useful things so that we can get to them more easily, without
/// having to `await` things from `leptos_axum::extract`. NOTE: There are some examples out there
/// on the wild web where this function doesn't return a value. I don't konw if the API for axum
/// changed, or if those are just mistakes, but you definintely need to return a value here! If not,
/// you'll find that your server functions kinda work, except that the payload is always empty and
/// your browser console will complain about eof while deserializing stuff.
#[cfg(feature="ssr")]
async fn server_func_handler(
    auth_session: AuthSession,
    session: tower_sessions::Session,
    State(app_state):State<AppState>,
    req:Request<axum::body::Body>,
) -> impl IntoResponse {
    
    handle_server_fns_with_context(move || {
        // AuthSession has a session within it, but you can still use the session extractor
        // directly to get access to the same session. This holds the `user` field, which will be
        // `Some(<userdata>)` if somebody is logged in, or `None` otherwise.
        provide_context(auth_session.clone());
        // This isn't strictly necessary, but if you want to use tower-sessions without axum_login,
        // this is how you would pass the session objects into your server functions.
        provide_context(session.clone());
        // This holds the connection pool and leptos options
        provide_context(app_state.clone());
        // This is the data from the `server_config.toml` file
        provide_context(app_state.server_config.clone());
    }, req).await
}

cfg_if::cfg_if! {
    if #[cfg(feature="ssr")]  {

        use sqlx::sqlite::SqlitePoolOptions;
        use sqlx::SqlitePool;
        use crate::config::ServerConfig;
        use crate::error_template::AppError;

        /// This is where the connection pool to the database gets built. For sqlite, it's super
        /// simple (just pass in the path). For postgres, you'll need the user, database, db_host,
        /// and password fields that you'd use to connect from any other tool. In this case, the
        /// path for sqlite is coming from the ServerConfig struct, which is mostly here as an
        /// example of how to use one.
        pub async fn database_connect(config: &ServerConfig) -> Result<SqlitePool,AppError> {
            use std::env;
            let ServerConfig{database_file,..} = config;
            let db_url = format!("sqlite://{database_file}");
            env::set_var("DATABASE_URL",db_url.clone());

            Ok(SqlitePoolOptions::new()
                .max_connections(5)
                .connect(&db_url).await
                .map_err(|e| AppError::DatabaseError(format!("{e}")))?)
        }
    }
}

/// main is what does the launching of the server. It's only relevant on server-side stuff, and its
/// sole job is to get things configured and started up. All of the real code should live in the
/// lib.
/// 
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    let opts:ServerOpts = structopt::StructOpt::from_args();
    // I have a simple config file defined to make it easier to adapt this to a more realistic
    // situation. Add stuff to it by editing config.rs, and server_config.toml
    let server_config = toml::from_str::<config::ServerConfig>(
        &std::fs::read_to_string(&opts.config_path)
            .expect(&format!("Couldn't open config file at {}", CONFIG_PATH)),
    )
    .expect(&format!(
        "Couldn't parse configuration file from {}!",
        CONFIG_PATH
    ));

    // this one is part of leptos, it digs around in the mystical dark spaces and finds things that
    // only leptos knows.
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // For sqlite, this just opens the file. However, a similar app with postgres might actually
    // need to do some network stuff here, hence the async.
    let pool = database_connect(&server_config)
        .await
        .expect("Failed to connect to database");
  
    // Set up sessions tables. This is used by TowerSessions to keep track of its data, and the
    // table format is managed by it as well. You don't need to do anything but call the migrate
    // method after you get the session store, and it will make sure to set things up correctly.
    let session_store = SqliteStore::new(pool.clone())
        .with_table_name(server_config.session_table_name.clone())
        .expect("Failed to create session store");

    log!("Applying session store migration...");
    // build the session table and stuff if needed.
    session_store
        .migrate()
        .await
        .expect("Failed to apply database migration");
    log!("Migration complete");

    // This is a middleware layer in axum that handles putting the session cookie into the
    // responses when appropriate. It will only do that if there is actually data in the session,
    // so if you don't see a session when you think there should be one, it's probably because it's
    // empty. Add something to it and it might start Just Working.
    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(time::Duration::seconds(server_config.session_timeout_seconds)));

    // Delete old sessions every few seconds
    // See this link for details https://github.com/maxcountryman/tower-sessions-stores/tree/main/sqlx-store
    let _deletion_task = tokio::task::spawn(session_store.clone().continuously_delete_expired(
        tokio::time::Duration::from_secs(server_config.session_cleanup_interval_seconds as u64),
    ));

    // Finally, make the actual database backend that's going to be used by the auth layer to keep
    // track of login status. This is where you'll keep your usernames, password hashes, and other
    // account stuff.
    let backend = SqliteBackend::new(pool.clone());

    // This builds on the session layer to keep track of the authentication status of a user. When
    // a user is authenticated (which happens when you tell it to be so), that fact is recorded in
    // the session_store table and the cookie that goes to the browser now has a session_id (if it
    // didn't have data before, it does now).
    let auth_session_layer = ServiceBuilder::new()
        .layer(AuthManagerLayerBuilder::new(backend,session_layer).build());

    // This is some semi-global stuff that will be useful in many places on the server, so it gets
    // passed around as a use_context (explicity by me) and also with an axum extractor.
    let app_state = AppState {
        pool,
        leptos_options,
        server_config,
    };

    // Now we get to the part where leptos is going to take control. The Router here is part of
    // axum, and we're telling it to send all api calls to the server_func_handler we defined
    // before. That one will then give the request to leptos via `handle_server_fns_with_context`.
    let app = Router::new()
        .route("/api/{*fn_name}", post(server_func_handler))
        .fallback(file_or_index_handler)
        .layer(auth_session_layer)
        .with_state(app_state);

    // run our app with axum
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
