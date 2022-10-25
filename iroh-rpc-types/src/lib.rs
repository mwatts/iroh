#![feature(fmt_internals, fmt_helpers_for_derive)]
#[macro_use]
mod macros;

pub mod gateway;
pub mod p2p;
mod store_proto;
// pub mod store_proxy;
// pub use store_proxy as store;
pub mod store_proxy_expanded;
pub use store_proxy_expanded as store;

// Reexport for convenience.
#[cfg(feature = "grpc")]
pub use tonic::transport::NamedService;

#[cfg(feature = "testing")]
pub mod test;

mod addr;
pub use crate::addr::Addr;
