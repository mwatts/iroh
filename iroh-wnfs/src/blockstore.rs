use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use iroh_api::Api;
use iroh_resolver::parse_links;
use libipld::{cid::Version, Cid};
use multihash::{Code, MultihashDigest};
use std::borrow::Cow;
use wnfs::ipld::IpldCodec;
use wnfs::{BlockStore, FsError};

pub struct IrohBlockStore<'a> {
    api: &'a Api,
}

impl<'a> IrohBlockStore<'a> {
    /// Create a new lock for the given program. This does not yet acquire the lock.
    pub fn new(api: &'a Api) -> Self {
        IrohBlockStore { api }
    }
}

#[async_trait(?Send)]
impl BlockStore for IrohBlockStore<'_> {
    /// Stores an array of bytes in the block store.
    async fn put_block(&mut self, blob: Vec<u8>, codec: IpldCodec) -> Result<Cid> {
        let hash = Code::Sha2_256.digest(&blob);
        let cid = Cid::new(Version::V1, codec.into(), hash)?;
        let links = parse_links(&cid, &blob)?;
        let b = Bytes::from(blob);
        self.api.store()?.put_block(cid, b, links).await?;
        Ok(cid)
    }

    /// Retrieves an array of bytes from the block store with given CID.
    async fn get_block(&self, cid: &Cid) -> Result<Cow<'_, Vec<u8>>> {
        let blob = self
            .api
            .store()?
            .get_block(*cid)
            .await?
            .ok_or(FsError::CIDNotFoundInBlockstore)?;
        let blob: Vec<u8> = blob.iter().copied();
        Ok(Cow::Owned(blob))
    }
}
