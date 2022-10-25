extern crate reqwest;

use rocket::serde::{Deserialize, DeserializeOwned, Serialize};
use rocket::serde::json::serde_json::{self, json, Value};
use rocket_cache_response::CacheResponse;
use crate::{APIError, cached_json, get_json, steamapi};
use crate::cache::{CacheItem, CacheMap};
use crate::steamapi::PublishedFileDetails;

#[get("/count")]
pub async fn count_1_4() -> Result<Value, APIError> {
    let url = format!("/IPublishedFileService/QueryFiles/v1/?key={}&appid={}&totalonly=true", steamapi::get_steam_key(), steamapi::APP_ID);
	let count = get_steam_api_json::<steamapi::CountResponse>(&url).await?;

	return Ok(json!(count.response));
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
enum ModSide {
	Both,
	Client,
	Server,
	NoSync
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
struct ModInfo {
	display_name: String,
	internal_name: String,
	mod_id: String,
	author: String,
	author_id: String,
	modside: String,
	homepage: String,
	tmodloader_version: String,
	version: String,
	mod_references: String,
	num_versions: u32,
	tags: Option<Vec<steamapi::ModTag>>,
	time_created: u64,
	time_updated: u64,
	workshop_icon_url: String,
	children: Option<Vec<steamapi::Child>>,

	downloads_total: u32,
	favorited: u32,
	followers: u32,
	views: u64,
	vote_data: Option<steamapi::VoteData>,
	playtime: String,
	num_comments: u32,
}

#[get("/author/<steamid>", rank=1)]
pub async fn author_1_4(steamid: u64) -> Result<CacheResponse<Value>, APIError> {
    get_author_info(steamid).await
}

#[get("/author/<steamname>", rank=2)]
pub async fn author_1_4_str(steamname: &str) -> Result<CacheResponse<Value>, APIError> {
	let steamid = steamapi::steamname_to_steamid(steamname).await?;
	get_author_info(steamid).await
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
struct AuthorInfo {
	mods: Vec<ModInfo>,
	total: u32,
	total_downloads: u64,
	total_favorites: u64,
	total_views: u64,
}

// global author cache variable
lazy_static! {
	static ref AUTHOR_CACHE: std::sync::RwLock<CacheMap<u64, AuthorInfo>> = std::sync::RwLock::new(CacheMap::new());
}

async fn get_author_info(steamid: u64) -> Result<CacheResponse<Value>, APIError> {
	let cache = {
		let mod_cache = AUTHOR_CACHE.read().unwrap();
		mod_cache.get(steamid, 3600).cloned()
	};

	let author = match cache {
		Some(cached_value) => cached_value.item,
		None => {
			let url = format!("/IPublishedFileService/GetUserFiles/v1/?key={}&appid={}&steamid={}&numperpage=100", steamapi::get_steam_key(), steamapi::APP_ID, steamapi::validate_steamid64(steamid)?);
			let author_data = get_steam_api_json::<steamapi::AuthorResponse>(&url).await
				.map_err(|_| APIError::InvalidSteamID(format!("Could not find an author with the id {}", steamid)))?;

			let mut mods: Vec<ModInfo> = Vec::new();
			let mut total_downloads: u64 = 0;
			let mut total_favorites: u64 = 0;
			let mut total_views: u64 = 0;

			// go through each mod
			for publishedfiledetail in author_data.response.publishedfiledetails {
				// increment total counts
				total_downloads += publishedfiledetail.subscriptions as u64;
				total_favorites += publishedfiledetail.favorited as u64;
				total_views += publishedfiledetail.views as u64;

				// filter mod data and add to Vec
				mods.push(
					get_filtered_mod_info(&publishedfiledetail)
				);
			}

			let author = AuthorInfo {
				mods,
				total: author_data.response.total,
				total_downloads,
				total_favorites,
				total_views,
			};

			// update cache value
			let mut cache = AUTHOR_CACHE.write().unwrap();
			cache.insert(steamid, CacheItem {
				item: author.clone(),
				time_stamp: std::time::SystemTime::now(),
			});

			author
		}
	};

	return cached_json!(author, 3600, false);
}

fn get_filtered_mod_info(publishedfiledetail: &steamapi::PublishedFileDetails) -> ModInfo {
	let publishedfiledetail = publishedfiledetail.clone();

	// tml specific data
	let kvtags = publishedfiledetail.kvtags.unwrap();
	let kvtags_iter = kvtags.iter();

	// get data from kvtags field
	let internal_name = find_kvtag_value(&kvtags_iter, "name").unwrap_or_default();
	let author = find_kvtag_value(&kvtags_iter, "Author").unwrap_or_default();
	let modside = find_kvtag_value(&kvtags_iter, "modside").unwrap_or_default();
	let homepage = find_kvtag_value(&kvtags_iter, "homepage").unwrap_or_default();
	let tmodloader_version = find_kvtag_value(&kvtags_iter, "modloaderversion").unwrap_or_default();
	let version = find_kvtag_value(&kvtags_iter, "version").unwrap_or_default();
	let mod_references = find_kvtag_value(&kvtags_iter, "modreferences").unwrap_or_default();

	// construct ModInfo struct
	return ModInfo{
		display_name: publishedfiledetail.title,
		internal_name,
		mod_id: publishedfiledetail.publishedfileid,
		author,
		author_id: publishedfiledetail.creator,
		modside,
		homepage,
		tmodloader_version,
		version,
		mod_references,
		num_versions: publishedfiledetail.revision_change_number.parse().unwrap(),
		tags: publishedfiledetail.tags,
		time_created: publishedfiledetail.time_created,
		time_updated: publishedfiledetail.time_updated,
		workshop_icon_url: publishedfiledetail.preview_url,
		children: publishedfiledetail.children,

		downloads_total: publishedfiledetail.subscriptions,
		favorited: publishedfiledetail.favorited,
		views: publishedfiledetail.views,
		playtime: publishedfiledetail.lifetime_playtime,
		followers: publishedfiledetail.followers,
		vote_data: publishedfiledetail.vote_data,
		num_comments: publishedfiledetail.num_comments_public,
	}
}

// global variable for mod cache
lazy_static! {
	static ref MOD_CACHE: std::sync::RwLock<CacheMap<u64, steamapi::PublishedFileDetails>> = std::sync::RwLock::new(CacheMap::new());
}

#[get("/mod/<modid>")]
pub async fn mod_1_4(modid: u64) -> Result<CacheResponse<Value>, APIError> {
	let mod_data = get_mod_data(modid).await?;

	let filtered_data = get_filtered_mod_info(&mod_data);
	return cached_json!(filtered_data, 3600, false);
}

#[get("/mod/<modname>")]
pub async fn mod_1_4_str(modname: &str) -> Result<CacheResponse<Value>, APIError> {
	let mod_id = modname_to_modid(modname);
	let mod_data = get_mod_data(mod_id).await?;

	let filtered_data = get_filtered_mod_info(&mod_data);
	return cached_json!(filtered_data, 3600, false);
}

fn modname_to_modid(modname: &str) -> u64 {
	0
}

async fn get_mod_data(modid: u64) -> Result<PublishedFileDetails, APIError> {
	let cache = {
		let mod_cache = MOD_CACHE.read().unwrap();
		mod_cache.get(modid, 3600).cloned()
	};

	return match cache {
		Some(cached_value) => Ok(cached_value.item),
		None => {
			let url = format!("/IPublishedFileService/GetDetails/v1/?key={}&publishedfileids%5B0%5D={}&includekvtags=true&includechildren=true&includetags=true&includevotes=true", steamapi::get_steam_key(), modid);
			let mod_info = get_steam_api_json::<steamapi::ModResponse>(&url).await
				.map_err(|_| APIError::InvalidModID(format!("Could not find a mod with the id {}", modid)))?;

			let details = mod_info.response.publishedfiledetails[0].clone();

			// update cache value
			let mut cache = MOD_CACHE.write().unwrap();
			cache.insert(modid, CacheItem {
				item: details.clone(),
				time_stamp: std::time::SystemTime::now(),
			});

			Ok(details)
		}
	}
}

// global variable for mod list cache
lazy_static! {
	static ref MODLIST_CACHE: std::sync::RwLock<CacheItem<Vec<ModInfo>>> = std::sync::RwLock::new(CacheItem::new());
}

#[get("/list")]
pub async fn list_1_4() -> Result<CacheResponse<Value>, APIError> {
	let cache = {
		let mod_cache = MODLIST_CACHE.read().unwrap();
		match mod_cache.expired(3600) {
			true => Some(mod_cache.item.clone()),
			false => None
		}
	};

	return match cache {
		Some(cached_value) => cached_json!(cached_value, 7200, false),
		None => {
			let mut mods: Vec<ModInfo> = Vec::new();
			let mut query = String::with_capacity(200);
			let mut page = 0;
			loop {
				// get list of 100 mod ids
				let url = format!("/IPublishedFileService/QueryFiles/v1/?key={}&appid={}&page={}&numperpage=100", steamapi::get_steam_key(), steamapi::APP_ID, page);
				let mod_ids = get_steam_api_json::<steamapi::ModListResponse>(&url).await;
				if mod_ids.is_err() {
					break; // if the response is empty, break the loop
				}

				// go trough each mod id in the list and add &publishedfileids[{i}]={id} to the query string
				for (i, detail) in mod_ids.unwrap().response.publishedfiledetails.iter().enumerate() {
					query.push_str(&format!("&publishedfileids%5B{}%5D={}", i, detail.publishedfileid));
				}

				// on every second page (for performance)
				if page % 2 == 0 {
					// get mod info for 200 mods in one request
					let mod_infos = get_steam_api_json::<steamapi::ModResponse>(&format!("/IPublishedFileService/GetDetails/v1/?key={}{}&includechildren=true&includekvtags=true&includechildren=true&includetags=true&includevotes=true", steamapi::get_steam_key(), query)).await?;
					mods.append(&mut mod_infos.response.publishedfiledetails.iter().map(|x| get_filtered_mod_info(&x)).collect()); // filter mod info and add to list
					query.clear(); // clear query
				}

				page += 1; // next page
			}

			// update cache value
			let mut cache = MODLIST_CACHE.write().unwrap();
			cache.item = mods.clone();
			cache.time_stamp = std::time::SystemTime::now();

			cached_json!(mods, 7200, false)
		}
	};
}

async fn get_steam_api_json<T: DeserializeOwned>(url: &str) -> Result<steamapi::Response<T>, APIError> {
	let json = get_json(&format!("https://api.steampowered.com{}", url)).await?;
	Ok(serde_json::from_value::<steamapi::Response<T>>(json)?)
}

fn find_kvtag_value(iter: &std::slice::Iter<steamapi::KVTag>, key: &str) -> Option<String> {
	for tag in iter.clone() {
		if tag.key == key {
			return Some(tag.value.clone());
		}
	};

	return None;
}