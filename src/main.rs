mod web;
use axum::{
    routing::{get, post},
    Router,
};
use scc::Queue;

use std::{net::SocketAddr, sync::Arc};
// use web::routers::scrape_stuff;
use tracing_subscriber;
use web::routers::{get_html_v2, AppState};
use web::routers::{scrape_stuff_v2, use_html_v2};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let state = Arc::new(Mutex::new(None));
    // let state = Arc::new(AppState {
    //     channel: tokio::sync::RwLocktokio::sync::broadcast::channel(1).0,
    //     ingester: tokio::sync::broadcast::channel(1).0,
    // });
    let _stdout_subscriber = tracing_subscriber::fmt::init();

    tracing::info!("App began");
    let state = Arc::new(AppState {
        db: Queue::default(),
    });

    // let mut shutdown_signal = tokio::sync::broadcast::channel::<()>(1);

    // let cloned = state.clone();
    // tokio::spawn(async move {
    //     let mut ingesting = cloned.channel.subscribe();
    //     let mut shutdown_sig = shutdown_signal.0.subscribe();
    //     loop {
    //         tokio::select! {
    //             result = ingesting.recv() => {
    //                 if let Ok(v) = result {
    //                     _ = cloned.ingester.send(v);
    //                 }
    //             }
    //             _ = shutdown_sig.recv() => {
    //                 break;
    //             }
    //         }
    //     }
    // });

    let app = Router::new()
        // normal get / home route
        .route("/", get(|| async { "Welcome!" }))
        // post route the user can post urls to
        .route("/", post(get_html_v2))
        // route the user can get the response of a get reqwest for the last url they posted in post route
        .route("/help", get(use_html_v2))
        // .route("/scrape", post(get_html))
        .route("/work", post(scrape_stuff_v2))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    // axum::serve(listener, app).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    // .with_graceful_shutdown(async move {
    //     _ = shutdown_signal.1.recv().await;
    // })
    .await
    .unwrap();

    Ok(())
}
