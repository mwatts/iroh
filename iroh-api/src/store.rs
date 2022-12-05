use anyhow::Result;
use bytes::Bytes;
use cid::Cid;
use iroh_rpc_client::StoreClient;
#[cfg(feature = "testing")]
use mockall::automock;

pub struct Store {
    client: StoreClient,
}

#[cfg_attr(feature = "testing", automock)]
#[cfg_attr(feature = "testing", allow(dead_code))]
impl Store {
    pub fn new(client: StoreClient) -> Self {
        Self { client }
    }

    pub async fn put_block(&self, cid: Cid, blob: Bytes, links: Vec<Cid>) -> Result<()> {
        self.client.put(cid, blob, links).await
    }

    pub async fn get_block(&self, cid: Cid) -> Result<Option<Bytes>> {
        self.client.get(cid).await
    }
}
