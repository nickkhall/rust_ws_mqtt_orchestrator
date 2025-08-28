use rustls::{Certificate, PrivateKey, RootCertStore, ServerConfig, AllowAnyAuthenticatedClient};
use tokio_rustls::TlsAcceptor;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

/// Load the server TLS configuration
/// - Reads server certificate chain and private key
/// - Loads trusted CA for client certificate verification
/// - Returns a ServerConfig ready for TlsAcceptor
pub fn load_server_config() -> ServerConfig {
    // 1️⃣ Load server certificate chain
    let cert_file = &mut BufReader::new(
        File::open("certs/server_cert.pem").expect("Server certificate not found")
    );
    let cert_chain = rustls_pemfile::certs(cert_file)
        .expect("Failed to read server certificates")
        .into_iter()
        .map(Certificate)
        .collect::<Vec<_>>();

    // 2️⃣ Load server private key
    let key_file = &mut BufReader::new(
        File::open("certs/server_key.pem").expect("Server private key not found")
    );
    let keys = rustls_pemfile::pkcs8_private_keys(key_file)
        .expect("Failed to read private keys")
        .into_iter()
        .map(PrivateKey)
        .collect::<Vec<_>>();

    let private_key = keys.first().expect("No private key found").clone();

    // 3️⃣ Load trusted client CA for mTLS
    let mut client_root_store = RootCertStore::empty();
    let ca_file = &mut BufReader::new(
        File::open("certs/ca_cert.pem").expect("Client CA cert not found")
    );
    client_root_store
        .add_pem_file(ca_file)
        .expect("Failed to add client CA");

    // 4️⃣ Enforce client certificate authentication
    let client_auth = AllowAnyAuthenticatedClient::new(client_root_store);

    // 5️⃣ Build ServerConfig
    ServerConfig::builder()
        .with_safe_defaults() // recommended TLS versions & ciphers
        .with_client_cert_verifier(client_auth)
        .with_single_cert(cert_chain, private_key)
        .expect("Failed to build ServerConfig")
}

/// Create a TlsAcceptor ready to wrap TcpStreams
pub fn create_tls_acceptor() -> TlsAcceptor {
    let config = load_server_config();
    TlsAcceptor::from(Arc::new(config)) // Arc allows shared ownership across async tasks
}

/// Optional: verify client certificate(s) manually
/// You could add custom enterprise policies here
pub fn verify_client_cert(certs: &[Certificate]) -> bool {
    if certs.is_empty() {
        println!("No client certificates provided");
        return false;
    }

    let client_cert = &certs[0].0; // take first cert
    println!("Client certificate length: {}", client_cert.len());

    // TODO: parse X.509 fields for org, CN, expiration
    true // return true if valid
}

