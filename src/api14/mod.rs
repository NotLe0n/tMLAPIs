pub mod db;

mod mod_api;
mod responses;
mod author_api;
mod history_api;
mod list_api;

use std::{sync::{Arc, Mutex}};
use responses::{AuthorInfo};
use sqlx::PgPool;
use crate::{cache::CacheMap, steamapi};
use rocket::response::content::RawHtml;

pub struct Api14State {
	pub steam_api_key: Arc<String>,
	pub db: Arc<PgPool>,
	pub author_cache: Arc<Mutex<CacheMap<u64, AuthorInfo>>>,
	pub mod_cache: Arc<Mutex<CacheMap<u64, steamapi::PublishedFileDetails>>>,
}

impl Api14State {
	pub fn init(steam_api_key: Arc<String>, db: Arc<PgPool>) -> Api14State {
		Api14State { 
			steam_api_key,
			db,
			author_cache: Arc::new(Mutex::new(CacheMap::new())),
			mod_cache: Arc::new(Mutex::new(CacheMap::new())),
		}
	}
}

#[get("/")]
fn index_1_4() -> RawHtml<&'static str> {
	RawHtml(r#"
		<h1>1.4 Index</h1>
		<a href="/1.4/count">count</a><br>
		<a href="/1.4/author">author</a><br>
		<a href="/1.4/mod">mod</a><br>
		<a href="/1.4/list">list</a><br>
		<a href="/1.4/list_authors">list_authors</a><br>
		<a href="/1.4/history">history</a><br>

		<br>
		<a href="/">go back</a><br>
	"#)
}

use mod_api::{index_mod_1_4, count_1_4, mod_1_4, mod_1_4_str};
use author_api::{index_author_1_4, author_1_4, author_1_4_str, get_steam_avatar};
use list_api::{list_1_4, list_authors};
use history_api::{index_history, index_history_mod, history_mod, history_mod_str, index_history_author, history_author, history_author_str, history_global};

pub fn get_routes() -> Vec<rocket::Route> {
	routes![
		index_1_4, 
		count_1_4, 
		index_author_1_4, author_1_4, author_1_4_str, 
		index_mod_1_4, mod_1_4, mod_1_4_str, 
		list_1_4, list_authors, 
		index_history,
		index_history_mod, history_mod, history_mod_str, 
		index_history_author, history_author, history_author_str,
		history_global,
		get_steam_avatar
	]
}