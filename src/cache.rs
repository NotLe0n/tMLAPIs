use std::collections::HashMap;
use std::hash::Hash;
use std::time::{self, Duration};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct CacheItem<T> {
    pub item: T,
    pub time_stamp: time::SystemTime
}

impl<T> CacheItem<T> {
    pub fn new() -> CacheItem<T> where T: Default {
        CacheItem {
            item: T::default(),
            time_stamp: time::SystemTime::UNIX_EPOCH
        }
    }

    pub fn expired(&self, seconds: u64) -> bool {
        self.time_stamp.elapsed().unwrap() < Duration::from_secs(seconds)
    }
}

pub struct CacheMap<K, C> {
    inner: HashMap<K, CacheItem<C>>
}

impl<K, C> CacheMap<K, C> where K: Eq+Hash {
    pub fn new() -> CacheMap<K, C> {
        CacheMap {
            inner: HashMap::new()
        }
    }
    
    pub fn get(&self, key: K, duration: u64) -> Option<&CacheItem<C>> {
        match self.inner.get(&key) {
            Some(cached_item) => match cached_item.expired(duration) {
                true => Some(cached_item),
                false => None
            }
            None => None
        }
    }

    pub fn insert(&mut self, key: K, value: CacheItem<C>) {
        self.inner.insert(key, value);
    }
}

pub fn lock_and_get<K, V>(cache: &Arc<Mutex<CacheMap<K, V>>>, key: K, time: u64) -> Option<V> where K:Eq+Hash, V:Clone {
    let locked_cache = cache.lock().unwrap();
    match locked_cache.get(key, time).cloned() {
        Some(i) => Some(i.item),
        None => None,
    }
}

pub fn lock_and_update<K, V>(cache: &Arc<Mutex<CacheMap<K, V>>>, key: K, value: V) -> V where K:Eq+Hash, V:Clone {
    let mut locked_cache = cache.lock().unwrap();
    locked_cache.insert(key, CacheItem {
        item: value.clone(),
        time_stamp: std::time::SystemTime::now(),
    });

    return value;
}