extern crate reqwest;

use rocket::serde::{Deserialize, DeserializeOwned, Serialize};
use rocket::serde::json::serde_json::{self, json, Value};
use rocket_cache_response::CacheResponse;
use crate::{APIError, cached_json, get_json, steamapi};
use crate::cache::{CacheItem, CacheMap};
use urlencoding;

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
	description: Option<String>,
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
	steam_id: u64,
	steam_name: String,
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
			let author_data = get_steam_api_json::<steamapi::ModListResponse>(&url).await
				.map_err(|_| APIError::InvalidSteamID(format!("Could not find an author with the id {}", steamid)))?;

			let mut mods: Vec<ModInfo> = Vec::new();
			let mut total_downloads: u64 = 0;
			let mut total_favorites: u64 = 0;
			let mut total_views: u64 = 0;

			// go through each mod
			for publishedfiledetail in author_data.response.publishedfiledetails.unwrap() {
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
				steam_id: steamid,
				steam_name: steamapi::steamid_to_steamname(steamid).await?,
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
	let version_old = find_kvtag_value(&kvtags_iter, "version").unwrap_or_default();
	let version_summary = find_kvtag_value(&kvtags_iter, "versionsummary").unwrap_or_default();
	let mod_references = find_kvtag_value(&kvtags_iter, "modreferences").unwrap_or_default();

	// the kvTag 'version' is deprecated
	let version = if version_summary == "" {
		version_old
	} else {
		format!("v{}", version_summary.split(&[':', ';']).nth(1).unwrap_or_default()) // tmod_version1:mod_version1;tmod_version2:mod_version2; ...
	};

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
		description: publishedfiledetail.file_description,
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

#[get("/mod/<modid>", rank=1)]
pub async fn mod_1_4(modid: u64) -> Result<CacheResponse<Value>, APIError> {
	let mod_data = get_mod_data(modid).await?;

	let filtered_data = get_filtered_mod_info(&mod_data);
	return cached_json!(filtered_data, 3600, false);
}

#[get("/mod/<modname>", rank=2)]
pub async fn mod_1_4_str(modname: &str) -> Result<CacheResponse<Value>, APIError> {
	let mod_id = modname_to_modid(modname).await?;
	let mod_data = get_mod_data(mod_id).await?;

	let filtered_data = get_filtered_mod_info(&mod_data);
	return cached_json!(filtered_data, 3600, false);
}

async fn modname_to_modid(modname: &str) -> Result<u64, APIError> {
	let url = format!("/IPublishedFileService/QueryFiles/v1/?key={}&input_json=%7B%22appid%22:{},%20%22required_kv_tags%22:[%7B%22key%22:%22name%22,%22value%22:%22{}%22%7D]%7D", steamapi::get_steam_key(), steamapi::APP_ID, modname);
	let mod_id = get_steam_api_json::<steamapi::ModIDListResponse>(&url).await
		.map_err(|_| APIError::InvalidModID(format!("Could not find mod with the provided name: {}", modname)))?;
	Ok(mod_id.response.publishedfiledetails[0].publishedfileid.parse().unwrap())
}

async fn get_mod_data(modid: u64) -> Result<steamapi::PublishedFileDetails, APIError> {
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
			let mut next_cursor = String::from("*");
			loop {
				// get list of 100 mod ids
				let url = format!("/IPublishedFileService/QueryFiles/v1/?key={}&appid={}&cursor={}&numperpage=10000&cache_max_age_seconds=0&return_details=true&return_kv_tags=true&return_children=true&return_tags=true&return_vote_data=true",
								  steamapi::get_steam_key(), steamapi::APP_ID, urlencoding::encode(&next_cursor));
				let list_res = get_steam_api_json::<steamapi::ModListResponse>(&url).await
					.expect("mod list request failed!");

				if list_res.response.total == 0 || list_res.response.publishedfiledetails.is_none() {
					break;
				}

				let details = &list_res.response.publishedfiledetails.unwrap();

				// add filtered mod info to vec
				mods.append(
					&mut details.iter().map(|x| get_filtered_mod_info(&x)).collect()
				);

				next_cursor = list_res.response.next_cursor.unwrap();
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