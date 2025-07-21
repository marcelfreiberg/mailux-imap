mod error;
pub use error::ImapError;

#[cfg(feature = "tokio-runtime")]
pub mod async_impl;
#[cfg(feature = "tokio-runtime")]
pub use async_impl::Builder;

#[cfg(feature = "blocking")]
pub mod blocking;
