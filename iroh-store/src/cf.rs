use bytecheck::CheckBytes;
use iroh_mount::Drive;
use cid::Cid;
use rkyv::{with::AsBox, Archive, Deserialize, Serialize};

/// Column family to store actual data.
/// - Maps id (u64) to bytes
pub const CF_BLOBS_V0: &str = "blobs-v0";
/// Column family to store mutable mount-point references
/// - indexed by id (u64)
pub const CF_MOUNTS_V0: &str = "mounts-v0";
/// Column family that stores metdata about a given blob.
/// - indexed by id (u64)
pub const CF_METADATA_V0: &str = "metadata-v0";
/// Column familty that stores the graph for a blob
/// - indexed by id (u64)
pub const CF_GRAPH_V0: &str = "graph-v0";
/// Column family that stores the mapping (multihash, code) to id.
///
/// By storing multihash first we can search for ids either by cid = (multihash, code) or by multihash.
pub const CF_ID_V0: &str = "id-v0";

// This wrapper type serializes the contained value out-of-line so that newer
// versions can be viewed as the older version.
#[derive(Debug, Archive, Deserialize, Serialize)]
#[repr(transparent)]
#[archive_attr(repr(transparent), derive(CheckBytes))]
pub struct Versioned<T>(#[with(AsBox)] pub T);

#[derive(Debug, Archive, Deserialize, Serialize)]
#[repr(C)]
#[archive_attr(repr(C), derive(CheckBytes))]
pub struct MetadataV0 {
    /// The codec of the original CID.
    pub codec: u64,
    pub multihash: Vec<u8>,
}

#[derive(Debug, Archive, Deserialize, Serialize)]
#[repr(C)]
#[archive_attr(repr(C), derive(CheckBytes))]
pub struct MountV0 {
    pub name: Vec<u8>,
    pub cid: Vec<u8>,
    pub key: Option<Vec<u8>>,
    pub private_name: Option<Vec<u8>>,
}

impl MountV0 {
    pub fn from(drive: Drive) -> Self {
        MountV0 {
            name: drive.name,
            cid: drive.cid.to_bytes(),
            key: drive.key,
            private_name: drive.private_name,
        }
    }
}

#[derive(Debug, Archive, Deserialize, Serialize)]
#[repr(C)]
#[archive_attr(repr(C), derive(CheckBytes))]
pub struct GraphV0 {
    pub children: Vec<u64>,
}
