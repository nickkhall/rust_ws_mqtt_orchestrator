pub mod accept;

use tokio::net::TcpListener;

pub async fn start_listener(addr: &str) {
    // bind tcp listener
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");
    
    println!("Websocket Server listening on {}", addr);

    // accept incoming connections
    loop {
        let (stream, client_addr) = listener.accept().await.expect("Failed to accept a new connection");
        println!("Connected to client: {:?}", client_addr);

        // spawn new async task for new client
        tokio::spawn(async move {
            accept::accept_connection(stream).await;
        });
    }
}

