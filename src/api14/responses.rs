use rocket::serde::{Serialize, Deserialize};
use crate::steamapi;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum ModSide {
	Both,
	Client,
	Server,
	NoSync
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ModVersion {
	pub mod_version: String,
	pub tmodloader_version: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ModSocials {
	pub youtube: Option<String>,
	pub twitter: Option<String>,
	pub reddit: Option<String>,
	pub facebook: Option<String>,
	pub sketchfab: Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ModInfo {
	pub display_name: String,
	pub internal_name: String,
	pub mod_id: u64,
	pub author: String,
	pub author_id: u64,
	pub modside: String,
	pub homepage: String,
	pub versions: Vec<ModVersion>,
	pub mod_references: String,
	pub num_versions: u32,
	pub tags: Option<Vec<steamapi::ModTag>>,
	pub time_created: u64,
	pub time_updated: u64,
	pub workshop_icon_url: String,
	pub children: Option<Vec<u64>>,
	pub description: Option<String>,
	pub downloads_total: u32,
	pub favorited: u32,
	pub followers: u32,
	pub views: u64,
	pub vote_data: Option<steamapi::VoteData>,
	pub playtime: String,
	pub num_comments: u32,
	pub socials: Option<ModSocials>
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AuthorInfo {
	pub steam_id: u64,
	pub steam_name: String,
	pub steam_avatar: String,
	pub mods: Vec<ModInfo>,
	pub total: u32,
	pub total_downloads: u64,
	pub total_favorites: u64,
	pub total_views: u64,
}
