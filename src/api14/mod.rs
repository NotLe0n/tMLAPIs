pub mod api;
mod responses;

use std::sync::{Mutex, Arc};
use responses::{AuthorInfo, ModInfo};
use crate::{cache::{CacheMap, CacheItem}, steamapi};
use rocket::response::content::RawHtml;

pub struct Api14State {
	pub steam_api_key: String,
	pub author_cache: Arc<Mutex<CacheMap<u64, AuthorInfo>>>,
    pub mod_cache: Arc<Mutex<CacheMap<u64, steamapi::PublishedFileDetails>>>,
	pub mod_list_cache: Arc<Mutex<CacheItem<Vec<ModInfo>>>>
}

impl Api14State {
    pub fn init(steam_api_key: String) -> Api14State {
        Api14State { 
			steam_api_key,
			author_cache: Arc::new(Mutex::new(CacheMap::new())),
            mod_cache: Arc::new(Mutex::new(CacheMap::new())),
			mod_list_cache: Arc::new(Mutex::new(CacheItem::new()))
        }
    }
}

#[get("/")]
fn index_1_4() -> RawHtml<&'static str> {
	RawHtml(r#"
        <h1>1.4 Index</h1>
		<a href="1.4/count">count</a><br>
		<a href="1.4/author">author</a><br>
		<a href="1.4/mod">mod</a><br>
		<a href="1.4/list">list</a><br>
		<br>
		<a href="..">go back</a><br>
	"#)
}

#[get("/mod")]
fn index_mod_1_4() -> RawHtml<&'static str> {
	RawHtml(r#"
		<form action="javascript: window.location.href += '/' + document.getElementById('input').value">
            <h1>Mod info (<a href="https://github.com/NotLe0n/tMLAPIs/wiki/1.4-mod">Docs</a>)</h1> 

			<label for="input">Mod ID or name:</label>
			<input type="text" id="input">
			<input type="submit" value="Go" />
		</form>

		<a href="..">go back</a>
	"#)
}

#[get("/author")]
fn index_author_1_4() -> RawHtml<&'static str> {
	RawHtml(r#"
		<h1>Author info (<a href="https://github.com/NotLe0n/tMLAPIs/wiki/1.4-author">Docs</a>)</h1> 

		<form action="javascript: window.location.href += '/' + document.getElementById('input').value">
			<label for="input">SteamID64 or vanity name:</label>
			<input type="text" id="input">
			<input type="submit" value="Go" />
		</form>

		<a href="..">go back</a>
	"#)
}

use api::*;

pub fn get_routes() -> Vec<rocket::Route> {
    routes![index_1_4, count_1_4, index_author_1_4, author_1_4, author_1_4_str, index_mod_1_4, mod_1_4, mod_1_4_str, list_1_4]
}