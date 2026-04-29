use axionvera_network_node::tls_utils::build_rustls_server_config;
use rcgen::{BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio_rustls::rustls::{Certificate as RCert, ClientConfig, RootCertStore};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::TlsConnector;

#[tokio::test]
async fn server_rejects_client_without_cert_when_mtls_required() {
    // Create CA
    let mut ca_params = CertificateParams::new(vec![]);
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_params.distinguished_name = DistinguishedName::new();
    ca_params
        .distinguished_name
        .push(DnType::CommonName, "Test CA");
    let ca_cert = rcgen::Certificate::from_params(ca_params).unwrap();

    // Server cert signed by CA
    let mut server_params = CertificateParams::new(vec!["localhost".to_string()]);
    server_params.distinguished_name = DistinguishedName::new();
    server_params
        .distinguished_name
        .push(DnType::CommonName, "localhost");
    server_params.is_ca = IsCa::NoCa;
    let server_cert = rcgen::Certificate::from_params(server_params).unwrap();
    let server_cert_pem = server_cert.serialize_pem_with_signer(&ca_cert).unwrap();
    let server_key_pem = server_cert.serialize_private_key_pem();
    let ca_pem = ca_cert.serialize_pem().unwrap();

    // Build rustls server config requiring client auth
    let rustls_cfg = build_rustls_server_config(
        server_cert_pem.as_bytes(),
        server_key_pem.as_bytes(),
        Some(ca_pem.as_bytes()),
        true,
    )
    .expect("build server config");

    let acceptor = TlsAcceptor::from(Arc::new(rustls_cfg));

    // Start listener
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Server task: accept and attempt TLS handshake
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        match acceptor.accept(stream).await {
            Ok(mut s) => {
                // If handshake unexpectedly succeeds, write marker
                let _ = s.write_all(b"OK").await;
            }
            Err(_) => {
                // Handshake failed; drop connection
            }
        }
    });

    // Client: build client config trusting CA, but provide no client cert
    let mut root_store = RootCertStore::empty();
    let ca_certs = rustls_pemfile::certs(&mut ca_pem.as_bytes()).unwrap();
    for cert in ca_certs {
        root_store.add(&RCert(cert)).unwrap();
    }

    let client_cfg = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(client_cfg));
    let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let domain = rustls::ServerName::try_from("localhost").unwrap();

    let res = connector.connect(domain, stream).await;

    assert!(
        res.is_err(),
        "Handshake should fail when client cert is missing"
    );

    let _ = server.await;
}
