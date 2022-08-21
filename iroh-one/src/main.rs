use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Result};
use clap::Parser;
use iroh_gateway::{bad_bits::BadBits, cli::Args, core::Core, metrics};
use iroh_metrics::gateway::Metrics;
use iroh_one::{
    config::{Config, CONFIG_FILE_NAME, ENV_PREFIX},
    // core::Core,
    core,
};
use iroh_rpc_types::Addr;
use iroh_util::{iroh_home_path, make_config};
use prometheus_client::registry::Registry;
use serde::Deserialize;
use tokio::sync::RwLock;

fn default_wt() -> usize {
    8 // 64
}

fn default_max_t() -> usize {
    512 // 1024*8
}

fn default_tss() -> usize {
    1024 * 1024 // 1024*1024
}

fn default_gqi() -> u32 {
    31 // 20
}

fn default_ei() -> u32 {
    255 // 255
}

#[derive(Deserialize, Debug)]
struct TkCfg {
    #[serde(default = "default_wt")]
    pub worker_threads: usize,
    #[serde(default = "default_max_t")]
    pub max_blocking_threads: usize,
    #[serde(default = "default_tss")]
    pub thead_stack_size: usize,
    #[serde(default = "default_gqi")]
    pub global_queue_interval: u32,
    #[serde(default = "default_ei")]
    pub event_interval: u32,
}

impl Default for TkCfg {
    fn default() -> Self {
        TkCfg {
            worker_threads: default_wt(),
            max_blocking_threads: default_max_t(),
            thead_stack_size: default_tss(),
            global_queue_interval: default_gqi(),
            event_interval: default_ei(),
        }
    }
}

// #[tokio::main(flavor = "multi_thread")]
fn main() -> Result<()> {
    let tkcfg = match envy::prefixed("FOO_").from_env::<TkCfg>() {
        Ok(config) => config,
        Err(err) => {
            println!("error parsing config from env: {}", err);
            TkCfg::default()
        }
    };

    println!("{:#?}", tkcfg);

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(tkcfg.worker_threads)
        .max_blocking_threads(tkcfg.max_blocking_threads)
        .thread_stack_size(tkcfg.thead_stack_size)
        .global_queue_interval(tkcfg.global_queue_interval)
        .event_interval(tkcfg.event_interval)
        .enable_all()
        .build()
        .unwrap()
        .block_on(something())
        .unwrap();
    Ok(())
}

async fn something() -> Result<()> {
    let args = Args::parse();

    let sources = vec![iroh_home_path(CONFIG_FILE_NAME), args.cfg.clone()];
    let mut config = make_config(
        // default
        Config::default(),
        // potential config files
        sources,
        // env var prefix for this config
        ENV_PREFIX,
        // map of present command line arguments
        args.make_overrides_map(),
    )
    .unwrap();

    // println!("PORT: {}", args.port.unwrap_or(9050));
    config.gateway.port = args.port.unwrap_or(9050);
    let p2p_port = 4444 + config.gateway.port as u16 - 9050;
    config.p2p.libp2p.listening_multiaddr = format!("/ip4/0.0.0.0/tcp/{}", p2p_port).parse().unwrap();

    config.gateway.raw_gateway = "dweb.link".to_string();
    config.store.path = PathBuf::from(format!("./iroh-store-db-{}", config.gateway.port));

    let (store_rpc, p2p_rpc) = {
        let (store_recv, store_sender) = Addr::new_mem();
        config.rpc_client.store_addr = Some(store_sender);
        let store_rpc = iroh_one::mem_store::start(store_recv, config.clone().store.into()).await?;

        let (p2p_recv, p2p_sender) = Addr::new_mem();
        config.rpc_client.p2p_addr = Some(p2p_sender);
        let p2p_rpc = iroh_one::mem_p2p::start(p2p_recv, config.clone().p2p.into()).await?;
        (store_rpc, p2p_rpc)
    };

    config.rpc_client.raw_gateway = Some(config.gateway.raw_gateway.clone());
    config.gateway.rpc_client = config.rpc_client.clone();
    config.p2p.rpc_client = config.rpc_client.clone();
    config.store.rpc_client = config.rpc_client.clone();

    config.metrics = metrics::metrics_config_with_compile_time_info(config.metrics);
    println!("{:#?}", config);

    let metrics_config = config.metrics.clone();
    let mut prom_registry = Registry::default();
    let gw_metrics = Metrics::new(&mut prom_registry);
    let rpc_addr = config
        .server_rpc_addr()?
        .ok_or_else(|| anyhow!("missing gateway rpc addr"))?;

    let bad_bits = match config.gateway.denylist {
        true => Arc::new(Some(RwLock::new(BadBits::new()))),
        false => Arc::new(None),
    };

    let mut core_tasks = Vec::<tokio::task::JoinHandle<()>>::new();

    // for i in 0..args.num_threads.unwrap_or(1) {
        let mut rcfg = config.clone();
        rcfg.gateway.port = rcfg.gateway.port + i;

        let shared_state = Core::make_state(
            Arc::new(rcfg.clone()),
            gw_metrics.clone(),
            &mut prom_registry,
            Arc::clone(&bad_bits),
        )
        .await?;

        let rpc_addr = rcfg
            .clone()
            .server_rpc_addr()?
            .ok_or_else(|| anyhow!("missing gateway rpc addr"))?;
        let handler = Core::new_with_state(rpc_addr, Arc::clone(&shared_state)).await?;

        let server = handler.server();
        println!("HTTP endpoint listening on {}", server.local_addr());
        let core_task = tokio::spawn(async move {
            server.await.unwrap();
        });
        core_tasks.push(core_task);
    // }

    // let uds_server_task = {
    //     let uds_server = core::uds_server(shared_state);
    //     tokio::spawn(async move {
    //         uds_server.await.unwrap();
    //     })
    // };

    let metrics_handle =
        iroh_metrics::MetricsHandle::from_registry_with_tracer(metrics_config, prom_registry)
            .await
            .expect("failed to initialize metrics");

    iroh_util::block_until_sigint().await;

    store_rpc.abort();
    p2p_rpc.abort();

    for ct in core_tasks {
        ct.abort();
    }
    // uds_server_task.abort();
    // core_task.abort();

    metrics_handle.shutdown();

    Ok(())
}
