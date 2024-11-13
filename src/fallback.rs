use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode, Uri},
    response::{Html, IntoResponse, Response as AxumResponse},
};
use leptos::{
    hydration::{AutoReload, HydrationScripts},
    prelude::*,
};
use leptos_meta::provide_meta_context;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use crate::state::AppState;


// In leptos 0.7, I make my own app shell. This function is where that happens.
pub async fn file_or_index_handler(
    uri: Uri,
    State(state): State<AppState>,
) -> AxumResponse {
    let root = state.leptos_options.site_root.clone();
    let res = get_static_file(uri.clone(), &root).await.unwrap();
    //log!("file_or_index_handler");
    if res.status() == StatusCode::OK {
        let r = res.into_response();
        //log!("result: {r:#?}");
        r
    } else {
        provide_meta_context();
        let r = Html(view! {
            <!DOCTYPE html> 
            <html lang="en">
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    <AutoReload options=state.leptos_options.clone()/>
                    <HydrationScripts options=state.leptos_options.clone()/>
                    <link rel="stylesheet" id="leptos" href="/pkg/leptos_axum_login.css"/>
                    <link rel="shortcut icon" type="image/ico" href="/favicon.ico"/>
                    <link rel="manifest" href="/leptos_axum_login.webmanifest"/>
                </head>
                <body class="overflow-x-hidden"></body>
            </html>
        }.to_html()).into_response();
        //log!("Generated result: {r:#?}");
        r
    }
}

static MANIFEST:&'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/public/leptos_axum_login.webmanifest"));

async fn get_static_file(
    uri: Uri,
    root: &str,
) -> Result<Response<Body>, (StatusCode, String)> {
    if uri.path().ends_with(".webmanifest") { // special case, send the web app manifest
        let resp = Response::builder()
            .status(200)
            .header("content-type", "application/manifest+json")
            .body(Body::from(MANIFEST));
        return Ok(resp.unwrap());
    }
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // This path is relative to the cargo root
    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.into_response()),
    }
}

