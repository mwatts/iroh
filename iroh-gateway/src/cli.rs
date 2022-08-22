/// CLI arguments support.
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub port: Option<u16>,
    #[clap(short, long)]
    pub writeable: Option<bool>,
    #[clap(short, long)]
    pub fetch: Option<bool>,
    #[clap(short, long)]
    pub cache: Option<bool>,
    #[clap(long)]
    pub metrics: bool,
    #[clap(long)]
    pub tracing: bool,
    #[clap(long)]
    pub cfg: Option<PathBuf>,
    #[clap(long)]
    pub denylist: bool,
    #[clap(long)]
    pub num_threads: Option<u16>,
}

impl Args {
    pub fn make_overrides_map(&self) -> HashMap<&str, String> {
        let mut map: HashMap<&str, String> = HashMap::new();
        if let Some(port) = self.port {
            map.insert("port", port.to_string());
        }
        if let Some(writable) = self.writeable {
            map.insert("writable", writable.to_string());
        }
        if let Some(fetch) = self.fetch {
            map.insert("fetch", fetch.to_string());
        }
        if let Some(cache) = self.cache {
            map.insert("cache", cache.to_string());
        }
        map.insert("denylist", self.denylist.to_string());
        map.insert("metrics.collect", self.metrics.to_string());
        map.insert("metrics.tracing", self.tracing.to_string());
        map
    }
}
