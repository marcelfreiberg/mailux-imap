#[cfg(feature = "tokio-runtime")]
pub mod async_impl;
#[cfg(feature = "tokio-runtime")]
pub use async_impl::Builder;

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "blocking")]
pub use blocking::Builder;

#[derive(Debug)]
pub enum ConnectionType {
    Tls,
    StartTls,
    Plain,
}

pub struct ConnectedState;

pub struct AuthenticatedState;