use crate::{Address, HashMap, TransactionOutput, TryInto};
use std::hash::{BuildHasher, Hasher};

struct U64Hasher {
    state: u64,
}

impl Hasher for U64Hasher {
    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        self.state = self.state ^ u64::from_le_bytes(bytes[4..12].try_into().unwrap());
    }

    #[inline(always)]
    fn write_usize(&mut self, _: usize) {}

    #[inline(always)]
    fn write_u32(&mut self, i: u32) {
        let i: u64 = i.into();
        self.state = self.state ^ i;
    }

    #[inline(always)]
    fn write_u8(&mut self, i: u8) {
        let i: u64 = i.into();
        self.state = self.state ^ i;
    }

    #[inline(always)]
    fn finish(&self) -> u64 {
        self.state
    }
}

struct BuildU64Hasher;

impl BuildHasher for BuildU64Hasher {
    type Hasher = U64Hasher;

    #[inline(always)]
    fn build_hasher(&self) -> U64Hasher {
        U64Hasher { state: 0 }
    }
}

#[derive(Debug)]
pub struct U64HashMap<K, V> {
    hashmap: HashMap<K, V, BuildU64Hasher>,
}

impl<K: std::cmp::Eq + std::hash::Hash, V> U64HashMap<K, V> {
    #[inline(always)]
    pub fn new() -> Self {
        U64HashMap {
            hashmap: HashMap::with_hasher(BuildU64Hasher),
        }
    }

    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        U64HashMap {
            hashmap: HashMap::with_capacity_and_hasher(capacity, BuildU64Hasher),
        }
    }

    #[inline(always)]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.hashmap.insert(key, value)
    }

    #[inline(always)]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.hashmap.remove(key)
    }

    #[inline(always)]
    pub fn get(&self, key: &K) -> Option<&V> {
        self.hashmap.get(key)
    }

    #[inline(always)]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.hashmap.get_mut(key)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.hashmap.len()
    }

    #[inline(always)]
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.hashmap.iter()
    }
}

impl<K, V> IntoIterator for U64HashMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::collections::hash_map::IntoIter<K, V>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.hashmap.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a U64HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = std::collections::hash_map::Iter<'a, K, V>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.hashmap.iter()
    }
}

pub type TransactionOutputHashMap<V> = U64HashMap<TransactionOutput, V>;

pub type AddressHashMap<V> = U64HashMap<Address, V>;
