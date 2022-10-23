use anyhow::Result;
use async_trait::async_trait;
use iroh_rpc_types::one::{One as RpcOne, OneServerAddr, VersionResponse};

#[derive(Default)]
pub struct One {}

#[async_trait]
impl RpcOne for One {
    #[tracing::instrument(skip(self))]
    async fn version(&self, _: ()) -> Result<VersionResponse> {
        let version = env!("CARGO_PKG_VERSION").to_string();
        Ok(VersionResponse { version })
    }
}

#[cfg(feature = "grpc")]
impl iroh_rpc_types::NamedService for One {
    const NAME: &'static str = "one";
}

pub async fn new(addr: OneServerAddr, one: One) -> Result<()> {
    iroh_rpc_types::one::serve(addr, one).await
}
