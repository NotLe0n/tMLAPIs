extern crate reqwest;

use std::collections::HashMap;
use rocket::serde::{json::serde_json::{self, json, Value}, Deserialize, Serialize};
use scraper::{Html, Selector};
use crate::{APIError, steamapi};

async fn get_html(url: &str) -> Result<Html, reqwest::Error> {
	let res = reqwest::get(url).await?;
    let body = res.text().await?;
	return Ok(Html::parse_document(&body));
}

#[get("/count")]
pub async fn count_1_3() -> Result<Value, APIError> {
	let html = get_html("http://javid.ddns.net/tModLoader/modmigrationprogress.php").await?;
	let selector = Selector::parse("table > tbody > tr").unwrap(); // get all 'tr' inside 'tbody' and 'table'
	let selection = html.select(&selector); // generate iterator based on selector
	let count = selection.skip(1).count(); // count the number of elements except the first one

	// return formatted json response
	Ok(json!({
		"total": count
	}))
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
#[allow(non_snake_case)]
struct ModInfo {
	displayname: String,
	name: String,
	version: String,
	author: String,
	download: String,
	downloads: u32,
	hot: u32,
	updateTimeStamp: String,
	modloaderversion: String,
	modreferences: String,
	modside: ModSide,
	description: Option<String>,
	homepage: Option<String>,
	icon: Option<String>
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct DescriptionResponse {
	description: String,
	homepage: String
}

#[get("/mod/<modname>")]
pub async fn mod_1_3(modname: String) -> Result<Value, APIError> {
	// get mod info
	let modinfo_json = crate::get_json(format!("http://javid.ddns.net/tModLoader/tools/modinfo.php?modname={}", modname)).await?;

	let mut modinfo: ModInfo = serde_json::from_value(modinfo_json).map_err(|_| {
		APIError::InvalidModName(format!("The mod '{}' does not exist", modname))
	})?;

	// get description response; save info in DescriptionResponse struct
	let response = reqwest::Client::new()
		.post("http://javid.ddns.net/tModLoader/moddescription.php")
		.form(&HashMap::from([("modname", &modname)]))
		.send()
		.await?;

	let description_json = response.text().await.map_err(|_| {
		APIError::ReqwestError("Post request on 'http://javid.ddns.net/tModLoader/moddescription.php' failed".to_string())
	})?;

	let description_res: DescriptionResponse = serde_json::from_str(&description_json)?;
	modinfo.description = Some(description_res.description);
	modinfo.homepage = Some(description_res.homepage);

	// get icon url if it exists
	let res = reqwest::get(format!("https://mirror.sgkoi.dev/direct/{}.png", modname)).await;
	modinfo.icon = match res {
		Ok(_) => Some(format!("https://mirror.sgkoi.dev/direct/{}.png", modname)),
		Err(_) => None
	};

	Ok(json!(modinfo))
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct AuthorModInfo {
	rank: u32,
	name: String,
	downloads: u32,
	downloads_yesterday: u32
}

#[get("/author/<steamid>", rank=1)]
pub async fn author_1_3(steamid: u64) -> Result<Value, APIError> {
	return get_author_info(steamapi::validate_steamid64(steamid)?).await;
}

#[get("/author/<steamname>", rank=2)]
pub async fn author_1_3_str(steamname: String) -> Result<Value, APIError> {
	let steamid = steamapi::steamname_to_steamid(steamname).await?;
	return get_author_info(steamid).await;
}

async fn get_author_info(steamid: u64) -> Result<Value, APIError> {
	let html = get_html(&format!("http://javid.ddns.net/tModLoader/tools/ranksbysteamid.php?steamid64={}", steamid)).await?;
	let table_selector = Selector::parse("table > tbody").unwrap();
	let mut tables = html.select(&table_selector); // there are 4 tables

	let first_table = tables.next().unwrap();
	let mod_selector = &Selector::parse("tr:not(:first-child)").unwrap();
	let mods_data = first_table.select(mod_selector);
	let mut mods: Vec<AuthorModInfo> = Vec::new();

	for mod_item in mods_data {
		let td_selector = &Selector::parse("td").unwrap();
		let mut children = mod_item.select(td_selector);

		// add mod info to mod list
		// a lot of unwraps because I trust that there is no garbage
		mods.push(AuthorModInfo {
			rank: children.next().unwrap().inner_html().parse().unwrap(),
			name: children.next().unwrap().inner_html(),
			downloads: children.next().unwrap().inner_html().parse().unwrap(),
			downloads_yesterday: children.next().unwrap().inner_html().parse().unwrap() 
		});
	}

	Ok(json!({
		"total": mods.len(),
		"mods": mods
	}))
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ModListInfo {
	rank: u32,
	internal_name: String,
	display_name: String,
	downloads_total: u32,
	downloads_today: u32,
	downloads_yesterday: u32,
	mod_version: String,
	tmodloader_version: String
}

#[get("/list")]
pub async fn list_1_3() -> Result<Value, APIError> {
	let mod_selector = &Selector::parse("table > tbody > tr:not(:first-child)").unwrap();
	let td_selector = &Selector::parse("td").unwrap();

	let mut mods: Vec<ModListInfo> = Vec::new();

	// new scopes because funny errors
	{
		let html = get_html("http://javid.ddns.net/tModLoader/modmigrationprogressalltime.php").await?;
		let mod_infos = html.select(mod_selector);

		for info in mod_infos {
			let mut td = info.select(td_selector);

			mods.push(ModListInfo {
				rank: td.next().unwrap().inner_html().parse().unwrap(),
				display_name: td.next().unwrap().inner_html(),
				downloads_total: td.next().unwrap().inner_html().parse().unwrap(),
				downloads_yesterday: td.next().unwrap().inner_html().parse().unwrap(),
				mod_version: td.next().unwrap().inner_html(),
				tmodloader_version: td.next().unwrap().inner_html(),

				internal_name: "<pending>".to_string(),
				downloads_today: 0,
			})
		}
	}

	{
		let html = get_html("http://javid.ddns.net/tModLoader/modmigrationprogress.php").await?;
		let mod_infos = html.select(mod_selector);

		for info in mod_infos {
			let mut td = info.select(td_selector);

			// get index by searching for the display name in the mods array
			let mod_name = td.next().unwrap().inner_html();
			let index = mods.iter().position(|x| x.display_name == mod_name).unwrap();

			// set missing fields
			mods[index].downloads_today = td.nth(0).unwrap().inner_html().parse().unwrap();
			mods[index].internal_name = td.nth(1).unwrap().inner_html();
		}
	}

	Ok(json!(mods))
}