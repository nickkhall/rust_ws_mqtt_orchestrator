pub mod listener;
pub mod handler;

pub async fn run_ws_server() {
    listener::start_listener("127.0.0.1:8000").await;
}
