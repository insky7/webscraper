use axum::{routing::get, Router};

async fn hello_world() -> &'static str {
    "Hello world!"
}
fn main() {
    fn init_router() -> Router {
        Router::new().route("/", get(hello_world))
    }
}
