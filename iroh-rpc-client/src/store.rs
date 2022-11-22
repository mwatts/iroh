use std::io::Cursor;

use anyhow::{Context, Result};
use bytes::Bytes;
use cid::Cid;
#[cfg(feature = "grpc")]
use futures::Stream;
#[cfg(feature = "grpc")]
use iroh_rpc_types::store::store_client::StoreClient as GrpcStoreClient;
use iroh_rpc_types::store::{
    GetLinksRequest, GetRequest, GetSizeRequest, HasRequest, PutManyRequest, PutRequest, Store,
    StoreClientAddr, StoreClientBackend,
};
use iroh_rpc_types::Addr;
#[cfg(feature = "grpc")]
use tonic::transport::Endpoint;
#[cfg(feature = "grpc")]
use tonic_health::proto::health_client::HealthClient;

#[cfg(feature = "grpc")]
use crate::status::{self, StatusRow};
use crate::ServiceStatus;

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
}

use iroh_rpc_types::qrpc;
use iroh_rpc_types::qrpc::store::*;

#[derive(Debug, Clone)]
pub struct StoreClient2 {
    client: quic_rpc::RpcClient<StoreService, crate::ChannelTypes>,
}

impl StoreClient2 {
    pub async fn new(addr: iroh_rpc_types::qrpc::addr::Addr<StoreService>) -> anyhow::Result<Self> {
        match addr {
            iroh_rpc_types::qrpc::addr::Addr::Qrpc(addr) => {
                todo!()
            }
            iroh_rpc_types::qrpc::addr::Addr::Mem(channel) => {
                let channel = quic_rpc::combined::Channel::new(Some(channel), None);
                Ok(Self {
                    client: quic_rpc::RpcClient::new(channel),
                })
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn version(&self) -> Result<String> {
        let res = self.client.rpc(VersionRequest).await?;
        Ok(res.version)
    }

    #[tracing::instrument(skip(self, blob))]
    pub async fn put(&self, cid: Cid, blob: Bytes, links: Vec<Cid>) -> Result<()> {
        self.client
            .rpc(qrpc::store::PutRequest { cid, blob, links })
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, blocks))]
    pub async fn put_many(&self, blocks: Vec<(Cid, Bytes, Vec<Cid>)>) -> Result<()> {
        let blocks = blocks
            .into_iter()
            .map(|(cid, blob, links)| qrpc::store::PutRequest { cid, blob, links })
            .collect();
        self.client
            .rpc(qrpc::store::PutManyRequest { blocks })
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(&self, cid: Cid) -> Result<Option<Bytes>> {
        let res = self.client.rpc(qrpc::store::GetRequest { cid }).await?;
        Ok(res.data)
    }

    #[tracing::instrument(skip(self))]
    pub async fn has(&self, cid: Cid) -> Result<bool> {
        let res = self.client.rpc(qrpc::store::HasRequest { cid }).await?;
        Ok(res.has)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_links(&self, cid: Cid) -> Result<Option<Vec<Cid>>> {
        let res = self
            .client
            .rpc(qrpc::store::GetLinksRequest { cid })
            .await?;
        Ok(res.links)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_size(&self, cid: Cid) -> Result<Option<u64>> {
        let res = self.client.rpc(qrpc::store::GetSizeRequest { cid }).await?;
        Ok(res.size)
    }

    #[tracing::instrument(skip(self))]
    pub async fn check(&self) -> StatusRow {
        let status: ServiceStatus = self
            .version()
            .await
            .map(|_| ServiceStatus::Serving)
            .unwrap_or_else(|e| ServiceStatus::Unknown);
        StatusRow {
            name: "store",
            number: 3,
            status,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn watch(&self) -> impl Stream<Item = StatusRow> {
        futures::stream::pending()
    }
}
