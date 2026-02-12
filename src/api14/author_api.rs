use rocket::State;
use rocket::response::content::RawHtml;
use rocket::serde::json::serde_json::{self, Value};
use rocket_cache_response::CacheResponse;
use std::collections::HashMap;

use crate::api14::mod_api::get_filtered_mod_info;
use crate::{api14::Api14State, cache, cached_json, steamapi};
use crate::api_error::APIError;
use crate::api14::responses::{AuthorInfo, ModInfo};

#[get("/author")]
pub fn index_author_1_4() -> RawHtml<&'static str> {
	RawHtml(r#"
		<h1>Author info (<a href="https://github.com/NotLe0n/tMLAPIs/wiki/1.4-author">Docs</a>)</h1> 

		<form action="javascript: window.location.href += '/' + document.getElementById('input').value">
			<label for="input">SteamID64 or vanity name:</label>
			<input type="text" id="input">
			<input type="submit" value="Go" />
		</form>

		<a href="/1.4">go back</a>
	"#)
}

async fn get_author_info(steamid: u64, state: &State<Api14State>) -> Result<CacheResponse<Value>, APIError> {
	let author = match cache::lock_and_get(&state.author_cache, steamid, 3600) {
		Some(cached_value) => cached_value,
		None => {
			let steam_user = steamapi::get_user_info(steamid, &state.steam_api_key).await?;
			let author_data = steamapi::get_user_mods(steamid, &state.steam_api_key).await?;

			let mut mods: Vec<ModInfo> = Vec::new();
			let mut total_downloads: u64 = 0;
			let mut total_favorites: u64 = 0;
			let mut total_views: u64 = 0;

			// go through each mod
			for publishedfiledetail in author_data.publishedfiledetails.unwrap_or_default() {
				// increment total counts
				total_downloads += publishedfiledetail.subscriptions.unwrap_or_default() as u64;
				total_favorites += publishedfiledetail.favorited.unwrap_or_default() as u64;
				total_views += publishedfiledetail.views.unwrap_or_default() as u64;

				// filter mod data and add to Vec
				mods.push(get_filtered_mod_info(&publishedfiledetail));
			}

			let author = AuthorInfo {
				steam_id: steamid,
				steam_name: steam_user.personaname,
				steam_avatar: steam_user.avatarfull,
				mods,
				total: author_data.total,
				total_downloads,
				total_favorites,
				total_views,
			};

			cache::lock_and_update(&state.author_cache, steamid, author)
		}
	};

	return cached_json!(author, 3600, false);
}

#[get("/author/<steamid>", rank=1)]
pub async fn author_1_4(steamid: u64, state: &State<Api14State>) -> Result<CacheResponse<Value>, APIError> {
	steamapi::validate_steamid64(steamid)?;
	get_author_info(steamid, state).await
}

#[get("/author/<steamname>", rank=2)]
pub async fn author_1_4_str(steamname: &str, state: &State<Api14State>) -> Result<CacheResponse<Value>, APIError> {
	let steamid = steamapi::steamname_to_steamid(steamname, &state.steam_api_key).await?;
	get_author_info(steamid, state).await
}


#[get("/get_steam_avatar?<steamids>")]
pub async fn get_steam_avatar(steamids: Vec<u64>, state: &State<Api14State>) -> Result<Value, APIError> {
	let user_infos = steamapi::get_users_info(steamids.as_slice(), &state.steam_api_key).await?;
	let s: HashMap<String, String> = user_infos.iter().map(|f| (f.steamid.clone(), f.avatarfull.clone())).collect();

	return Ok(serde_json::json!(s));
}