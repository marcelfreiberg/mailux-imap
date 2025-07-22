use crate::ImapError;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use std::sync::Arc;

pub fn create_tls_config() -> Arc<ClientConfig> {
    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };

    let mut config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    if cfg!(debug_assertions) {
        config.key_log = Arc::new(rustls::KeyLogFile::new());
    }

    Arc::new(config)
}

pub fn parse_server_name(addr: &str) -> Result<ServerName<'static>, ImapError> {
    let (host, _) = addr
        .rsplit_once(':')
        .ok_or_else(|| ImapError::InvalidAddressFormat(addr.into()))?;

    let server_name = ServerName::try_from(host.to_string())?;

    Ok(server_name)
}
