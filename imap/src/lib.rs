mod error;
pub use error::ImapError;

pub mod parser;
mod messages;

#[cfg(feature = "tokio-runtime")]
pub mod async_impl;
#[cfg(feature = "tokio-runtime")]
pub use async_impl::{Builder, connect_tls, connect_starttls, connect_plain};

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "blocking")]
pub use blocking::{Builder, connect_tls, connect_starttls, connect_plain};