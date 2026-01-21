// define other modules
mod api_error;
mod steamapi;
mod cache;
mod api13;
mod api14;
mod db;

// import modules
use crate::api_error::APIError;

// import libraries
#[macro_use] extern crate rocket;
extern crate reqwest;
use api13::Api13State;
use api14::Api14State;
use rocket::serde::json::{Value, serde_json};
use rocket::response::content::RawHtml;
use rocket::fs::FileServer;

use chrono::Utc;
use clokwerk::{AsyncScheduler, Job, TimeUnits};
use rocket::tokio;
use std::sync::Arc;
use std::time::Duration;

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

#[get("/")]
fn index() -> RawHtml<&'static str>{
	RawHtml(r#"
		<h1>Index</h1>
		<a href="/1.3">1.3</a><br>
		<a href="/1.4">1.4</a><br>
		<a href="/img">img</a><br>
		<a href="/version">version</a>
	"#)
}

#[get("/version")]
fn version() -> Value {
	serde_json::json!({
		"version": std::env!("CARGO_PKG_VERSION")
	})
}

#[get("/")]
fn index_img() -> RawHtml<&'static str>{
	RawHtml(r#"
		<form action="javascript: window.location.href='/img/Item_' + document.getElementById('input').value + '.png'">
			<label for="input">Item ID:</label>
			<input type="number" id="input" name="quantity" min="0" max="5042">
			<input type="submit" value="Go" />
		</form>

		<a href="..">go back</a><br>
	"#)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error>{
	let steam_api_key = std::env::var("STEAM_API_KEY").expect("the 'STEAM_API_KEY' environment variable could not be read");
	let pool = Arc::new(db::create_pool().await);
	let api13_state = Api13State::init(steam_api_key.clone());
	let api14_state = Api14State::init(steam_api_key.clone(), Arc::clone(&pool));

	let key = Arc::new(steam_api_key);
	let mut scheduler = AsyncScheduler::with_tz(Utc);

	scheduler.every(1.day()).at("13:00").run(move || {
		let pool = Arc::clone(&pool);
		let steam_api_key = Arc::clone(&key);

		async move {
			if let Err(e) = db::update_mod_history(&pool, &steam_api_key).await {
				log::error!("Could not update mod history: {e}");
			}
		}
	});

	tokio::spawn(async move {
		loop {
			scheduler.run_pending().await;
			tokio::time::sleep(Duration::from_secs(60)).await;
		}
	});

	// use variable to get info like config or routes
    let _ = rocket::build().manage(api14_state).manage(api13_state)
		.mount("/", routes![index, version])
		.mount("/1.3/", api13::get_routes())
		.mount("/1.4/", api14::get_routes())
		.mount("/img/", FileServer::from("./img/"))
		.mount("/img/", routes![index_img])
		.ignite().await?
		.launch().await?; // blocking

	Ok(())
}
