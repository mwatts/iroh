use anyhow::Result;
use async_trait::async_trait;
use iroh_mount::Drive;
// use iroh_resolver::unixfs_builder::AddEvent;
// use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
#[cfg(feature = "testing")]
use mockall::automock;
use std::path::PathBuf;

use crate::Api;


#[cfg_attr(feature = "testing", automock)]
#[async_trait(?Send)]
pub trait Fs: Api {
    // async fn cat(&self, file: PathBuf) -> Result<OutType> {
      // // if ipfs_path.cid().is_none() {
      // //   return Err(anyhow!("IPFS path does not refer to a CID"));
      // // }
      // // let root_path = get_root_path(ipfs_path, output_path);
      // // if root_path.exists() {
      // //     return Err(anyhow!(
      // //         "output path {} already exists",
      // //         root_path.display()
      // //     ));
      // // }

      // let blocks = self.get_stream(ipfs_path);
      // tokio::pin!(blocks);
      // while let Some(block) = blocks.next().await {
      //     let (path, out) = block?;
      //     let full_path = path.to_path(root_path);
      //     match out {
      //         OutType::Dir => {
      //             tokio::fs::create_dir_all(full_path).await?;
      //         }
      //         OutType::Reader(mut reader) => {
      //             if let Some(parent) = path.parent() {
      //                 tokio::fs::create_dir_all(parent.to_path(root_path)).await?;
      //             }
      //             let mut f = tokio::fs::File::create(full_path).await?;
      //             tokio::io::copy(&mut reader, &mut f).await?;
      //         }
      //         OutType::Symlink(target) => {
      //             if let Some(parent) = path.parent() {
      //                 tokio::fs::create_dir_all(parent.to_path(root_path)).await?;
      //             }
      //             #[cfg(windows)]
      //             tokio::task::spawn_blocking(move || {
      //                 make_windows_symlink(target, full_path).map_err(|e| anyhow::anyhow!(e))
      //             })
      //             .await??;
    
      //             #[cfg(unix)]
      //             tokio::fs::symlink(target, full_path).await?;
      //         }
      //     }
      // }
      // Ok(())
    // }

    // async fn cp(&self, from: PathBuf, to: PathBuf) -> Result<AddEvent>;
    async fn ls(&self, _dir: &PathBuf) -> Result<Vec<Drive>> {
      // if dir.iter(). == 0 {
        self.list_mounts().await
      // } else {
      //   Err(anyhow!("cannot list {}", dir.display()))
      // }
    }
    // async fn mkdir(&self, target: PathBuf) -> Result<AddEvent>;
    // TODO(b5):
    // async fn mv(&self, from: PathBuf, to: PathBuf) -> Result<()>;
    // TODO(b5):
    // async fn rm(&self, file: PathBuf, recursive: bool) -> Result<()>;
    // async fn tree(&self, dir: PathBuf) -> Result<()>;
    // TODO(b5):
    // async fn write(&self, dest: PathBuf) -> Result<()>;
}

impl<T> Fs for T where T: Api {}

// #[async_trait]
// impl Fs for ClientFs {
//   async fn cat(&self, file: PathBuf) -> Result<OutType> {

//   }
// }