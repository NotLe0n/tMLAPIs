extern crate reqwest;
use std::collections::HashMap;

use rocket::serde::{Deserialize, DeserializeOwned, Serialize};
use rocket::serde::json::serde_json::Value;
use crate::APIError;

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
	pub steamid: Option<String>,
	#[allow(dead_code)] pub success: u32
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
	pub app_name: Option<String>,
	pub ban_reason: Option<String>,
	pub ban_text_check_result: Option<u32>,
	pub banned: Option<bool>,
	pub banner: Option<String>,
	pub can_be_deleted: Option<bool>,
	pub can_subscribe: Option<bool>,
	pub consumer_appid: Option<u32>,
	pub consumer_shortcutid: Option<u32>,
	pub creator: Option<String>,
	pub creator_appid: Option<u32>,
	pub favorited: Option<u32>,
	pub file_size: Option<String>,
	pub file_type: Option<u32>,
	pub file_description: Option<String>,
	pub filename: Option<String>,
	pub flags: Option<u32>,
	pub followers: Option<u32>,
	pub hcontent_file: Option<String>,
	pub hcontent_preview: Option<String>,
	pub kvtags: Option<Vec<KVTag>>,
	pub language: Option<u32>,
	pub lifetime_favorited: Option<u32>,
	pub lifetime_followers: Option<u32>,
	pub lifetime_playtime: Option<String>,
	pub lifetime_playtime_sessions: Option<String>,
	pub lifetime_subscriptions: Option<u32>,
	pub maybe_inappropriate_sex: Option<bool>,
	pub maybe_inappropriate_violence: Option<bool>,
	pub num_children: Option<u32>,
	pub num_comments_developer: Option<u32>,
	pub num_comments_public: Option<u32>,
	pub num_reports: Option<u32>,
	pub preview_file_size: Option<String>,
	pub preview_url: Option<String>,
	pub publishedfileid: Option<String>,
	pub result: Option<u32>,
	pub revision: Option<u32>,
	pub revision_change_number: Option<String>,
	pub show_subscribe_all: Option<bool>,
	pub subscriptions: Option<u32>,
	pub tags: Option<Vec<ModTag>>,
	pub time_created: Option<u64>,
	pub time_updated: Option<u64>,
	pub title: Option<String>,
	pub url: Option<String>,
	pub views: Option<u64>,
	pub visibility: Option<u32>,
	pub vote_data: Option<VoteData>,
	pub workshop_accepted: Option<bool>,
	pub workshop_file: Option<bool>,
	pub children: Option<Vec<Child>>,
	// 1: NudityOrSexualContent, 2: FrequentViolenceOrGore, 3: AdultOnlySexualContent, 4: GratuitousSexualContent, 5: AnyMatureContent
	pub content_descriptorids: Option<Vec<u32>>, 
	pub available_revisions: Option<Vec<u32>>,
	
	#[serde(flatten)]
    extra: HashMap<String, Value>,
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

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
#[allow(dead_code)]
pub enum ContentDescriptor {
	NudityOrSexualContent = 1,
	FrequentViolenceOrGore,
	AdultOnlySexualContent,
	GratuitousSexualContent,
	AnyMatureContent
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
	pub personastate: Option<u32>,
	pub primaryclanid: Option<String>,
	pub timecreated: Option<u64>,
	pub personastateflags: Option<u32>,
	pub loccountrycode: Option<String>,
}

fn check_missing_fields(files: &Vec<PublishedFileDetails>) {
	if files.iter().any(|f| !f.extra.is_empty()) {
		for f in files.iter().filter(|f| !f.extra.is_empty()) {
			log::warn!("mod[{}] missing fields: {:?}", f.publishedfileid.clone().unwrap(), f.extra);
		}
	}
}

const STEAM_API_URL: &str = "https://api.steampowered.com";
pub const APP_ID: &str = "1281930";

// does a get reqwests on the specified URL and Returns a Json<String> if successful or a Status if it errored
async fn get_steam<T: DeserializeOwned>(url: &str) -> Result<T, APIError> {
	let res = reqwest::get(format!("{STEAM_API_URL}{url}")).await?;
	log::debug!("Requesting SteamAPI at: {STEAM_API_URL}{url}");
	Ok(res.json::<Response<T>>().await?.response)
}

pub async fn get_mod_count(api_key: &str) -> Result<CountResponse, APIError> {
	let url = format!("/IPublishedFileService/QueryFiles/v1/?key={}&appid={APP_ID}&totalonly=true", api_key);
	Ok(get_steam::<CountResponse>(&url).await?)
}

