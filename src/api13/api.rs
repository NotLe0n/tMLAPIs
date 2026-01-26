extern crate reqwest;

use std::collections::HashMap;
use rocket::State;
use rocket::serde::json::serde_json::{self, json, Value};
use rocket_cache_response::CacheResponse;
use scraper::{Html, Selector};
use crate::{APIError, cached_json, steamapi, steamapi::get_user_info};
use crate::cache;
use crate::api13::responses::*;

use super::Api13State;

const API_URL: &str = "http://javid.ddns.net/tModLoader";

async fn get_html(url: &str) -> Result<Html, reqwest::Error> {
	let res = reqwest::get(url).await?;
	let body = res.text().await?;
	return Ok(Html::parse_document(&body));
}

#[get("/count")]
pub async fn count_1_3() -> Result<Value, APIError> {
	let html = get_html(&format!("{API_URL}/modmigrationprogress.php")).await?;
	let selector = Selector::parse("table > tbody > tr")?; // get all 'tr' inside 'tbody' and 'table'
	let selection = html.select(&selector); // generate iterator based on selector
	let count = selection.skip(1).count(); // count the number of elements except the first one

	// return formatted json response
	Ok(json!({
		"total": count
	}))
}

#[get("/mod/<modname>")]
pub async fn mod_1_3(modname: &str, state: &State<Api13State>) -> Result<CacheResponse<Value>, APIError> {
	let mod_info = match cache::lock_and_get(&state.mod_cache, modname.to_owned(), 3600) {
		Some(cached_value) => cached_value,
		None => {
			let client = reqwest::Client::new();

			// get mod info
			let res = client.get(&format!("{API_URL}/tools/modinfo.php?modname={}", modname)).send().await?;
			let mut modinfo: ModInfo = res.json::<ModInfo>().await.map_err(|_| {
				APIError::InvalidModName(modname.to_owned())
			})?;

			// get description response; save info in DescriptionResponse struct
			let description: DescriptionResponse = client 
				.post(format!("{API_URL}/moddescription.php"))
				.form(&HashMap::from([("modname", &modname)]))
				.send().await?
				.json().await?;

			modinfo.description = Some(description.description);
			modinfo.homepage = Some(description.homepage);

			// get icon url if it exists
			let icon_url = format!("{API_URL}/modicons/modiconuploads/{}_{}.png", modname, modinfo.version);
			let res = client.get(&icon_url).send().await;
			modinfo.icon = match res {
				Ok(_) => Some(icon_url),
				Err(_) => None
			};

			cache::lock_and_update(&state.mod_cache, modname.to_owned(), modinfo)
		}
	};

	return cached_json!(mod_info, 3600, false);
}

#[get("/author/<steamid>", rank=1)]
pub async fn author_1_3(steamid: u64, state: &State<Api13State>) -> Result<CacheResponse<Value>, APIError> {
	return get_author_info(steamapi::validate_steamid64(steamid)?, state).await;
}

#[get("/author/<steamname>", rank=2)]
pub async fn author_1_3_str(steamname: &str, state: &State<Api13State>) -> Result<CacheResponse<Value>, APIError> {
	let steamid = steamapi::steamname_to_steamid(steamname, &state.steam_api_key).await?;
	return get_author_info(steamid, state).await;
}

