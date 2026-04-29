use std::io::Cursor;
use rustls_pemfile::{certs, read_one, Item};
use tokio_rustls::rustls::{Certificate, PrivateKey, RootCertStore, ServerConfig as RustlsServerConfig, AllowAnyAuthenticatedClient};

pub fn build_rustls_server_config(
    cert_pem: &[u8],
    key_pem: &[u8],
    client_ca_pem: Option<&[u8]>,
    require_client_auth: bool,
) -> Result<RustlsServerConfig, String> {
    // Parse server certs
    let mut cert_cursor = Cursor::new(cert_pem);
    let server_certs = certs(&mut cert_cursor).map_err(|_| "Failed to parse server cert PEM".to_string())?;
    if server_certs.is_empty() {
        return Err("No server certificates found in TLS cert data".to_string());
    }

    // Parse private key
    let mut key_cursor = Cursor::new(key_pem);
    let mut keys = Vec::new();
    loop {
        match read_one(&mut key_cursor).map_err(|_| "Failed to parse TLS private key PEM".to_string())? {
            Some(Item::PKCS8Key(key)) => { keys.push(key); }
            Some(Item::RSAKey(key)) => { keys.push(key); }
            Some(_) => {}
            None => break,
        }
    }
    if keys.is_empty() {
        return Err("No private keys found in TLS key data".to_string());
    }

    let der_certs = server_certs.into_iter().map(Certificate).collect::<Vec<_>>();
    let der_key = PrivateKey(keys.remove(0));

    let cfg = if let Some(ca_pem) = client_ca_pem {
        let mut roots = RootCertStore::empty();
        let mut cursor = Cursor::new(ca_pem);
        let parsed = certs(&mut cursor).map_err(|_| "Failed to parse client CA PEM".to_string())?;
        if parsed.is_empty() {
            return Err("No CA certificates found in client CA PEM".to_string());
        }
        for cert in parsed { roots.add(&Certificate(cert)).map_err(|e| format!("Failed to add CA cert: {}", e))?; }

        if require_client_auth {
            RustlsServerConfig::builder()
                .with_safe_defaults()
                .with_client_cert_verifier(AllowAnyAuthenticatedClient::new(roots))
                .with_single_cert(der_certs, der_key)
                .map_err(|e| format!("Failed to create rustls server config: {}", e))?
        } else {
            RustlsServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(der_certs, der_key)
                .map_err(|e| format!("Failed to create rustls server config: {}", e))?
        }
    } else {
        RustlsServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(der_certs, der_key)
            .map_err(|e| format!("Failed to create rustls server config: {}", e))?
    };

    Ok(cfg)
}
