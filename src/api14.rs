extern crate reqwest;

use rocket::serde::DeserializeOwned;
use rocket::serde::json::serde_json::{self, json, Value};
use crate::{APIError, get_json, steamapi};

#[get("/count")]
pub async fn count_1_4() -> Result<Value, APIError> {
    let url = format!("/IPublishedFileService/QueryFiles/v1/?key={}&appid={}&totalonly=true", steamapi::get_steam_key(), steamapi::APP_ID);
	let count = get_steam_api_json::<steamapi::CountResponse>(url).await?;

	return Ok(json!(count.response));
}

#[get("/author/<steamid>", rank=1)]
pub async fn author_1_4(steamid: u64) -> Result<Value, APIError> {
    let url = format!("/IPublishedFileService/GetUserFiles/v1/?key={}&appid={}&steamid={}&numperpage=100", steamapi::get_steam_key(), steamapi::APP_ID, steamapi::validate_steamid64(steamid)?);
	let author = get_steam_api_json::<steamapi::AuthorResponse>(url).await?;

	return Ok(json!(author.response));
}

#[get("/author/<steamname>", rank=2)]
pub async fn author_1_4_str(steamname: String) -> Result<Value, APIError> {
	let steamid = steamapi::steamname_to_steamid(steamname).await?;
	let url = format!("/IPublishedFileService/GetUserFiles/v1/?key={}&appid={}&steamid={}&numperpage=100", steamapi::get_steam_key(), steamapi::APP_ID, steamapi::validate_steamid64(steamid)?);
	let author = get_steam_api_json::<steamapi::AuthorResponse>(url).await?;

	return Ok(json!(author.response));
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