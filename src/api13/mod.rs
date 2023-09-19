use std::sync::{Arc, Mutex};

use crate::cache::{CacheMap, CacheItem};

use self::responses::*;

pub mod api;
mod responses;

pub struct Api13State {
	pub steam_api_key: String,
	pub author_cache: Arc<Mutex<CacheMap<u64, AuthorInfo>>>,
    pub mod_cache: Arc<Mutex<CacheMap<String, ModInfo>>>,
	pub mod_list_cache: Arc<Mutex<CacheItem<Vec<ModListInfo>>>>
}

impl Api13State {
    pub fn init(steam_api_key: String) -> Api13State {
        Api13State {
			steam_api_key,
			author_cache: Arc::new(Mutex::new(CacheMap::new())),
            mod_cache: Arc::new(Mutex::new(CacheMap::new())),
			mod_list_cache: Arc::new(Mutex::new(CacheItem::new()))
        }
    }
}