pub async fn get_user_mods(steamid: u64, api_key: &str) -> Result<ModListResponse, APIError> {
	let url = format!("/IPublishedFileService/GetUserFiles/v1/?key={}&appid={APP_ID}&steamid={}&numperpage=100&return_short_description=false&return_children=true", api_key, steamid);
	let res = get_steam::<ModListResponse>(&url).await?;

	if let Some(files) = res.publishedfiledetails.as_ref() {
		check_missing_fields(files);
	}
	
	Ok(res)
}

pub async fn get_mod_info(modid: u64 , api_key: &str) -> Result<PublishedFileDetails, APIError> {
	let url = format!("/IPublishedFileService/GetDetails/v1/?key={api_key}&appid={APP_ID}\
		&publishedfileids%5B0%5D={modid}\
		&includekvtags=true\
		&includechildren=true\
		&includetags=true\
		&includevotes=true"
	);
	let res = get_steam::<ModResponse>(&url).await?;
	
	match res.publishedfiledetails[0].clone() {
		SteamResult::Ok(pfd) => {
			if !pfd.extra.is_empty() {
				log::warn!("Unknown fields at mod[{}]: {:?}", pfd.publishedfileid.clone().unwrap(), pfd.extra.keys());
			}
			Ok(pfd)
		},
		SteamResult::Err(_) => Err(APIError::InvalidModID(modid))
	}
}

pub async fn modname_to_modid(modname: &str, api_key: &str) -> Result<u64, APIError> {
	let url = format!(r#"/IPublishedFileService/QueryFiles/v1/?key={api_key}&appid={APP_ID}&input_json={{"required_kv_tags":[{{"key":"name","value":"{modname}"}}]}}"#);
	let res = get_steam::<ModIDListResponse>(&url).await?;
	
	match res.publishedfiledetails {
		Some(pfd) => Ok(pfd[0].publishedfileid.parse().unwrap()),
		None => Err(APIError::InvalidModName(modname.to_owned()))
	}
}

// Idea: filter by tag: &requiredtags[0]=Both&requiredtags[1]=Client&requiredtags[2]=Server&requiredtags[3]=NoSync&match_all_tags=false
pub async fn get_mod_list(client: &reqwest::Client, cursor: &str, api_key: &str) -> Result<ModListResponse, APIError> {
	let c = urlencoding::encode(cursor);
	let url = format!("{STEAM_API_URL}/IPublishedFileService/QueryFiles/v1/?key={api_key}&appid={APP_ID}&cursor={c}\
		&numperpage=10000\
		&cache_max_age_seconds=0\
		&return_details=true\
		&return_kv_tags=true\
		&return_children=true\
		&return_tags=true\
		&return_vote_data=true"
	);

	log::debug!("Requesting SteamAPI at: {url}");
	let res = client.get(url).send().await?;
    let mod_list = res.json::<Response<ModListResponse>>().await?.response;

	if let Some(files) = mod_list.publishedfiledetails.as_ref() {
    	check_missing_fields(files);
	}

	Ok(mod_list)
}

pub async fn steamname_to_steamid(steamname: &str, api_key: &str) -> Result<u64, APIError> {
	let url = format!("/ISteamUser/ResolveVanityURL/v1/?key={}&vanityurl={}", api_key, steamname);
	let res: IDResponse = get_steam(&url).await?;
	
	match res.steamid {
		Some(id) => Ok(id.parse().unwrap()),
		None => Err(APIError::SteamNameNotResolveable(steamname.to_owned()))
	}
}

pub async fn get_user_info(steamid: u64, api_key: &str) -> Result<SteamUserInfo, APIError> {
	let url = format!("/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}", api_key, steamid);
	let res: SteamUserInfoResponse = get_steam(&url).await?;
	
	match res.players.first() {
		Some(user) => Ok(user.clone()),
		None => Err(APIError::SteamIDNotFound(steamid))
	}
}

pub async fn get_users_info(steamids: &[u64], api_key: &str) -> Result<Vec<SteamUserInfo>, APIError> {
	let steamids_csv = steamids.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
	let url = format!("/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}", api_key, steamids_csv);
	let res: SteamUserInfoResponse = get_steam(&url).await?;
	
	return Ok(res.players);
}

// steamid64 is only valid in a specific number range
pub fn validate_steamid64(steamid: u64) -> Result<u64, APIError> {
	match steamid {
		0x0110000100000001..=0x01100001FFFFFFFF => Ok(steamid),
		_ => Err(APIError::InvalidSteamID(steamid))
	}
}