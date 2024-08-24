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
        /// and Handlers in your Router, it's because you need this derived.
        #[derive(Clone,FromRef)]
        pub struct AppState {
            pub options: LeptosOptions,
            pub pool: SqlitePool,
        }

        async fn server_fn_handler(auth_session: AuthSession<SqliteBackend>, session: Session, State(appstate):State<AppState>,request: http::Request<Body>) -> impl IntoResponse{
            //log!("server_fn_handler: req={request:#?}");
            //session.insert("foo","bar").await;
            //session.save().await;
            log!("************ server_fn_handler *******************");
            //log!("AuthSession: {auth_session:#?}");
            log!("Session id in server_fn_handler: {:?}",session.id());
            log!("************ end server_fn_handler *******************");
            handle_server_fns_with_context(move || {
                provide_context(auth_session.clone());
                provide_context(appstate.pool.clone());
                provide_context(session.clone());
            }, request).await
        }

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
    use core::time;

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

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        //.with_expiry(tower_sessions::Expiry::OnInactivity(time::Duration::days(5)));
        ;
    let backend = SqliteBackend::new()
        .await.expect("failed to get auth backend");
    let pool = backend.pool.clone();
    backend.migrate()
        .await.expect("failed database migration");
    let auth_layer = ServiceBuilder::new()
        .layer(AuthManagerLayerBuilder::new(backend, session_layer.clone()).build());

    let app_state = AppState {
        options:leptos_options.clone(),
        pool
    };

    let app = Router::new()
        .route("/api/*fn_name", post(server_fn_handler))
        .leptos_routes_with_handler(routes, get(leptos_routes_handler) )
        .fallback(file_and_error_handler)
        //.layer(session_layer)
        .layer(auth_layer)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
