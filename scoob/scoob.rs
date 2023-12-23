use ink::storage::traits::Packed;
use ink::prelude::vec::Vec;
use scale::Encode;

use crate::filemap::filemap;

#[cfg_attr(
  feature="std", 
  derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct _Scoob <K, V: Packed + Default> {
  pub filedir: filemap::_FileMap<K, V>,
  pub fileowner: Vec<u8>,
}

impl <K:Encode,V:Packed + Default> _Scoob<K, V> {
  pub fn _one_file(&self, key:&K) {
     self.filedir.values.get(key).unwrap_or_default();
  }
}