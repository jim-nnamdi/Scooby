use ink::storage::{traits::{Packed, Storable}, Mapping};
use scale::Encode;

#[cfg_attr(
  feature = "std", 
  derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct _FileMap<K, V: Packed + Default>{
  pub values: Mapping<K, V>,
  pub length: u32
}

impl <K: Encode, V: Packed + Default> _FileMap<K, V> {
  pub fn _get(&self, key:&K) -> V {
    self.values.get(key).unwrap_or_default()
  }
  pub fn _set<I, U>(&mut self, key:I, value:&U)
  where 
  I: scale::EncodeLike<K>,
  U: scale::EncodeLike<V> + Storable,
  {
    if self.values.insert(key, value).is_none(){
      self.length += 1
    }
  }
  pub fn _remove(&mut self, key:&K) {
    if self.values.take(key).is_some() {
      self.length -= 1
    }
  }
}