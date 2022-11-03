use std::io::Cursor;

use anyhow::{Context, Result};
use bytes::Bytes;
use cid::Cid;
#[cfg(feature = "grpc")]
use futures::Stream;
use iroh_mount::Drive;
#[cfg(feature = "grpc")]
use iroh_rpc_types::store::store_client::StoreClient as GrpcStoreClient;
use iroh_rpc_types::store::{
    GetLinksRequest, GetRequest, GetSizeRequest, HasRequest, PutManyRequest, PutRequest, Store,
    StoreClientAddr, StoreClientBackend, 
    ListMountsRequest, GetMountRequest, Mount
};
use iroh_rpc_types::Addr;
#[cfg(feature = "grpc")]
use tonic::transport::Endpoint;
#[cfg(feature = "grpc")]
use tonic_health::proto::health_client::HealthClient;

#[cfg(feature = "grpc")]
use crate::status::{self, StatusRow};

impl_client!(Store);

impl StoreClient {
    #[tracing::instrument(skip(self))]
    pub async fn version(&self) -> Result<String> {
        let res = self.backend.version(()).await?;
        Ok(res.version)
    }

    #[tracing::instrument(skip(self, blob))]
    pub async fn put(&self, cid: Cid, blob: Bytes, links: Vec<Cid>) -> Result<()> {
        let req = PutRequest {
            cid: cid.to_bytes(),
            blob,
            links: links.iter().map(|l| l.to_bytes()).collect(),
        };
        self.backend.put(req).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, blocks))]
    pub async fn put_many(&self, blocks: Vec<(Cid, Bytes, Vec<Cid>)>) -> Result<()> {
        let blocks = blocks
            .into_iter()
            .map(|(cid, blob, links)| PutRequest {
                cid: cid.to_bytes(),
                blob,
                links: links.iter().map(|l| l.to_bytes()).collect(),
            })
            .collect();
        self.backend.put_many(PutManyRequest { blocks }).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(&self, cid: Cid) -> Result<Option<Bytes>> {
        let req = GetRequest {
            cid: cid.to_bytes(),
        };
        let res = self.backend.get(req).await?;
        Ok(res.data)
    }

    #[tracing::instrument(skip(self))]
    pub async fn has(&self, cid: Cid) -> Result<bool> {
        let req = HasRequest {
            cid: cid.to_bytes(),
        };
        let res = self.backend.has(req).await?;
        Ok(res.has)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_links(&self, cid: Cid) -> Result<Option<Vec<Cid>>> {
        let req = GetLinksRequest {
            cid: cid.to_bytes(),
        };
        let links = self.backend.get_links(req).await?.links;
        if links.is_empty() {
            Ok(None)
        } else {
            let links: Result<Vec<Cid>> = links
                .iter()
                .map(|l| Cid::read_bytes(Cursor::new(l)).context(format!("invalid cid: {:?}", l)))
                .collect();
            Ok(Some(links?))
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_size(&self, cid: Cid) -> Result<Option<u64>> {
        let req = GetSizeRequest {
            cid: cid.to_bytes(),
        };
        let size = self.backend.get_size(req).await?.size;
        Ok(size)
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_mounts(&self) -> Result<Vec<Drive>> {
        let req = ListMountsRequest {
            limit: None
        };
        let mounts = self.backend.list_mounts(req).await?.mounts;
        let mounts: Vec<Drive> = mounts
            .iter()
            .map(|m| {
                Drive{
                    name: m.name.clone(),
                    cid: Cid::try_from(m.cid.clone()).unwrap(),
                    key: None, // TODO(b5)
                    private_name: None, // TODO(b5)
                }
            })
            .collect();
        Ok(mounts)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_mount(&self, name: Vec<u8>) -> Result<Option<Drive>> {
        let req = GetMountRequest { name };
        let mount = self.backend.get_mount(req).await?.mount;
        match mount {
            None => Ok(None),
            Some(mount) => {
                Ok(Some(Drive{
                    name: mount.name,
                    cid: Cid::try_from(mount.cid).unwrap(),
                    key: None, // TODO(b5)
                    private_name: None, // TODO(b5)
                }))
            }
        }
    }

    pub async fn put_mount(&self, mount: Drive) -> Result<()> {
        let req = Mount {
            name: mount.name,
            cid: mount.cid.to_bytes(),
            key: mount.key,
            private_name: mount.private_name,
        };
        self.backend.put_mount(req).await?;
        Ok(())
    }

}
