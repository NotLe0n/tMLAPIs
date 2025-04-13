pub mod api;
mod responses;

use std::sync::{Mutex, Arc};
use responses::{AuthorInfo, ModInfo, ModListInfo};
use crate::cache::{CacheMap, CacheItem};
use rocket::response::content::RawHtml;

pub struct Api13State {
	pub steam_api_key: String,
	pub author_cache: Arc<Mutex<CacheMap<u64, AuthorInfo>>>,
    pub mod_cache: Arc<Mutex<CacheMap<String, ModInfo>>>,
	pub mod_list_cache: Arc<Mutex<CacheItem<Vec<ModListInfo>>>>
}

impl Api13State {
    pub fn init(steam_api_key: String) -> Api13State {
        Api13State {
			steam_api_key,
			author_cache: Arc::new(Mutex::new(CacheMap::new())),
            mod_cache: Arc::new(Mutex::new(CacheMap::new())),
			mod_list_cache: Arc::new(Mutex::new(CacheItem::new()))
        }
    }
}

#[get("/")]
fn index_1_3() -> RawHtml<&'static str> {
	RawHtml(r#"
        <h1>1.3 Index (<a href="..">Go Back</a>)</h1>
		<a href="1.3/count">count</a><br>
		<a href="1.3/author">author</a><br>
		<a href="1.3/mod">mod</a><br>
		<a href="1.3/list">list</a><br>
		<a href="1.3/history">history</a><br>
	"#)
}

#[get("/mod")]
fn index_mod_1_3() -> RawHtml<&'static str> {
	RawHtml(r#"
		<form action="javascript: window.location.href += '/' + document.getElementById('input').value">
            <h1>Mod info (<a href="https://github.com/NotLe0n/tMLAPIs/wiki/1.3-mod">Docs</a>)</h1> 

			<label for="input">Mod name:</label>
			<input type="text" id="input">
			<input type="submit" value="Go" />
		</form>
	"#)
}

#[get("/author")]
fn index_author_1_3() -> RawHtml<&'static str> {
	RawHtml(r#"
		<h1>Author info (<a href="https://github.com/NotLe0n/tMLAPIs/wiki/1.3-author">Docs</a>)</h1> 

		<form action="javascript: window.location.href += '/' + document.getElementById('input').value">
			<label for="input">SteamID64 or vanity name:</label>
			<input type="text" id="input">
			<input type="submit" value="Go" />
		</form>
	"#)
}

#[get("/history")]
fn index_history_1_3() -> RawHtml<&'static str> {
	RawHtml(r#"
		<h1>1.3 mod version history (<a href="https://github.com/NotLe0n/tMLAPIs/wiki/1.3-history">Docs</a>)</h1> 
		
		<form action="javascript: window.location.href += '/' + document.getElementById('input').value">
			<label for="input">mod name:</label>
			<input type="text" id="input">
			<input type="submit" value="Go" />
		</form>
	"#)
}

use api::*;

pub fn get_routes() -> Vec<rocket::Route> {
    routes![index_1_3, count_1_3, index_author_1_3, author_1_3, author_1_3_str, index_mod_1_3, mod_1_3, list_1_3, index_history_1_3, history_1_3]
}