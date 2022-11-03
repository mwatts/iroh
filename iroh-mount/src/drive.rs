use cid::Cid;

// pub enum Mount {}

// #[derive(PartialEq, Eq, Debug, Clone)]
// pub enum DriveFormat {
//   UnixFS,
//   Ipld,
//   WnfsPublic,
//   WnfsPrivate,
// }

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Drive {
  pub name: Vec<u8>,
  // pub format: DriveFormat,
  pub cid: Cid,
  pub key: Option<Vec<u8>>,
  pub private_name: Option<Vec<u8>>,
}