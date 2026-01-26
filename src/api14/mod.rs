mod api;
pub mod db;
mod responses;

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

#[get("/mod")]
fn index_mod_1_4() -> RawHtml<&'static str> {
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

#[get("/author")]
fn index_author_1_4() -> RawHtml<&'static str> {
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

#[get("/history")]
fn index_history() -> RawHtml<&'static str> {
	RawHtml(r#"
		<h1>1.4/History</h1>
		<a href="/1.4/history/mod">mod history</a><br>
		<a href="/1.4/history/global">global history</a><br>
		<a href="/1.4/history/author">author history</a><br>

		<br>
		<a href="/1.4">go back</a><br>
	"#)
}

#[get("/history/mod")]
fn index_history_mod() -> RawHtml<&'static str> {
	RawHtml(r#"
		<form action="javascript: window.location.href += '/' + document.getElementById('input').value">
			<h1>Mod History (<a href="https://github.com/NotLe0n/tMLAPIs/wiki/1.4-mod-history">Docs</a>)</h1> 

			<label for="input">Mod ID or name:</label>
			<input type="text" id="input">
			<input type="submit" value="Go" />
		</form>

		<a href="/1.4/history">go back</a>
	"#)
}

#[get("/history/author")]
fn index_history_author() -> RawHtml<&'static str> {
	RawHtml(r#"
		<h1>Author History (<a href="https://github.com/NotLe0n/tMLAPIs/wiki/1.4-author-history">Docs</a>)</h1> 

		<form action="javascript: window.location.href += '/' + document.getElementById('input').value">
			<label for="input">SteamID64 or vanity name:</label>
			<input type="text" id="input">
			<input type="submit" value="Go" />
		</form>

		<a href="/1.4/history">go back</a>
	"#)
}
use api::*;

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
		history_global
	]
}