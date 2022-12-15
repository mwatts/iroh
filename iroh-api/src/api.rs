use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::config::{Config, CONFIG_FILE_NAME, ENV_PREFIX};
use crate::IpfsPath;
use crate::P2pApi;
use anyhow::{ensure, Context, Result};
use cid::Cid;
use futures::stream::LocalBoxStream;
use futures::{StreamExt, TryStreamExt};
use iroh_resolver::resolver::Resolver;
use iroh_rpc_client::{Client, ClientStatus};
use iroh_unixfs::{
    builder::Entry as UnixfsEntry,
    content_loader::{FullLoader, FullLoaderConfig},
    ResponseClip,
};
use iroh_util::{iroh_config_path, make_config};
#[cfg(feature = "testing")]
use mockall::automock;
use relative_path::RelativePathBuf;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::store::add_blocks_to_store;

#[derive(Debug, Clone)]
pub struct Api {
    client: Client,
    resolver: Resolver<FullLoader>,
}

pub enum OutType {
    Dir,
    Reader(Box<dyn AsyncRead + Unpin>),
    Symlink(PathBuf),
}

impl fmt::Debug for OutType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dir => write!(f, "Dir"),
            Self::Reader(_) => write!(f, "Reader(impl AsyncRead + Unpin>)"),
            Self::Symlink(arg0) => f.debug_tuple("Symlink").field(arg0).finish(),
        }
    }
}

#[cfg_attr(feature = "testing", allow(dead_code), automock)]
impl Api {
    // The lifetime is needed for mocking.
    #[allow(clippy::needless_lifetimes)]
    pub async fn new<'a>(
        config_path: Option<&'a Path>,
        overrides_map: HashMap<String, String>,
    ) -> Result<Self> {
        let cfg_path = iroh_config_path(CONFIG_FILE_NAME)?;
        let sources = [Some(cfg_path.as_path()), config_path];
        let config = make_config(
            // default
            Config::default(),
            // potential config files
            &sources,
            // env var prefix for this config
            ENV_PREFIX,
            // map of present command line arguments
            overrides_map,
        )
        .unwrap();
        Api::from_config(config).await
    }

    pub async fn from_config(config: Config) -> Result<Self> {
        let client = Client::new(config.rpc_client).await?;
        let content_loader = FullLoader::new(
            client.clone(),
            FullLoaderConfig {
                http_gateways: config
                    .http_resolvers
                    .iter()
                    .flatten()
                    .map(|u| u.parse())
                    .collect::<Result<_>>()
                    .context("invalid gateway url")?,
                indexer: config
                    .indexer_endpoint
                    .as_ref()
                    .map(|u| u.parse())
                    .transpose()
                    .context("invalid indexer endpoint")?,
            },
        )?;
        let resolver = Resolver::new(content_loader);

        Ok(Self { client, resolver })
    }

    pub fn from_client_and_resolver(client: Client, resolver: Resolver<FullLoader>) -> Self {
        Self { client, resolver }
    }

    pub async fn provide(&self, cid: Cid) -> Result<()> {
        self.client.try_p2p()?.start_providing(&cid).await
    }

    pub fn p2p(&self) -> Result<P2pApi> {
        let p2p_client = self.client.try_p2p()?;
        Ok(P2pApi::new(p2p_client))
    }

    /// High level get, equivalent of CLI `iroh get`.
    pub fn get(
        &self,
        ipfs_path: &IpfsPath,
    ) -> Result<LocalBoxStream<'static, Result<(RelativePathBuf, OutType)>>> {
        ensure!(
            ipfs_path.cid().is_some(),
            "IPFS path does not refer to a CID"
        );

        tracing::debug!("get {:?}", ipfs_path);
        let resolver = self.resolver.clone();
        let results = resolver.resolve_recursive_with_paths(ipfs_path.clone());
        let sub_path = ipfs_path.to_relative_string();

        let stream = async_stream::try_stream! {
            tokio::pin!(results);
            while let Some(res) = results.next().await {
                let (relative_ipfs_path, out) = res?;
                let relative_path = RelativePathBuf::from_path(&relative_ipfs_path.to_relative_string())?;
                // TODO(faassen) this focusing in on sub-paths should really be handled in the resolver:
                // * it can be tested there far more easily than here (where currently it isn't)
                // * it makes sense to have an API "what does this resolve to" in the resolver
                // * the resolver may have opportunities for optimization we don't have
                if !relative_path.starts_with(&sub_path) {
                    continue;
                }
                let relative_path = relative_path.strip_prefix(&sub_path).expect("should be a prefix").to_owned();
                if out.is_dir() {
                    yield (relative_path, OutType::Dir);
                } else if out.is_symlink() {
                    let mut reader = out.pretty(resolver.clone(), Default::default(), ResponseClip::NoClip)?;
                    let mut target = String::new();
                    reader.read_to_string(&mut target).await?;
                    let target = PathBuf::from(target);
                    yield (relative_path, OutType::Symlink(target));
                } else {
                    let reader = out.pretty(resolver.clone(), Default::default(), ResponseClip::NoClip)?;
                    yield (relative_path, OutType::Reader(Box::new(reader)));
                }
            }
        };

        Ok(stream.boxed_local())
    }

    pub async fn check(&self) -> ClientStatus {
        self.client.check().await
    }

    pub async fn watch(&self) -> LocalBoxStream<'static, iroh_rpc_client::ClientStatus> {
        self.client.clone().watch().await.boxed_local()
    }

    /// The `add_stream` method encodes the entry into a DAG and adds
    /// the resulting blocks to the store. It returns a stream of
    /// CIDs and the size of the _raw data_ associated with that block.
    /// If the block does not contain raw data (only link data), the
    /// size of the block will be 0.
    pub async fn add_stream(
        &self,
        entry: UnixfsEntry,
    ) -> Result<LocalBoxStream<'static, Result<(Cid, u64)>>> {
        let blocks = match entry {
            UnixfsEntry::File(f) => f.encode().await?.boxed_local(),
            UnixfsEntry::Directory(d) => d.encode(),
            UnixfsEntry::Symlink(s) => Box::pin(async_stream::try_stream! {
                yield s.encode()?
            }),
        };

        Ok(Box::pin(
            add_blocks_to_store(Some(self.client.clone()), blocks).await,
        ))
    }

    /// The `add` method encodes the entry into a DAG and adds the resulting
    /// blocks to the store.
    pub async fn add(&self, entry: UnixfsEntry) -> Result<Cid> {
        let add_events = self.add_stream(entry).await?;

        add_events
            .try_fold(None, |_acc, (cid, _)| async move { Ok(Some(cid)) })
            .await?
            .context("No cid found")
    }
}
