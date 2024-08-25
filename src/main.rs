#![allow(unused)]
//#![feature(duration_constructors)]

use axum::response::IntoResponse;
use cfg_if::cfg_if;
use leptos_axum::handle_server_fns_with_context;

cfg_if!{
    if #[cfg(feature="ssr")] {
        use axum_login::AuthSession;
        use tower_sessions::Session;
        use logging::log;
        use axum_macros::FromRef;
        use axum::{body::Body, extract::State};
        use sqlx::SqlitePool;
        use leptos_axum_auth::{app::App,sqlite_backend::SqliteBackend};
        use leptos::*;

        /// This holds stuff I need to pass through to my server-side handler functions. YOU
        /// HAVE TO DERIVE `FromRef` ON THIS!!!! If you get an error message about LeptosOptions
        /// and Handlers in your Router, it's because you need this derived. I spent a long time
        /// cursing before I remembered seeing something about that in one of the docs.
        #[derive(Clone,FromRef)]
        pub struct AppState {
            /// Leptos magic stuff. It's important but I left it as a black box
            pub options: LeptosOptions,
            /// A `sqlx::SqlitePool` to hold the stuff we need to talk to our database.
            pub pool: SqlitePool,
        }

        /// This function gets called on the way to all of the server functions we'll define. Its purpose
        /// is to fill up the context with useful things so that we can get to them more easily, without
        /// having to `await` things from `leptos_axum::extract`. NOTE: There are some examples out there
        /// on the wild web where this function doesn't return a value. I don't konw if the API for axum
        /// changed, or if those are just mistakes, but you definintely need to return a value here! If not,
        /// you'll find that your server functions kinda work, except that the payload is always empty and
        /// your browser console will complain about eof while deserializing stuff.
        async fn server_fn_handler(
            // AuthSession has a session within it, but you can still use the session extractor
            // directly to get access to the same session. This holds the `user` field, which will be
            // `Some(<userdata>)` if somebody is logged in, or `None` otherwise.
            auth_session: AuthSession<SqliteBackend>,
            // This isn't strictly necessary, but if you want to use tower-sessions without axum_login,
            // this is how you would pass the session objects into your server functions.
            session: Session,
            // This holds the connection pool and leptos options
            State(appstate):State<AppState>,
            request: http::Request<Body>) -> impl IntoResponse{
            //log!("server_fn_handler: req={request:#?}");
            //session.insert("foo","bar").await;
            //session.save().await;
            log!("************ server_fn_handler *******************");
            //log!("AuthSession: {auth_session:#?}");
            log!("Session id in server_fn_handler: {:?}",session.id());
            log!("************ end server_fn_handler *******************");
            // This is what does the actual server function magic. The closure gets called to provide the
            // things within as context, so that we don't have to use extractors. It seems like it's faster
            // this way, asserted with absolutely no testing other than noticing there's no call to `await`
            // required.
            handle_server_fns_with_context(move || {
                provide_context(auth_session.clone());
                provide_context(appstate.pool.clone());
                provide_context(session.clone());
            }, request).await
        }

        /// This function handles everything that's rendered by your leptos WASM. It doesn't serve up the
        /// static files, and it doesn't do server functions. As with the `server_fn_handler`, you pass
        /// arguments to it and axum will pull out matching data to feed in when it calls the function
        /// (see extractors in Axum docs). I'm pulling out the session just because I can. In this case,
        /// the session is the same as the one inside of AuthSession, but having it directly makes it easier
        /// to use.
        #[axum_macros::debug_handler]
        async fn leptos_routes_handler(auth_session: AuthSession<SqliteBackend>,
            session: Session,
            State(appstate): State<AppState>,
            req: http::Request<Body>) -> http::Response<Body>{
                let AppState{pool,options} = appstate;
                //session.insert("route_handler",178).await;
                //session.save().await;
                log!("************ leptos_routes_handler *******************");
                log!("Request: {req:#?}");
                //log!("AuthSession: {auth_session:#?}");
                //log!("Session: {session:#?}");
                log!("Session id in leptos_routes_handler: {:?}",session.id());
                log!("************ end leptos_routes_handler *******************");
                // This is where the content is rendered. Same as server functions, provide context by
                // calling `provide_context` with whatever it is in the closure.
                let handler = leptos_axum::render_app_to_stream_with_context(options.clone(),
                move || {
                    provide_context(auth_session.clone());
                    provide_context(pool.clone());
                    provide_context(session.clone());
                },
                || view! { <App/> }
            );
            handler(req).await
        }
    }
}


#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {

    use axum::Router;
    use axum::routing::{get,post};
    use leptos::*;
    use tower::ServiceBuilder;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use leptos_axum_auth::{app::*, sqlite_backend::SqliteBackend};
    use leptos_axum_auth::fileserv::file_and_error_handler;
    use axum::ServiceExt;
    use axum_login::{
        login_required,
        tower_sessions::{MemoryStore, SessionManagerLayer},
        AuthManagerLayerBuilder,
    };

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // I'm using a memory store for session data, which is fine for small projects. There are a bunch
    // of other stores already implemented, look at the https://github.com/maxcountryman/tower-sessions
    // page for a list. You can also roll your own without too much difficulty.
    let session_store = MemoryStore::default();
    // This gets passed to the axum router as a layer, and it handles distributing session ids. Note
    // that you *will not* get a session ID in the browser unless your session has at least one
    // item stored in it! That hung me up for a few hours, trying to figure out what I was doing wrong
    // when it was just the designed behavior. As soon as your put something in, the session will have an id.
    let session_layer = SessionManagerLayer::new(session_store)
        // expire in five days. Note that `tower-sessions` requires you to use the `time` crate,
        // not `chrono` and not `std::time`.
        .with_expiry(tower_sessions::Expiry::OnInactivity(time::Duration::days(5)));
    // This is the AuthSession backend we defined elsewhere in this example.
    let backend = SqliteBackend::new()
        .await.expect("failed to get auth backend");
    // The backend allocates the SqlitePool (probably shouldn't, but it does for now). It's `clone`-able so
    // we get a copy of it here to give to the rest of the server.
    let pool = backend.pool.clone();
    // sqlx can automatically update database schema with this function, so we do that. The database schema
    // is defined in the `db` directory of this project, and you can mess with it using the sqlx command line tool, which
    // gets installed via `cargo install sqlx-cli`
    backend.migrate()
        .await.expect("failed database migration");

    // This is where we get into the `axum_login` stuff. This makes a layer to add to the axum router
    // that will handle authentication/authorization for us.
    let auth_layer = ServiceBuilder::new()
        .layer(AuthManagerLayerBuilder::new(backend, session_layer).build());

    let app_state = AppState {
        options:leptos_options.clone(),
        pool
    };

    let app = Router::new()
        // All server functions will go through `server_fn_handler`
        .route("/api/*fn_name", post(server_fn_handler))
        // All wasm pages from leptos will go through `leptos_routes_handler`
        .leptos_routes_with_handler(routes, get(leptos_routes_handler) )
        // Everything else gets `file_and_error_handler`
        .fallback(file_and_error_handler)
        // This adds the auth stuff to the router so that it can mess with all of the requests
        .layer(auth_layer)
        // This makes the AppState we created up there accessible to the extractors in our handler functions
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// this is here so that we can have a main when we're not actually compiling a server
#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
