mod blockstore;
pub mod volume;

use anyhow::{Context, Result};
use async_recursion::async_recursion;
use chrono::Utc;
use cid::Cid;
use futures::StreamExt;
use iroh_api::{AddEvent, Api, ChunkerConfig, IpfsPath, OutType, DEFAULT_CHUNKS_SIZE};
use ptree::{print_tree, TreeBuilder};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::blockstore::IrohBlockStore;
use crate::volume::Volume;
use wnfs::{Metadata, PublicDirectory, PublicNode, PublicOpResult};

pub async fn mkdir(api: &Api, v: &mut Volume, path: PathBuf) -> Result<Cid> {
    let store = &mut IrohBlockStore::new(api);
    let root = v.root(store).await?;

    let PublicOpResult { root_dir, .. } = Rc::new(root)
        .mkdir(&segments(path), Utc::now(), store)
        .await
        .unwrap();

    // Store the the file tree in the memory blockstore.
    let cid = root_dir.store(store).await?;
    v.root = Some(cid);
    v.write()?;
    Ok(cid)
}

pub async fn cat(api: &Api, v: &Volume, path: PathBuf) -> Result<()> {
    let store = &mut IrohBlockStore::new(api);
    let root = v.root(store).await?;
    let PublicOpResult { result, .. } = Rc::new(root).read(&segments(path), store).await?;
    let root_path = &IpfsPath::from_cid(result);
    let blocks = api.get(root_path)?;

    tokio::pin!(blocks);
    while let Some(block) = blocks.next().await {
        let (_, out) = block?;
        match out {
            OutType::Dir => {
                anyhow::bail!("can't cat a dir");
            }
            OutType::Reader(mut reader) => {
                let mut stdout = tokio::io::stdout();
                tokio::io::copy(&mut reader, &mut stdout).await?;
            }
            OutType::Symlink(_target) => {
                anyhow::bail!("nope no symlink support yet");
            }
        }
    }
    Ok(())
}

pub async fn cp(api: &Api, v: &mut Volume, src: PathBuf, dst: PathBuf) -> Result<()> {
    let store = &mut IrohBlockStore::new(api);
    let root = v.root(store).await?;
    let root_cid = copy_files(api, store, root, &src.as_path(), &dst.as_path()).await?;
    v.root = Some(root_cid);
    v.write()?;
    Ok(())
}

async fn copy_files(
    api: &Api,
    store: &mut IrohBlockStore<'_>,
    root: PublicDirectory,
    src: &Path,
    dst: &Path,
) -> Result<Cid> {
    // let mut stack = vec![src];
    // let _dst_root = PathBuf::from(dst);
    // let _src_root = PathBuf::from(src).components().count();
    let chunker = ChunkerConfig::Fixed(DEFAULT_CHUNKS_SIZE);

    if src.is_file() {
        println!("copying {:?} to {:?}", src, dst);
        let content_cid = add_file(api, src, chunker).await?;
        let PublicOpResult { root_dir, .. } = Rc::new(root)
            .write(&segments(dst.to_path_buf()), content_cid, Utc::now(), store)
            .await?;
        let root_cid = root_dir.store(store).await?;
        return Ok(root_cid);
    }

    todo!("directory copying");

    // while let Some(working_path) = stack.pop() {
    //     println!("process: {:?}", &working_path);

    //     // Generate a relative path
    //     let src: PathBuf = working_path.components().skip(from_root).collect();

    //     // Create a destination if missing
    //     let dest = if src.components().count() == 0 {
    //         to_root.clone()
    //     } else {
    //         to_root.join(&src)
    //     };
    //     if fs::metadata(&dest).is_err() {
    //         fs::create_dir_all(&dest)?;
    //     }

    //     for entry in fs::read_dir(working_path)? {
    //         let entry = entry?;
    //         let path = entry.path();
    //         if path.is_dir() {
    //             stack.push(path);
    //         } else {
    //             match path.file_name() {
    //                 Some(filename) => {
    //                     let dest_path = dest.join(filename);
    //                     println!("  copy: {:?} -> {:?}", &path, &dest_path);
    //                     fs::copy(&path, &dest_path)?;
    //                 }
    //                 None => {
    //                     println!("failed: {:?}", path);
    //                 }
    //             }
    //         }
    //     }
    // }

    // Ok(())
}

async fn add_file(api: &Api, path: &Path, chunker: ChunkerConfig) -> Result<Cid> {
    if !path.exists() {
        anyhow::bail!("Path does not exist");
    }
    if !path.is_file() {
        anyhow::bail!("Path is not a file");
    }

    let mut progress = api.add_stream(path, false, chunker).await?;
    let mut cids = Vec::new();
    while let Some(add_event) = progress.next().await {
        match add_event? {
            AddEvent::ProgressDelta { cid, .. } => {
                cids.push(cid);
            }
        }
    }
    let root = *cids.last().context("File processing failed")?;
    Ok(root)
}

pub async fn ls(api: &Api, v: &Volume, path: PathBuf) -> Result<Vec<(String, Metadata)>> {
    let store = &mut IrohBlockStore::new(api);
    let root = v.root(store).await?;
    let PublicOpResult { result, .. } = Rc::new(root).ls(&segments(path), store).await?;
    Ok(result)
}

pub async fn tree(api: &Api, v: &Volume, path: PathBuf) -> Result<()> {
    let store = &IrohBlockStore::new(api);
    let root = v.root(store).await?;

    // Build a tree using a TreeBuilder
    let PublicOpResult { result, .. } = Rc::new(root)
        .get_node(&segments(path.clone()), store)
        .await?;
    let name = segments(path).last().unwrap().clone();
    let mut tree = TreeBuilder::new(name.clone());
    if let Some(node) = result {
        tree_visit(&mut tree, store, node, name, true).await?;
    }

    // Print out the tree using default formatting
    let built = tree.build();
    print_tree(&built)?;
    Ok(())
}

#[async_recursion(?Send)]
pub async fn tree_visit(
    tree: &mut TreeBuilder,
    store: &IrohBlockStore<'_>,
    node: PublicNode,
    name: String,
    is_root: bool,
) -> Result<()> {
    if node.is_dir() {
        if !is_root {
            tree.begin_child(name.clone());
        }
        let PublicOpResult { result, .. } = node.as_dir()?.ls(&vec![], store).await?;
        for (name, _) in result {
            let PublicOpResult { result, .. } =
                node.as_dir()?.get_node(&[name.clone()], store).await?;
            if let Some(ch) = result {
                tree_visit(tree, store, ch, name.clone(), false).await?;
            }
        }
        if !is_root {
            tree.end_child();
        }
    } else if node.is_file() {
        tree.add_empty_child(name);
    }
    Ok(())
}

pub async fn rm(_api: &Api, _v: &Volume, _path: PathBuf) -> Result<()> {
    todo!("finish me");
}

pub async fn log(_api: &Api, _v: &Volume, _path: PathBuf) -> Result<()> {
    todo!("finish me");
}

fn segments(path: PathBuf) -> Vec<String> {
    path.components()
        // TODO(b5) - terrible.
        .map(|s| s.as_os_str().to_string_lossy().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segments() {
        let p = Path::new("public/README.md");
        assert_eq!(segments(p.to_path_buf()), vec!["public", "README.md"]);
    }
}
