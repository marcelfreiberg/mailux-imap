#[cfg(feature = "tokio-runtime")]
pub mod async_impl;
#[cfg(feature = "tokio-runtime")]
pub use async_impl::Builder;

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "blocking")]
pub use blocking::Builder;

use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub enum ConnectionType {
    Tls,
    StartTls,
    Plain,
}

pub struct ConnectedState;
pub struct AuthenticatedState;

static TAG_COUNTER: AtomicU32 = AtomicU32::new(1);

fn next_tag() -> String {
    let tag_num = TAG_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("A{:04}", tag_num)
}

pub fn reset_tag_counter() {
    TAG_COUNTER.store(1, Ordering::SeqCst);
}
