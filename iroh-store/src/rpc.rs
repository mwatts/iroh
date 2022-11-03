use std::io::Cursor;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use bytes::BytesMut;
use cid::Cid;
use iroh_rpc_types::store::{
    GetLinksRequest, GetLinksResponse, GetRequest, GetResponse, GetSizeRequest, GetSizeResponse,
    HasRequest, HasResponse, PutManyRequest, PutRequest, Store as RpcStore, StoreServerAddr,
    VersionResponse,
    Mount as RpcMount, ListMountsRequest, ListMountsResponse,GetMountRequest, GetMountResponse
};
use tracing::info;

use crate::store::Store;

#[cfg(feature = "rpc-grpc")]
impl iroh_rpc_types::NamedService for Store {
    const NAME: &'static str = "store";
}

#[async_trait]
impl RpcStore for Store {
    #[tracing::instrument(skip(self))]
    async fn version(&self, _: ()) -> Result<VersionResponse> {
        let version = env!("CARGO_PKG_VERSION").to_string();
        Ok(VersionResponse { version })
    }

    #[tracing::instrument(skip(self, req))]
    async fn put(&self, req: PutRequest) -> Result<()> {
        let cid = cid_from_bytes(req.cid)?;
        let links = links_from_bytes(req.links)?;
        let res = self.put(cid, req.blob, links)?;

        info!("store rpc call: put cid {}", cid);
        Ok(res)
    }

    #[tracing::instrument(skip(self, req))]
    async fn put_many(&self, req: PutManyRequest) -> Result<()> {
        let req = req
            .blocks
            .into_iter()
            .map(|req| {
                let cid = cid_from_bytes(req.cid)?;
                let links = links_from_bytes(req.links)?;
                Ok((cid, req.blob, links))
            })
            .collect::<Result<Vec<_>>>()?;
        self.put_many(req)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, req: GetRequest) -> Result<GetResponse> {
        let cid = cid_from_bytes(req.cid)?;
        if let Some(res) = self.get(&cid)? {
            Ok(GetResponse {
                data: Some(BytesMut::from(&res[..]).freeze()),
            })
        } else {
            Ok(GetResponse { data: None })
        }
    }

    #[tracing::instrument(skip(self))]
    async fn has(&self, req: HasRequest) -> Result<HasResponse> {
        let cid = cid_from_bytes(req.cid)?;
        let has = self.has(&cid)?;

        Ok(HasResponse { has })
    }

    #[tracing::instrument(skip(self))]
    async fn get_links(&self, req: GetLinksRequest) -> Result<GetLinksResponse> {
        let cid = cid_from_bytes(req.cid)?;
        if let Some(res) = self.get_links(&cid)? {
            let links = res.into_iter().map(|cid| cid.to_bytes()).collect();
            Ok(GetLinksResponse { links })
        } else {
            Ok(GetLinksResponse { links: Vec::new() })
        }
    }

    #[tracing::instrument(skip(self))]
    async fn get_size(&self, req: GetSizeRequest) -> Result<GetSizeResponse> {
        let cid = cid_from_bytes(req.cid)?;
        if let Some(size) = self.get_size(&cid).await? {
            Ok(GetSizeResponse {
                size: Some(size as u64),
            })
        } else {
            Ok(GetSizeResponse { size: None })
        }
    }

    async fn list_mounts(&self, _req: ListMountsRequest) -> Result<ListMountsResponse> {
        let mounts = self.list_mounts()?
            .iter()
            .map(|d| {
                RpcMount{
                    name: d.name.clone(),
                    cid: d.cid.to_bytes(),
                    key: d.key.clone(),
                    private_name: d.private_name.clone(),
                }
            })
            .collect();
        Ok(ListMountsResponse{ mounts })
    }

    async fn get_mount(&self, req: GetMountRequest) -> Result<GetMountResponse> {
        let mount = self.get_mount(req.name.clone())
            .map_err(|_| anyhow!("mount {:?} not found", req.name))?;
        match mount {
            Some(mount) => {
                Ok(GetMountResponse{
                    mount: Some(RpcMount{
                        name: mount.name,
                        cid: mount.cid.to_bytes(),
                        key: mount.key,
                        private_name: mount.private_name,
                    })
                })
            },
            None => Ok(GetMountResponse{ mount: None })
        }
    }

    async fn put_mount(&self, req: RpcMount) -> Result<()> {
        let cid = Cid::try_from(req.cid)?;
        self.put_mount(iroh_mount::Drive { 
            name: req.name,
            cid,
            key: req.key,
            private_name: req.private_name,
        })
    }
}

#[tracing::instrument(skip(store))]
pub async fn new(addr: StoreServerAddr, store: Store) -> Result<()> {
    info!("rpc listening on: {}", addr);
    iroh_rpc_types::store::serve(addr, store).await
}

#[tracing::instrument]
fn cid_from_bytes(b: Vec<u8>) -> Result<Cid> {
    Cid::read_bytes(Cursor::new(b)).context("invalid cid")
}

#[tracing::instrument]
fn links_from_bytes(l: Vec<Vec<u8>>) -> Result<Vec<Cid>> {
    l.into_iter().map(cid_from_bytes).collect()
}
