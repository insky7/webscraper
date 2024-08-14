use axum::{
    body::{to_bytes, Body, Bytes},
    extract::{Request, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use scc::Queue;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thirtyfour::{By, DesiredCapabilities, WebDriver};
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
    let body_bytes = to_bytes(req.into_body(), usize::MAX).await.unwrap();
    let url = {
        let payload = serde_json::from_slice::<UrlFinder>(&body_bytes).unwrap();
        payload.url
    };
    // pretty cool to just add elem name into the struct i thought ;)
    let searched_elem = {
        let payload = serde_json::from_slice::<UrlFinder>(&body_bytes).unwrap();
        payload.element_name
    };
    let caps = DesiredCapabilities::chrome();
    // i thought, shouldn't the url share the port of the axum server?? O_o, not sure if thats the way to go.
    let mut driver = WebDriver::new("http://localhost:9515", caps).await;
    // let url = state.lock().unwrap().clone();
    // I guess this part below could crash if URL is not supplied, I need a middleware to validate proper formatted requests, think that is a good approach.
    driver.as_mut().unwrap().get(url).await.unwrap();
    // Is a vec appropriate here??
    // create vec to store found elems
    let mut stringified_elem: Vec<u8> = Vec::new();
    /*
    THIS NEEDS BETTER ERROR HANDLING BEYOND THIS POINT, the request will literally crash if the element isnt found! My idea is to match the result of Ok(driver)
    and instead of unwrapping I can properly handle the option types returned in the iteration of elements
    */
    for elem in driver
        .unwrap()
        // https://www.youtube.com/watch?v=TCm9788Tb5g
        .find_all(By::ClassName(searched_elem))
        .await
        .iter()
    {
        elem.iter()
            .next()
            // oh god
            .expect("ELEMENT NOT FOUND")
            .text()
            .await
            // oh no what is he doing chat
            .expect("ELEMENT NOT FOUND")
            .as_bytes()
            // is cloning bad here?
            // ALERT!!!! UNNECESSARY CLONING!
            // UNNECESSARY CLONING!
            // UNNECESSARY CLONING!
            // UNNECESSARY CLONING!
            .clone_into(&mut stringified_elem);
    }
    // return vec of elems
    (stringified_elem).into_response()
}
