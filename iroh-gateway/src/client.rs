use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use axum::body::StreamBody;
use futures::StreamExt;
use hyper::Body;
use iroh_metrics::gateway::Metrics;
use iroh_resolver::resolver::CidOrDomain;
use iroh_resolver::resolver::Metadata;
use iroh_resolver::resolver::OutMetrics;
use iroh_resolver::resolver::OutPrettyReader;
use iroh_resolver::resolver::Resolver;
use iroh_resolver::resolver::Source;
use prometheus_client::registry::Registry;
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;
use tracing::info;
use tracing::warn;

use crate::core::GetParams;
use crate::response::ResponseFormat;

#[derive(Debug, Clone)]
pub struct Client {
    resolver: Arc<Resolver<iroh_rpc_client::Client>>,
}

pub const CHUNK_SIZE: usize = 1024;

pub type PrettyStreamBody = StreamBody<ReaderStream<OutPrettyReader<iroh_rpc_client::Client>>>;

impl Client {
    pub fn new(rpc_client: &iroh_rpc_client::Client, registry: &mut Registry) -> Self {
        Self {
            resolver: Arc::new(Resolver::new(rpc_client.clone(), registry)),
        }
    }

    #[tracing::instrument(skip(self, rpc_client, metrics))]
    pub async fn get_file(
        &self,
        path: iroh_resolver::resolver::Path,
        rpc_client: &iroh_rpc_client::Client,
        start_time: std::time::Instant,
        metrics: &Metrics,
    ) -> Result<(PrettyStreamBody, Metadata), String> {
        info!("get file {}", path);
        let res = self
            .resolver
            .clone()
            .resolve(path)
            .await
            .map_err(|e| e.to_string())?;
        metrics
            .ttf_block
            .set(start_time.elapsed().as_millis() as u64);
        let metadata = res.metadata().clone();
        if metadata.source == Source::Bitswap {
            metrics
                .hist_ttfb
                .observe(start_time.elapsed().as_millis() as f64);
        } else {
            metrics
                .hist_ttfb_cached
                .observe(start_time.elapsed().as_millis() as f64);
        }
        let reader = res.pretty(
            rpc_client.clone(),
            OutMetrics {
                metrics: metrics.clone(),
                start: start_time,
            },
        );
        let stream = ReaderStream::new(reader);
        let body = StreamBody::new(stream);

        Ok((body, metadata))
    }

    #[tracing::instrument(skip(self, rpc_client, metrics))]
    pub async fn get_file_recursive(
        self,
        path: iroh_resolver::resolver::Path,
        rpc_client: iroh_rpc_client::Client,
        start_time: std::time::Instant,
        metrics: Metrics,
    ) -> Result<axum::body::Body, String> {
        info!("get file {}", path);
        let (mut sender, body) = axum::body::Body::channel();

        tokio::spawn(async move {
            let res = self.resolver.resolve_recursive(path);
            tokio::pin!(res);

            while let Some(res) = res.next().await {
                match res {
                    Ok(res) => {
                        metrics
                            .ttf_block
                            .set(start_time.elapsed().as_millis() as u64);
                        let metadata = res.metadata().clone();
                        if metadata.source == Source::Bitswap {
                            metrics
                                .hist_ttfb
                                .observe(start_time.elapsed().as_millis() as f64);
                        } else {
                            metrics
                                .hist_ttfb_cached
                                .observe(start_time.elapsed().as_millis() as f64);
                        }
                        let mut reader = res.pretty(
                            rpc_client.clone(),
                            OutMetrics {
                                metrics: metrics.clone(),
                                start: start_time,
                            },
                        );
                        let mut bytes = Vec::new();
                        reader.read_to_end(&mut bytes).await.unwrap();
                        sender.send_data(bytes.into()).await.unwrap();
                    }
                    Err(e) => {
                        warn!("failed to load recursively: {:?}", e);
                        sender.abort();
                        break;
                    }
                }
            }
        });

        Ok(body)
    }

    #[tracing::instrument()]
    pub async fn get_file_simulated(
        &self,
        _path: &str,
        start_time: std::time::Instant,
    ) -> Result<Body, String> {
        let (mut sender, body) = Body::channel();

        tokio::spawn(async move {
            // let test_path = Path::new("test_big.txt");
            // let mut file = File::open(test_path).unwrap();
            // let mut buf = [0u8; CHUNK_SIZE];
            sender
                .send_data(axum::body::Bytes::from("hepek"))
                .await
                .unwrap();
        });

        Ok(body)
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub format: ResponseFormat,
    pub cid: CidOrDomain,
    pub resolved_path: iroh_resolver::resolver::Path,
    pub query_file_name: String,
    pub content_path: String,
    pub download: bool,
    pub query_params: GetParams,
}
