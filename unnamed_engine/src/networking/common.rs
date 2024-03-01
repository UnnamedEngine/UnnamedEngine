use std::{error::Error, net::SocketAddr, sync::Arc};

use quinn::{ClientConfig, Endpoint, ServerConfig};

pub async fn run_server(server_endpoint: Endpoint) {
  // Accept a single connection
  let incomming_connection = match server_endpoint.accept().await {
    Some(incomming_connection) => incomming_connection,
    None => {
      log::error!("Failed to accept incoming connection");
      return;
    }
  };
  let connection = incomming_connection.await.unwrap();
  log::info!("Connection accepted: addr={}", connection.remote_address());
}

pub async fn run_client(server_addr: SocketAddr, server_cert: Vec<u8>) {
  let endpoint = make_client_endpoint("0.0.0.0:0".parse().unwrap(), &[&server_cert]).unwrap();
  let connection = match endpoint.connect(server_addr, "localhost").unwrap().await {
    Ok(connection) => connection,
    Err(_) => {
      log::error!("Failed to connect to server");
      return;
    }
  };
  log::info!("Connected: addr={}", connection.remote_address());
}

pub fn make_client_endpoint(
  bind_addr: SocketAddr,
  server_certs: &[&[u8]],
) -> Result<Endpoint, Box<dyn Error>> {
  let client_cfg = configure_client(server_certs)?;
  let mut endpoint = Endpoint::client(bind_addr)?;
  endpoint.set_default_client_config(client_cfg);
  Ok(endpoint)
}

pub fn make_server_endpoint(bind_addr: SocketAddr) -> Result<(Endpoint, Vec<u8>), Box<dyn Error>> {
  let (server_config, server_cert) = configure_server()?;
  let endpoint = Endpoint::server(server_config, bind_addr)?;
  Ok((endpoint, server_cert))
}

fn configure_client(server_certs: &[&[u8]]) -> Result<ClientConfig, Box<dyn Error>> {
  let mut certs = rustls::RootCertStore::empty();
  for cert in server_certs {
    certs.add(&rustls::Certificate(cert.to_vec()))?;
  }

  let client_config = ClientConfig::with_root_certificates(certs);
  Ok(client_config)
}

fn configure_server() -> Result<(ServerConfig, Vec<u8>), Box<dyn Error>> {
  let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
  let cert_der = cert.serialize_der().unwrap();
  let priv_key = cert.serialize_private_key_der();
  let priv_key = rustls::PrivateKey(priv_key);
  let cert_chain = vec![rustls::Certificate(cert_der.clone())];

  let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;
  let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
  transport_config.max_concurrent_uni_streams(0_u8.into());

  Ok((server_config, cert_der))
}
