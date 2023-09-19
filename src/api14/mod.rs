pub mod api;
mod responses;

use std::sync::{Mutex, Arc};

use crate::{cache::{CacheMap, CacheItem}, steamapi};

use self::responses::*;

pub struct Api14State {
	pub steam_api_key: String,
	pub author_cache: Arc<Mutex<CacheMap<u64, AuthorInfo>>>,
    pub mod_cache: Arc<Mutex<CacheMap<u64, steamapi::PublishedFileDetails>>>,
	pub mod_list_cache: Arc<Mutex<CacheItem<Vec<ModInfo>>>>
}

impl Api14State {
    pub fn init(steam_api_key: String) -> Api14State {
        Api14State { 
			steam_api_key,
			author_cache: Arc::new(Mutex::new(CacheMap::new())),
            mod_cache: Arc::new(Mutex::new(CacheMap::new())),
			mod_list_cache: Arc::new(Mutex::new(CacheItem::new()))
        }
    }
}