async fn get_author_info(steamid: u64, state: &State<Api13State>) -> Result<CacheResponse<Value>, APIError> {
	let author = match cache::lock_and_get(&state.author_cache, steamid, 3600) {
		Some(cached_value) => cached_value,
		None => {
			let steam_user = get_user_info(steamid, &state.steam_api_key).await?;

			let td_selector = &Selector::parse("td")?;

			let html = get_html(&format!("{API_URL}/tools/ranksbysteamid.php?steamid64={}", steamid)).await?;
			let table_selector = Selector::parse("table > tbody")?;
			let mut tables = html.select(&table_selector); // there are 4 tables

			let first_table = tables.next().unwrap();
			let mod_selector = &Selector::parse("tr:not(:first-child)")?;
			let mods_data = first_table.select(mod_selector);
			let mut mods: Vec<AuthorModInfo> = Vec::new();

			let mut total_downloads: u32 = 0;
			let mut total_downloads_yesterday: u32 = 0;

			for mod_item in mods_data {
				let mut children = mod_item.select(td_selector);

				// add mod info to mod list
				// a lot of unwraps because I trust that there is no garbage
				let rank = children.next().unwrap().inner_html().parse().unwrap();
				let display_name = children.next().unwrap().inner_html();
				let downloads_total = children.next().unwrap().inner_html().parse().unwrap();
				let downloads_yesterday = children.next().unwrap().inner_html().parse().unwrap();

				// increment totals
				total_downloads += downloads_total;
				total_downloads_yesterday += downloads_yesterday;

				mods.push(AuthorModInfo { rank, display_name, downloads_total, downloads_yesterday });
			}

			let maintainer_table = tables.last().unwrap();
			let maintained_mods_selector = Selector::parse("tr:not(:first-child)")?;
			let maintained_mods = maintainer_table.select(&maintained_mods_selector);

			let mut maintained_mods_infos: Vec<MaintainedModInfo> = Vec::new();

			for maintained_mod in maintained_mods {
				let mut children = maintained_mod.select(td_selector);

				maintained_mods_infos.push(MaintainedModInfo {
					internal_name: children.next().unwrap().inner_html(),
					downloads_total: children.next().unwrap().inner_html().parse().unwrap(),
					downloads_yesterday: children.next().unwrap().inner_html().parse().unwrap()
				})
			}

			let author = AuthorInfo {
				steam_id: steamid,
				steam_name: steam_user.personaname,
				downloads_total: total_downloads,
				downloads_yesterday: total_downloads_yesterday,
				total: mods.len() as u32,
				mods,
				maintained_mods: maintained_mods_infos
			};

			cache::lock_and_update(&state.author_cache, steamid, author)
		}
	};

	return cached_json!(author, 3600, false);
}

#[get("/list")]
pub async fn list_1_3(state: &State<Api13State>) -> Result<CacheResponse<Value>, APIError> {
	let cache = {
		let mod_cache = state.mod_list_cache.lock().unwrap();
		match mod_cache.expired(3600) {
			true => Some(mod_cache.item.clone()),
			false => None
		}
	};

	let mods = match cache {
		Some(cached_value) => cached_value,
		None => {
			let mod_selector = &Selector::parse("table > tbody > tr:not(:first-child)")?;
			let td_selector = &Selector::parse("td")?;

			let mut mods: Vec<ModListInfo> = Vec::new();

			// new scopes because funny errors
			{
				let html = get_html(&format!("{API_URL}/modmigrationprogressalltime.php")).await?;
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
				let html = get_html(&format!("{API_URL}/modmigrationprogress.php")).await?;
				let mod_infos = html.select(mod_selector);

				for info in mod_infos {
					let mut td = info.select(td_selector);

					// get index by searching for the display name in the mods array
					let mod_name = td.next().unwrap().inner_html();
					let index = mods.iter().position(|x| x.display_name == mod_name).unwrap();

					// set missing fields
					mods[index].downloads_today = td.nth(0).unwrap().inner_html().parse().unwrap();
					mods[index].internal_name = td.nth(2).unwrap().inner_html();
				}
			}

			// update cache value
			let mut cache = state.mod_list_cache.lock().unwrap();
			cache.item = mods.clone();
			cache.time_stamp = std::time::SystemTime::now();

			mods
		}
	};

	return cached_json!(mods, 7200, false)
}


#[get("/history/<modname>")]
pub async fn history_1_3(modname: &str) -> Result<CacheResponse<Value>, APIError> {
	let html = get_html(&format!("{API_URL}/tools/moddownloadhistory.php?modname={}", modname)).await?;
	let versions_selector = &Selector::parse("table > tbody > tr:not(:first-child)")?;
	let versions = html.select(versions_selector);

	let td_selector = &Selector::parse("td")?;

	let mut history: Vec<ModHistory> = Vec::new();
	for version in versions {
		let mut version_data = version.select(td_selector);

		history.push(ModHistory{
			version: version_data.next().unwrap().inner_html(),
			downloads_total: version_data.next().unwrap().inner_html().parse().unwrap(),
			tmodloader_version: version_data.next().unwrap().inner_html(),
			publish_date: version_data.next().unwrap().inner_html()
		});
	}

	return cached_json!(history, 7200, false);
}