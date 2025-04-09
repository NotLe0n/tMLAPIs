#[derive(Responder, Debug)]
pub enum APIError {
	#[response(status = 500, content_type = "json")]
	JSONError(String),
	#[response(status = 500, content_type = "json")]
	ReqwestError(String),
	#[response(status = 400, content_type = "json")]
	SteamIDNotFound(String),
	#[response(status = 400, content_type = "json")]
	InvalidSteamID(String),
	#[response(status = 400, content_type = "json")]
	InvalidModName(String),
	#[response(status = 400, content_type = "json")]
	InvalidModID(String),
	#[response(status = 500, content_type = "json")]
	ScrapeError(String)
}

impl From<reqwest::Error> for APIError {
    fn from(e: reqwest::Error) -> Self {
		APIError::ReqwestError(format!("could not parse request: {}", e.to_string()))
    }
}

impl From<rocket::serde::json::serde_json::Error> for APIError {
    fn from(e: rocket::serde::json::serde_json::Error) -> Self {
		APIError::JSONError(format!("could not parse json: {}", e.to_string()))
    }
}

impl From<scraper::error::SelectorErrorKind<'_>> for APIError {
	fn from(e: scraper::error::SelectorErrorKind) -> Self {
		APIError::ScrapeError(format!("could not scrape html: {}", e.to_string()))
	}
}
