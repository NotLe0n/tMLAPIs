extern crate reqwest;

use rocket::State;
use rocket::response::content::RawHtml;
use rocket::serde::json::serde_json::{self, Value};
use rocket_cache_response::CacheResponse;
use crate::{APIError, cache, cached_json, steamapi};
use super::{responses::*, Api14State};

#[get("/mod")]
pub fn index_mod_1_4() -> RawHtml<&'static str> {
	RawHtml(r#"
		<form action="javascript: window.location.href += '/' + document.getElementById('input').value">
			<h1>Mod info (<a href="https://github.com/NotLe0n/tMLAPIs/wiki/1.4-mod">Docs</a>)</h1> 

			<label for="input">Mod ID or name:</label>
			<input type="text" id="input">
			<input type="submit" value="Go" />
		</form>

		<a href="/1.4">go back</a>
	"#)
}

#[get("/count")]
pub async fn count_1_4(state: &State<Api14State>) -> Result<Value, APIError> {
	let count = steamapi::get_mod_count(&state.steam_api_key).await?;
	Ok(serde_json::json!(count))
}

pub fn get_filtered_mod_info(publishedfiledetail: &steamapi::PublishedFileDetails) -> ModInfo {
	let publishedfiledetail = publishedfiledetail.clone();
	if publishedfiledetail.result.unwrap_or_default() > 1 {
		log::warn!("Unexpected multiple result at mod {}", publishedfiledetail.publishedfileid.clone().unwrap_or_default())
	}

	// get data from kvtags (tml specific data) field
	let mut internal_name = String::new();
	let mut author = String::new();
	let mut modside = String::new();
	let mut homepage = String::new();
	let mut deprecated_version_mod = String::new();
	let mut deprecated_version_tmodloader = String::new();
	let mut version_summary = String::new();
	let mut mod_references = String::new();
	let mut youtube: Option<String> = None;
	let mut twitter: Option<String> = None;
	let mut reddit: Option<String> = None;
	let mut facebook: Option<String> = None;
	let mut sketchfab: Option<String> = None;

	if let Some(kvtags) = publishedfiledetail.kvtags {
		// `into_iter()` moves the KVTag, so we can move its `String`s without cloning
		for steamapi::KVTag { key, value } in kvtags.into_iter() {
			match key.as_str() {
				"name"              => internal_name = value,
				"Author"            => author = value,
				"modside"           => modside = value,
				"homepage"          => homepage = value,
				"version"           => deprecated_version_mod = value,
				"modloaderversion"  => deprecated_version_tmodloader = value,
				"versionsummary"    => version_summary = value,
				"modreferences"     => mod_references = value,
				"youtube"           => youtube = (!value.is_empty()).then_some(value),
				"twitter"           => twitter = (!value.is_empty()).then_some(value),
				"reddit"            => reddit = (!value.is_empty()).then_some(value),
				"facebook"          => facebook = (!value.is_empty()).then_some(value),
				"sketchfab" 		=> sketchfab = (!value.is_empty()).then_some(value),
				tag => log::warn!("missing KV Tag: {tag}")
			}
		}
	}

	// the kvTags 'version' and 'modloaderversion' are deprecated
	let versions = if version_summary.is_empty() {
		vec![ModVersion {
			mod_version: deprecated_version_mod,
			tmodloader_version: deprecated_version_tmodloader,
		}]
	} else {
		version_summary.split(';').map(|version| {
			let mut c = version.splitn(2, ':');
			ModVersion {
				mod_version: c.next().unwrap().to_string(),
				tmodloader_version: c.next().unwrap().to_string()
			}
		}).collect()
	};

	let socials: Option<ModSocials> = 
		if youtube == None && twitter == None && reddit == None && facebook == None && sketchfab == None {
			None
		} else { 
			Some(ModSocials { 
				youtube,
				twitter,
				reddit,
				facebook,
				sketchfab,
			})
		 };

	let children = publishedfiledetail.children.map(|children| 
		children.iter()
			.filter_map(|c| c.publishedfileid.parse().ok())
			.collect()
		);

	// construct ModInfo struct
	return ModInfo{
		display_name: publishedfiledetail.title.unwrap_or_default(),
		internal_name,
		mod_id: publishedfiledetail.publishedfileid.unwrap_or_default().parse().unwrap_or_default(),
		author,
		author_id: publishedfiledetail.creator.unwrap_or_default(),
		modside,
		homepage,
		versions,
		mod_references,
		num_versions: publishedfiledetail.revision_change_number.unwrap_or_default().parse().unwrap_or_default(),
		tags: publishedfiledetail.tags,
		time_created: publishedfiledetail.time_created.unwrap_or_default(),
		time_updated: publishedfiledetail.time_updated.unwrap_or_default(),
		workshop_icon_url: publishedfiledetail.preview_url.unwrap_or_default(),
		children: children,
		description: publishedfiledetail.file_description,
		downloads_total: publishedfiledetail.subscriptions.unwrap_or_default(),
		favorited: publishedfiledetail.favorited.unwrap_or_default(),
		views: publishedfiledetail.views.unwrap_or_default(),
		playtime: publishedfiledetail.lifetime_playtime.unwrap_or_default(),
		followers: publishedfiledetail.followers.unwrap_or_default(),
		vote_data: publishedfiledetail.vote_data,
		num_comments: publishedfiledetail.num_comments_public.unwrap_or_default(),
		socials,
	}
	
}


#[get("/mod/<modid>", rank=1)]
pub async fn mod_1_4(modid: u64, state: &State<Api14State>) -> Result<CacheResponse<Value>, APIError> {
	let mod_data = get_mod_data(modid, state).await?;

	let filtered_data = get_filtered_mod_info(&mod_data);
	return cached_json!(filtered_data, 3600, false);
}

#[get("/mod/<modname>", rank=2)]
pub async fn mod_1_4_str(modname: &str, state: &State<Api14State>) -> Result<CacheResponse<Value>, APIError> {
	let mod_id = steamapi::modname_to_modid(modname, &state.steam_api_key).await?;
	let mod_data = get_mod_data(mod_id, state).await?;

	let filtered_data = get_filtered_mod_info(&mod_data);
	return cached_json!(filtered_data, 3600, false);
}

async fn get_mod_data(modid: u64, state: &State<Api14State>) -> Result<steamapi::PublishedFileDetails, APIError> {
	return match cache::lock_and_get(&state.mod_cache, modid, 3600) {
		Some(cached_value) => Ok(cached_value),
		None => {
			let details = steamapi::get_mod_info(modid, &state.steam_api_key).await?;

			// update cache value
			Ok(cache::lock_and_update(&state.mod_cache, modid, details))
		}
	}
}

pub async fn get_filtered_mod_list(steam_api_key: &str) -> Result<Vec<ModInfo>, APIError> {
	let client = reqwest::Client::new();

	let mut mods: Vec<ModInfo> = Vec::new();
	let mut next_cursor = String::from("*");
	loop {
		let list = steamapi::get_mod_list(&client, &next_cursor, steam_api_key).await?;
		if list.total == 0 || list.publishedfiledetails.is_none() {
			break;
		}

		let details = &list.publishedfiledetails.unwrap();

		// add filtered mod info to vec
		mods.extend(details.iter().map(get_filtered_mod_info));

		next_cursor = list.next_cursor.unwrap();
	}

	Ok(mods)
}