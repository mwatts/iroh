use cid::Cid;

// pub enum Mount {}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Drive {
  pub name: Vec<u8>,
  pub cid: Cid,
  pub key: Option<Vec<u8>>,
  pub private_name: Option<Vec<u8>>,
}