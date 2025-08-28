use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::tls

// Global rate limiter: Mutex protects HashMap, keys are Arc<String> for high-performance sharing
lazy_static! {
    static ref RATE_LIMITER: Mutex<HashMap<Arc<String>, Instant>> = Mutex::new(HashMap::new());
}

pub async fn accept_connection(stream: TcpStream) {
    // Get peer address
    let peer_addr = match stream.peer_addr() {
        Ok(addr) => Arc::new(addr.to_string()), 
        Err(_) => Arc::new("unknown".to_string()),
    };

    println!("Accepted connection from {}", peer_addr);

    // Rate limiting (1 connection per IP per second)
    {
        let mut limiter = RATE_LIMITER.lock().await; // async lock, mutex ensures safe concurrent access

        if let Some(last_time) = limiter.get(&peer_addr) {
            if last_time.elapsed() < Duration::from_secs(1) {
                println!("Connection from {} rejected: rate limit exceeded", peer_addr);
                return; // drop the stream early
            }
        }

        // Insert the key with current timestamp
        limiter.insert(Arc::clone(&peer_addr), Instant::now());
    }

    // Authentication token check
    if !authenticate_client(&stream).await {
        println!("Connection from {} rejected: invalid certificate", peer_addr);
        return; // drop the stream
    }

    // Initial handshake / protocol validation
    if !initial_handshake(&stream).await {
        println!("Connection from {} rejected: handshake failed", peer_addr);
        return; // drop the stream
    }

    // Delegate to handler: WebSocket upgrade & message processing
    handler::upgrade_to_websocket(stream).await;
}

// Authenticate with TLS certificate
async fn authenticate_client(_stream: &TcpStream) -> bool {
    let acceptor = tls::create_tls_acceptor();

    let tls_stream = match acceptor.accept(stream).await {
        Ok(s) => s,
        Err(_) => {
            println!("TLS handshake failed");
            return;
        }
    };

    if !tls::verify_client_cert(tls_stream.get_ref().1.peer_certificates().unwrap_or(&[])) {
        println!("Client certificate invalid");
        return;
    }
}

// Validate protocol headers, versions, etc.
async fn initial_handshake(_stream: &TcpStream) -> bool {
    true
}

fn load_server_config() -> ServerConfig {
    // ... load server cert, private key, trusted CA, return ServerConfig
}

fn verify_client_cert(certs: &[Certificate]) -> bool {
    // ... validate client certificate(s)
}

