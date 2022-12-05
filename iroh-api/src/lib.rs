mod api;
mod config;
mod error;
mod p2p;
mod store;

pub mod fs;

#[cfg(not(feature = "testing"))]
pub use crate::api::Api;
#[cfg(feature = "testing")]
pub use crate::api::MockApi as Api;
pub use crate::api::OutType;
pub use crate::error::ApiError;
#[cfg(feature = "testing")]
pub use crate::p2p::MockP2p as P2pApi;
#[cfg(not(feature = "testing"))]
pub use crate::p2p::P2p as P2pApi;
pub use crate::p2p::PeerIdOrAddr;
#[cfg(feature = "testing")]
pub use crate::store::MockStore as StoreApi;
#[cfg(not(feature = "testing"))]
pub use crate::store::Store as StoreApi;
pub use bytes::Bytes;
pub use cid::Cid;
pub use iroh_resolver::chunker::DEFAULT_CHUNKS_SIZE;
pub use iroh_resolver::resolver::Path as IpfsPath;
pub use iroh_resolver::unixfs_builder::{AddEvent, ChunkerConfig, Config as UnixfsConfig};
pub use iroh_rpc_client::{Lookup, ServiceStatus, StatusRow, StatusTable};
pub use libp2p::gossipsub::MessageId;
pub use libp2p::{Multiaddr, PeerId};
