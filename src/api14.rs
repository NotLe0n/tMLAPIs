extern crate reqwest;

use std::slice::Iter;
use rocket::serde::{Deserialize, DeserializeOwned, Serialize};
use rocket::serde::json::serde_json::{self, json, Value};
use crate::{APIError, get_json, steamapi};
use crate::steamapi::{Child, KVTag, ModTag, VoteData};

#[get("/count")]
pub async fn count_1_4() -> Result<Value, APIError> {
    let url = format!("/IPublishedFileService/QueryFiles/v1/?key={}&appid={}&totalonly=true", steamapi::get_steam_key(), steamapi::APP_ID);
	let count = get_steam_api_json::<steamapi::CountResponse>(url).await?;

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

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct AuthorModInfo {
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
	tags: Option<Vec<ModTag>>,
	time_created: u64,
	time_updated: u64,
	workshop_icon_url: String,
	children: Option<Vec<Child>>,

	downloads_total: u32,
	favorited: u32,
	followers: u32,
	views: u64,
	vote_data: Option<VoteData>,
	playtime: String,
	num_comments: u32,
}

#[get("/author/<steamid>", rank=1)]
pub async fn author_1_4(steamid: u64) -> Result<Value, APIError> {
    get_author_info(steamid).await
}

#[get("/author/<steamname>", rank=2)]
pub async fn author_1_4_str(steamname: String) -> Result<Value, APIError> {
	let steamid = steamapi::steamname_to_steamid(steamname).await?;
	get_author_info(steamid).await
}

async fn get_author_info(steamid: u64) -> Result<Value, APIError> {
	let url = format!("/IPublishedFileService/GetUserFiles/v1/?key={}&appid={}&steamid={}&numperpage=100", steamapi::get_steam_key(), steamapi::APP_ID, steamapi::validate_steamid64(steamid)?);
	let author_data = get_steam_api_json::<steamapi::AuthorResponse>(url).await?;

	let mut mods: Vec<AuthorModInfo> = Vec::new();
	let mut total_downloads: u64 = 0;
	let mut total_favorites: u64 = 0;
	let mut total_views: u64 = 0;

	for publishedfiledetail in author_data.response.publishedfiledetails {
		// tml specific data
		let kvtags = publishedfiledetail.kvtags.unwrap();
		let kvtags_iter = kvtags.iter();

		let internal_name = find_kvtag_value(kvtags_iter.clone(), "name").unwrap();
		let author = find_kvtag_value(kvtags_iter.clone(), "Author").unwrap();
		let modside = find_kvtag_value(kvtags_iter.clone(), "modside").unwrap();
		let homepage = find_kvtag_value(kvtags_iter.clone(), "homepage").unwrap();
		let tmodloader_version = find_kvtag_value(kvtags_iter.clone(), "modloaderversion").unwrap();
		let version = find_kvtag_value(kvtags_iter.clone(), "version").unwrap();
		let mod_references = find_kvtag_value(kvtags_iter.clone(), "modreferences").unwrap();

		// increment total counts
		total_downloads += publishedfiledetail.subscriptions as u64;
		total_favorites += publishedfiledetail.favorited as u64;
		total_views += publishedfiledetail.views as u64;

		// add mod data to Vec
		mods.push(AuthorModInfo{
			display_name: publishedfiledetail.title,
			internal_name: internal_name.to_string(),
			mod_id: publishedfiledetail.publishedfileid,
			author: author.to_string(),
			author_id: publishedfiledetail.creator,
			modside: modside.to_string(),
			homepage: homepage.to_string(),
			tmodloader_version: tmodloader_version.to_string(),
			version: version.to_string(),
			mod_references: mod_references.to_string(),
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
		});
	}

	return Ok(json!({
		"mods": mods,
		"total": author_data.response.total,
		"total_downloads": total_downloads,
		"total_favorites": total_favorites,
		"total_views": total_views,
	}));
}

#[get("/mod/<modid>")]
pub async fn mod_1_4(modid: u64) -> Result<Value, APIError> {
    let url = format!("/IPublishedFileService/GetDetails/v1/?key={}&publishedfileids%5B0%5D={}&includechildren=true", steamapi::get_steam_key(), modid);
	let mod_info = get_steam_api_json::<steamapi::ModResponse>(url).await?;

	return Ok(json!(mod_info.response.publishedfiledetails[0]));
}

#[get("/list")]
pub async fn list_1_4() -> Result<Value, APIError> {
	let mut mods: Vec<steamapi::PublishedFileDetails> = Vec::new();
	let mut page = 0;
	loop {
		let url = format!("/IPublishedFileService/QueryFiles/v1/?key={}&appid={}&page={}&numperpage=100", steamapi::get_steam_key(), steamapi::APP_ID, page);
		let mod_ids = get_steam_api_json::<steamapi::ModListResponse>(url).await;
		if mod_ids.is_err() {
			break;
		}

		let mut query = "".to_owned();
		for (i, detail) in mod_ids.unwrap().response.publishedfiledetails.iter().enumerate() {
			query.push_str(&format!("&publishedfileids%5B{}%5D={}", i, detail.publishedfileid));
		}

		let mut mod_infos = get_steam_api_json::<steamapi::ModResponse>(format!("/IPublishedFileService/GetDetails/v1/?key={}{}&includechildren=true", steamapi::get_steam_key(), query)).await?;
		mods.append(&mut mod_infos.response.publishedfiledetails);

		page += 1;
	}
	
	return Ok(json!(mods));
}

async fn get_steam_api_json<T: DeserializeOwned>(url: String) -> Result<steamapi::Response<T>, APIError> {
	let json = get_json(format!("https://api.steampowered.com{}", url)).await?;
	Ok(serde_json::from_value::<steamapi::Response<T>>(json)?)
}

fn find_kvtag_value(iter: Iter<KVTag>, key: &str) -> Option<String> {
	for tag in iter {
		if tag.key == key {
			return Some(tag.value.clone());
		}
	};

	return None;
}