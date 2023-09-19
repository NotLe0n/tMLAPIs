// define other modules
mod api_error;
mod steamapi;
mod cache;
mod api13;
mod api14;

// import modules
use crate::api13::api::*;
use crate::api14::api::*;
use crate::api_error::APIError;

// import libraries
#[macro_use] extern crate rocket;
extern crate reqwest;
use api13::Api13State;
use api14::Api14State;
use rocket::serde::json::{Value, serde_json};
use rocket::response::content::RawHtml;
use rocket::fs::FileServer;

#[macro_export]
macro_rules! cached_json {
    ($json:tt, $max_age:literal, $revalidate:literal) => {
		Ok(CacheResponse::Public {
			responder: serde_json::json!($json),
			max_age: $max_age, // cached for seconds
			must_revalidate: $revalidate,
		})
	};
}

// does a get reqwests on the specified URL and Returns a Json<String> if successful or a Status if it errored
async fn get_json(url: &str) -> Result<Value, APIError> {
	let res = reqwest::get(url).await?;
    let body = res.text().await?;

	Ok(serde_json::from_str(&body)?)
}

#[get("/")]
fn index() -> RawHtml<&'static str>{
	RawHtml("
		<a href=\"/1.3\">1.3</a><br>
		<a href=\"/1.4\">1.4</a><br>
		<a href=\"/img\">img</a><br>
		<a href=\"/version\">version</a>
	")
}

#[get("/")]
fn index_1_3() -> RawHtml<&'static str> {
	RawHtml("
		<a href=\"1.3/count\">count</a><br>
		<a href=\"1.3/author\">author</a><br>
		<a href=\"1.3/mod\">mod</a><br>
		<a href=\"1.3/list\">list</a><br>
		<a href=\"1.3/history\">history</a><br>
	")
}

#[get("/")]
fn index_1_4() -> RawHtml<&'static str> {
	RawHtml("
		<a href=\"1.4/count\">count</a><br>
		<a href=\"1.4/author\">author</a><br>
		<a href=\"1.4/mod\">mod</a><br>
		<a href=\"1.4/list\">list</a><br>
	")
}

#[get("/")]
fn index_img() -> RawHtml<&'static str>{
	RawHtml("
		<form action=\"javascript: window.location.href='/img/Item_' + document.getElementById('input').value + '.png'\">
			<input type=\"number\" id=\"input\" name=\"quantity\" min=\"0\" max=\"5042\">
			<input type=\"submit\" value=\"Go\" />
		</form>
	")
}

#[get("/mod")]
fn index_mod() -> RawHtml<&'static str> {
	just_input()
}


#[get("/author")]
fn index_author() -> RawHtml<&'static str> {
	just_input()
}

#[get("/history")]
fn index_history_1_3() -> RawHtml<&'static str> {
	just_input()
}

fn just_input() -> RawHtml<&'static str> {
	RawHtml("
		<form action=\"javascript: window.location.href += '/' + document.getElementById('input').value\">
			<input type=\"text\" id=\"input\">
			<input type=\"submit\" value=\"Go\" />
		</form>
	")
}

#[get("/version")]
fn version() -> Value {
	serde_json::json!({
		"version": std::env!("CARGO_PKG_VERSION")
	})
}

// This is where the API starts
#[launch]
fn rocket() -> _ {
	let steam_api_key = std::env::var("STEAM_API_KEY").expect("the 'STEAM_API_KEY' environment variable could not be read");
	let api13_state = Api13State::init(steam_api_key.clone());
	let api14_state = Api14State::init(steam_api_key.clone());
	
    rocket::build().manage(api14_state).manage(api13_state)
		.mount("/", routes![index, version])
		.mount("/1.3/", routes![index_1_3, count_1_3, index_author, author_1_3, author_1_3_str, index_mod, mod_1_3, list_1_3, index_history_1_3, history_1_3])
		.mount("/1.4/", routes![index_1_4, count_1_4, index_author, author_1_4, author_1_4_str, index_mod, mod_1_4, mod_1_4_str, list_1_4])
		.mount("/img/", FileServer::from("./img/"))
		.mount("/img/", routes![index_img])
}