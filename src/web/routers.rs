use axum::{
    body::{to_bytes, Body, Bytes},
    extract::{Request, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use scc::Queue;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thirtyfour::{By, ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};
#[derive(Deserialize, Serialize)]
// I made a struct to capture URL by user input, honestly probably not needed, maybe enum better?
pub struct UrlFinder {
    url: String,
    element_name: String,
}

// type Sender = tokio::sync::broadcast::Sender<String>;
// type Ingester = tokio::sync::broadcast::Sender<String>;

#[derive(Clone)]
pub struct AppState {
    pub db: Queue<String>,
}

// Handler to fetch the HTML content from a given URL and store it in the channel
pub async fn get_html_v2(
    // Json(payload): Json<UrlFinder>,
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let url = {
        let body_bytes = to_bytes(req.into_body(), usize::MAX).await.unwrap();
        let payload = serde_json::from_slice::<UrlFinder>(&body_bytes).unwrap();
        payload.url
    };
    match reqwest::get(url).await {
        Ok(response) => {
            let body = response.text().await.unwrap();
            state.db.push(body.clone());
            (StatusCode::OK, Bytes::from(body)).into_response()
        }
        Err(_) => (StatusCode::BAD_REQUEST, "Failed to fetch URL".to_string()).into_response(),
    }
}

// Handler to retrieve the last fetched HTML content from the channel
pub async fn use_html_v2(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.db.pop() {
        Some(entry) => (StatusCode::OK, Html((**entry).clone())).into_response(),
        None => (
            StatusCode::OK,
            Html("<h1>No content available</h1>".to_string()),
        )
            .into_response(),
    }
}

pub async fn scrape_stuff_v2(req: Request<Body>) -> impl IntoResponse {
    let body_bytes = match to_bytes(req.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::info!("No request received: {:?}", error);
            return (StatusCode::BAD_REQUEST, "Invalid request").into_response();
        }
    };

    // Deserialize the body bytes into UrlFinder
    let payload = match serde_json::from_slice::<UrlFinder>(&body_bytes) {
        Ok(data) => data,
        Err(error) => {
            tracing::info!("Failed to deserialize request body: {:?}", error);
            return (StatusCode::BAD_REQUEST, "Invalid JSON in request").into_response();
        }
    };

    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless().unwrap();
    // i thought, shouldn't the url share the port of the axum server?? O_o, not sure if thats the way to go.
    let driver: WebDriver = match WebDriver::new("http://localhost:9515", caps).await {
        Ok(driver) => match driver.goto(&payload.url).await {
            Ok(_d) => driver,
            Err(_e) => {
                tracing::info!(
                    "Web driver encountered an error navigating to the URL {:?}",
                    payload.url
                );
                return (StatusCode::BAD_REQUEST, "Invalid request").into_response();
            }
        },
        Err(_err) => {
            tracing::info!(
                "Web driver encountered an error, usually this means it did not start successfully"
            );
            return (StatusCode::BAD_REQUEST, "Invalid request").into_response();
        }
    };
    // Is a vec appropriate here??
    // create vec to store found elems
    let mut stringified_elem: Vec<u8> = Vec::new();

    match driver
        // https://www.youtube.com/watch?v=TCm9788Tb5g
        .find_all(By::ClassName(payload.element_name))
        .await
    {
        Ok(elem) => match elem.iter().next() {
            Some(val) => match val.text().await {
                // is cloning bad here? i feel like theres a way to push it into the vec but im too lazy
                Ok(t) => t.as_bytes().clone_into(&mut stringified_elem),
                Err(_) => todo!(),
            },
            None => {
                tracing::info!("Web driver encountered an error locating the given elements");
                return (StatusCode::BAD_REQUEST, "Invalid request").into_response();
            }
        },
        Err(_) => {
            tracing::info!("Web driver encountered an error locating the given elements");
            return (StatusCode::BAD_REQUEST, "Invalid request").into_response();
        }
    }
    // return vec of elems
    (stringified_elem).into_response()
}
