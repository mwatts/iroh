use anyhow::Result;
use chrono::Utc;
use cid::Cid;
use iroh_util::iroh_data_path;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs, path::PathBuf};
use wnfs::{dagcbor, BlockStore, PublicDirectory};

use crate::blockstore::IrohBlockStore;

pub fn default_volume() -> Result<Volume> {
    Volume::open(iroh_default_volume_path()?)
}

fn iroh_default_volume_path() -> Result<PathBuf> {
    iroh_data_path("wnfs.json")
}

#[derive(Serialize, Deserialize)]
pub struct Volume {
    path: PathBuf,
    pub root: Option<Cid>,
    // RootKey         *wnfs.Key
    // PrivateRootName *wnfs.PrivateName
}

impl Volume {
    pub fn open(path: PathBuf) -> Result<Self> {
        match fs::File::open(path.clone()) {
            Ok(rdr) => {
                let v: Volume = serde_json::from_reader(rdr)?;
                Ok(v)
            }
            Err(_) => Ok(Volume { path, root: None }),
        }
    }

    pub fn write(&self) -> Result<()> {
        let contents = serde_json::to_string(self)?;
        fs::write(self.path.clone(), contents).map_err(|e| e.into())
    }

    pub async fn root(&self, store: &IrohBlockStore<'_>) -> Result<PublicDirectory> {
        if self.root.is_none() {
            return Ok(PublicDirectory::new(Utc::now()));
        }

        let root_cid = &self.root.unwrap();
        let data = store.get_block(root_cid).await?;
        dagcbor::decode::<PublicDirectory>(&data)
    }
}
