use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub enum ModSide {
	Both,
	Client,
	Server,
	NoSync
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
#[allow(non_snake_case)]
pub struct ModInfo {
	#[serde(rename(serialize = "display_name"))] pub displayname: String,
	#[serde(rename(serialize = "internal_name"))] pub name: String,
	pub version: String,
	pub author: String,
	#[serde(rename(serialize = "download_link"))] pub download: String,
	#[serde(rename(serialize = "downloads_total"))] pub downloads: u32,
	#[serde(rename(serialize = "downloads_yesterday"))] pub hot: u32,
	#[serde(rename(serialize = "last_updated"))] pub updateTimeStamp: String,
	#[serde(rename(serialize = "tmodloader_version"))] pub modloaderversion: String,
	pub modreferences: String,
	pub modside: ModSide,
	pub description: Option<String>,
	pub homepage: Option<String>,
	pub icon: Option<String>
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DescriptionResponse {
	pub description: String,
	pub homepage: String
}

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AuthorModInfo {
	pub rank: u32,
	pub display_name: String,
	pub downloads_total: u32,
	pub downloads_yesterday: u32
}

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct MaintainedModInfo {
	pub internal_name: String,
	pub downloads_total: u32,
	pub downloads_yesterday: u32
}

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AuthorInfo {
	pub steam_id: u64,
	pub steam_name: String,
	pub downloads_total: u32,
	pub downloads_yesterday: u32,
	pub total: u32,
	pub mods: Vec<AuthorModInfo>,
	pub maintained_mods: Vec<MaintainedModInfo>
}

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ModListInfo {
	pub rank: u32,
	pub internal_name: String,
	pub display_name: String,
	pub downloads_total: u32,
	pub downloads_today: u32,
	pub downloads_yesterday: u32,
	pub mod_version: String,
	pub tmodloader_version: String
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ModHistory {
	pub version: String,
	pub downloads_total: u32,
	pub tmodloader_version: String,
	pub publish_date: String,
}