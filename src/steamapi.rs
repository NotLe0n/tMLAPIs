extern crate reqwest;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::serde_json::Value;
use crate::{APIError, get_json};

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Response<T> {
	pub response: T
}

// serde tries to serialize T, if it is successful => Ok, otherwise => Err
#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde", untagged)]
pub enum SteamResult<T> {
	Ok(T),
	#[allow(dead_code)] Err(Value) // fallback
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CountResponse {
	pub total: u32,
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct IDResponse {
	pub steamid: Option<String>
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ModResponse {
	pub publishedfiledetails: Vec<SteamResult<PublishedFileDetails>>
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ModListResponse {
	pub total: u32,
	pub next_cursor: Option<String>,
	pub publishedfiledetails: Option<Vec<PublishedFileDetails>>
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ModIDListResponse {
	pub publishedfiledetails: Option<Vec<PublishedFileID>>
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct PublishedFileID {
	pub publishedfileid: String
}

#[derive(Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
#[allow(dead_code)]
pub struct PublishedFileDetails {
	pub app_name: String,
	pub ban_reason: String,
	pub ban_text_check_result: u32,
	pub banned: bool,
	pub banner: String,
	pub can_be_deleted: bool,
	pub can_subscribe: bool,
	pub consumer_appid: u32,
	pub consumer_shortcutid: u32,
	pub creator: String,
	pub creator_appid: u32,
	pub favorited: u32,
	pub file_size: String,
	pub file_type: u32,
	pub file_description: Option<String>,
	pub filename: String,
	pub flags: u32,
	pub followers: u32,
	pub hcontent_file: String,
	pub hcontent_preview: String,
	pub kvtags: Option<Vec<KVTag>>,
	pub language: u32,
	pub lifetime_favorited: u32,
	pub lifetime_followers: u32,
	pub lifetime_playtime: String,
	pub lifetime_playtime_sessions: String,
	pub lifetime_subscriptions: u32,
	pub maybe_inappropriate_sex: Option<bool>,
	pub maybe_inappropriate_violence: Option<bool>,
	pub num_children: u32,
	pub num_comments_developer: Option<u32>,
	pub num_comments_public: u32,
	pub num_reports: u32,
	pub preview_file_size: String,
	pub preview_url: String,
	pub publishedfileid: String,
	pub result: u32,
	pub revision: u32,
	pub revision_change_number: String,
	pub show_subscribe_all: bool,
	pub subscriptions: u32,
	pub tags: Option<Vec<ModTag>>,
	pub time_created: u64,
	pub time_updated: u64,
	pub title: String,
	pub url: String,
	pub views: u64,
	pub visibility: u32,
	pub vote_data: Option<VoteData>,
	pub workshop_accepted: bool,
	pub workshop_file: bool,
	pub children: Option<Vec<Child>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Child {
	pub publishedfileid: String,
	pub sortorder: u32,
	pub file_type: u32
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct VoteData {
	pub score: f64,
	pub votes_up: u32,
	pub votes_down: u32
}

#[derive(Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct KVTag {
	pub key: String,
	pub value: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ModTag {
	pub tag: String,
	pub display_name: String
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct SteamUserInfoResponse {
	pub players: Vec<SteamUserInfo>
}

#[derive(Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
#[allow(dead_code)]
pub struct SteamUserInfo {
	pub steamid: String,
	pub communityvisibilitystate: u32,
	pub profilestate: Option<u32>,
	pub personaname: String,
	pub profileurl: String,
	pub avatar: String,
	pub avatarmedium: String,
	pub avatarfull: String,
	pub avatarhash: String,
	pub lastlogoff: Option<u64>,
	pub personastate: u32,
	pub primaryclanid: String,
	pub timecreated: u64,
	pub personastateflags: u32,
	pub loccountrycode: Option<String>
}

const STEAM_API_URL: &str = "https://api.steampowered.com";
pub const APP_ID: &str = "1281930";

pub async fn get_mod_count(api_key: &str) -> Result<CountResponse, APIError> {
	let url = format!("{STEAM_API_URL}/IPublishedFileService/QueryFiles/v1/?key={}&appid={APP_ID}&totalonly=true", api_key);
	let res = get_json::<Response<CountResponse>>(&url).await?;
	Ok(res.response)
}

pub async fn get_user_mods(steamid: u64, api_key: &str) -> Result<ModListResponse, APIError> {
	let url = format!("{STEAM_API_URL}/IPublishedFileService/GetUserFiles/v1/?key={}&appid={APP_ID}&steamid={}&numperpage=100", api_key, steamid);
	let res = get_json::<Response<ModListResponse>>(&url).await?;
	Ok(res.response)
}

pub async fn get_mod_info(modid: u64 , api_key: &str) -> Result<PublishedFileDetails, APIError> {
	let url = format!("{STEAM_API_URL}/IPublishedFileService/GetDetails/v1/?key={}&publishedfileids%5B0%5D={}&includekvtags=true&includechildren=true&includetags=true&includevotes=true", api_key, modid);
	let res = get_json::<Response<ModResponse>>(&url).await?;
	
	match res.response.publishedfiledetails[0].clone() {
		SteamResult::Ok(pfd) => Ok(pfd),
		SteamResult::Err(_) => Err(APIError::InvalidModID(format!("Could not find a mod with the id {}", modid)))
	}
}

pub async fn modname_to_modid(modname: &str, api_key: &str) -> Result<u64, APIError> {
	let url = format!("{STEAM_API_URL}/IPublishedFileService/QueryFiles/v1/?key={}&input_json=%7B%22appid%22:{APP_ID},%20%22required_kv_tags%22:[%7B%22key%22:%22name%22,%22value%22:%22{}%22%7D]%7D", api_key, modname);
	let res = get_json::<Response<ModIDListResponse>>(&url).await?;
	
	match res.response.publishedfiledetails {
		Some(pfd) => Ok(pfd[0].publishedfileid.parse().unwrap()),
		None => Err(APIError::InvalidModID(format!("Could not find mod with the provided name: {}", modname)))
	}
}

pub async fn get_mod_list(client: &reqwest::Client, cursor: &str, api_key: &str) -> Result<ModListResponse, APIError> {
	let url = format!("{STEAM_API_URL}/IPublishedFileService/QueryFiles/v1/?key={}&appid={APP_ID}&cursor={}&numperpage=10000&cache_max_age_seconds=0&return_details=true&return_kv_tags=true&return_children=true&return_tags=true&return_vote_data=true",
						api_key, urlencoding::encode(cursor));
	let res = client.get(url).send().await?;
	Ok(res.json::<Response<ModListResponse>>().await?.response)
}

pub async fn steamname_to_steamid(steamname: &str, api_key: &str) -> Result<u64, APIError> {
	let url = format!("{STEAM_API_URL}/ISteamUser/ResolveVanityURL/v1/?key={}&vanityurl={}", api_key, steamname);
	let res: Response<IDResponse> = get_json(&url).await?;
	
	match res.response.steamid {
		Some(id) => Ok(id.parse().unwrap()),
		None => Err(APIError::SteamIDNotFound(format!("No steamid found for the specified steam name of: {}", steamname)))
	}
}

pub async fn get_user_info(steamid: u64, api_key: &str) -> Result<SteamUserInfo, APIError> {
	let url = format!("{STEAM_API_URL}/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}", api_key, steamid);
	let res: Response<SteamUserInfoResponse> = get_json(&url).await?;
	
	match res.response.players.first() {
		Some(user) => Ok(user.clone()),
		None => Err(APIError::SteamIDNotFound(format!("No steam user found for the specified steam id of: {}", steamid)))
	}
}

// steamid64 is only valid in a specific number range
pub fn validate_steamid64(steamid: u64) -> Result<u64, APIError> {
	match steamid {
		0x0110000100000001..=0x01100001FFFFFFFF => Ok(steamid),
		_ => Err(APIError::InvalidSteamID(format!("The steamid '{}' is invalid", steamid)))
	}
}