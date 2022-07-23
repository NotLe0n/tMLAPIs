// define other modules
pub mod api14;
pub mod api13;
pub mod api_error;
pub mod steamapi;

// import modules
use crate::api13::*;
use crate::api14::*;
use crate::api_error::APIError;

// import libraries
#[macro_use] extern crate rocket;
extern crate reqwest;
use rocket::serde::json::{Value, serde_json};
use rocket::response::content::RawHtml;
use once_cell::sync::OnceCell;

// Holds the SteamAPI Key
static INSTANCE: OnceCell<String> = OnceCell::new();

// does a get reqwests on the specified URL and Returns a Json<String> if successfull or a Status if it errored 
async fn get_json(url: String) -> Result<Value, APIError> {
	let res = reqwest::get(url).await?;
    let body = res.text().await?;

	Ok(serde_json::from_str(&body)?)
}

#[get("/")]
async fn index() -> RawHtml<&'static str>{
	RawHtml("
		<a href=\"/1.3\">1.3</a><br>
		<a href=\"/1.4\">1.4</a>
	")
}

#[get("/")]
async fn index_1_3() -> RawHtml<&'static str> {
	RawHtml("
		<a href=\"1.3/count\">count</a><br>
		<a href=\"1.3/author\">author</a><br>
		<a href=\"1.3/mod\">mod</a><br>
		<a href=\"1.3/list\">list</a><br>
	")
}

#[get("/")]
async fn index_1_4() -> RawHtml<&'static str> {
	RawHtml("
		<a href=\"1.4/count\">count</a><br>
		<a href=\"1.4/author\">author</a><br>
		<a href=\"1.4/mod\">mod</a><br>
		<a href=\"1.4/list\">list</a><br>
	")
}

// This is where the API starts
#[launch]
fn rocket() -> _ {
	INSTANCE.set(std::env::var("STEAM_API_KEY").expect("the 'STEAM_API_KEY' environment variable could not be read")).expect("OnceCEll couldn't be set");

    rocket::build()
		.mount("/", routes![index])
		.mount("/1.3/", routes![index_1_3, count_1_3, author_1_3, author_1_3_str, mod_1_3])
		.mount("/1.4/", routes![index_1_4, count_1_4, author_1_4, author_1_4_str, mod_1_4])
}