extern crate reqwest;

use rocket::serde::json::serde_json::{self, json, Value};
use crate::{APIError, get_json, steamapi};

#[get("/count")]
pub async fn count_1_4() -> Result<Value, APIError> {
    let url = format!("https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/?key={}&appid={}&totalonly=true", steamapi::get_steam_key(), steamapi::APP_ID);
	let json = get_json(url).await?;
	let res: steamapi::Response<steamapi::CountResponse> = serde_json::from_value(json).unwrap();

	return Ok(json!(res.response));
}

#[get("/author/<steamid>", rank=1)]
pub async fn author_1_4(steamid: u64) -> Result<Value, APIError> {
    let url = format!("https://api.steampowered.com/IPublishedFileService/GetUserFiles/v1/?key={}&appid={}&steamid={}&numperpage=100", steamapi::get_steam_key(), steamapi::APP_ID, steamapi::validate_steamid64(steamid)?);
	let json = get_json(url).await?;
	let author: steamapi::Response<steamapi::AuthorResponse> = serde_json::from_value(json)?;

	return Ok(json!(author.response));
}

#[get("/author/<steamname>", rank=2)]
pub async fn author_1_4_str(steamname: String) -> Result<Value, APIError> {
	let steamid = steamapi::steamname_to_steamid(steamname).await?;
	let url = format!("https://api.steampowered.com/IPublishedFileService/GetUserFiles/v1/?key={}&appid={}&steamid={}&numperpage=100", steamapi::get_steam_key(), steamapi::APP_ID, steamapi::validate_steamid64(steamid)?);
	let json = get_json(url).await?;
	let author: steamapi::Response<steamapi::AuthorResponse> = serde_json::from_value(json)?;

	return Ok(json!(author.response));
}

#[get("/mod/<modid>")]
pub async fn mod_1_4(modid: u64) -> Result<Value, APIError> {
    let url = format!("https://api.steampowered.com/IPublishedFileService/GetDetails/v1/?key={}&publishedfileids%5B0%5D={}&includechildren=true", steamapi::get_steam_key(), modid);
	let json = get_json(url).await?;
	let modinfo: steamapi::Response<steamapi::ModResponse> = serde_json::from_value(json)?;
	
	return Ok(json!(modinfo.response.publishedfiledetails[0]));
}

#[get("/list")]
pub async fn list_1_4() -> Result<Value, APIError> {
	let mut mods: Vec<steamapi::PublishedFileDetails> = Vec::new();
	let mut page = 0;
	loop {
		let url = format!("https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/?key={}&appid={}&page={}&numperpage=100", steamapi::get_steam_key(), steamapi::APP_ID, page);
		let idlist_json = get_json(url).await?;
		let modids: Result<steamapi::Response<steamapi::ModListResponse>, _> = serde_json::from_value(idlist_json);
		if modids.is_err() {
			break;
		}

		let mut query = "".to_owned();
		for (i, detail) in modids.unwrap().response.publishedfiledetails.iter().enumerate() {
			query.push_str(&format!("&publishedfileids%5B{}%5D={}", i, detail.publishedfileid));
		}

		let infolist_json = get_json(format!("https://api.steampowered.com/IPublishedFileService/GetDetails/v1/?key={}{}&includechildren=true", steamapi::get_steam_key(), query)).await?;
		let mut modinfos: steamapi::Response<steamapi::ModResponse> = serde_json::from_value(infolist_json)?;
		mods.append(&mut modinfos.response.publishedfiledetails);

		page += 1;
	}
	
	return Ok(json!(mods));
}