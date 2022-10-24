// import libraries
extern crate reqwest;
use rocket::serde::{json, Deserialize, Serialize};
use crate::APIError;

pub const APP_ID: &str = "1281930";

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Response<T> {
	pub response: T
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CountResponse {
	pub total: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct IDResponse {
	pub message: Option<String>,
	pub steamid: Option<String>,
	pub success: u8
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct AuthorResponse {
	pub total: u32,
	pub publishedfiledetails: Vec<PublishedFileDetails>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ModResponse {
	pub publishedfiledetails: Vec<PublishedFileDetails>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ModListResponse {
	pub publishedfiledetails: Vec<PublishedFileID>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct PublishedFileID {
	pub publishedfileid: String
} 

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
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
	pub maybe_inappropriate_sex: bool,
	pub maybe_inappropriate_violence: bool,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct SteamUserInfoResponse {
	pub players: Vec<SteamUserInfo>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct SteamUserInfo {
	pub steamid: String,
	pub communityvisibilitystate: u32,
	pub profilestate: u32,
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
	pub loccountrycode: String
}

// Holds the SteamAPI Key
lazy_static! {
	static ref STEAM_KEY: String = std::env::var("STEAM_API_KEY").expect("the 'STEAM_API_KEY' environment variable could not be read");
}

pub fn get_steam_key() -> String {
	STEAM_KEY.to_string()
}

pub async fn steamname_to_steamid(steamname: &str) -> Result<u64, APIError> {
	let steamid_url = format!("http://api.steampowered.com/ISteamUser/ResolveVanityURL/v0001/?key={}&vanityurl={}", get_steam_key(), steamname);
	let steamid_json = crate::get_json(&steamid_url).await?;
	let steamid_res: Response<IDResponse> = json::serde_json::from_value(steamid_json)?;
	let steamid: u64 = match steamid_res.response.steamid {
		Some(id) => Ok(id.parse().unwrap()),
		None => Err(APIError::SteamIDNotFound(format!("No steamid found for the specified steam name of: {}", steamname)))
	}?;

	Ok(steamid)
}

pub async fn steamid_to_steamname(steamid: u64) -> Result<String, APIError> {
	let steaminfo_url = format!("https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}", get_steam_key(), steamid);
	let steaminfo_json = crate::get_json(&steaminfo_url).await?;
	let steaminfo_res: Response<SteamUserInfoResponse> =  json::serde_json::from_value(steaminfo_json)?;
	match steaminfo_res.response.players.get(0) {
		Some(user) => Ok(user.personaname.clone()),
		None => Err(APIError::SteamIDNotFound(format!("No steam name found for the specified steam id of: {}", steamid)))
	}
}

// steamid64 is only valid in a specific number range
pub fn validate_steamid64(steamid: u64) -> Result<u64, APIError> {
	match steamid {
		0x0110000100000001..=0x01100001FFFFFFFF => Ok(steamid),
		_ => Err(APIError::InvalidSteamID(format!("The steamid '{}' is invalid", steamid)))
	}
}