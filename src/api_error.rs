use rocket::response::{self, Responder, Response};
use rocket::Request;
use rocket::http::Status;
use rocket::serde::json::Json;

use rocket::serde::Serialize;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ErrorResponse {
	error: String,
	message: String,
}

pub enum APIError {
	JSONError(String),
	ReqwestError(String),
	SteamNameNotResolveable(String),
	SteamIDNotFound(u64),
	InvalidSteamID(u64),
	InvalidModName(String),
	InvalidModID(u64),
	ScrapeError(String)
}

impl std::fmt::Display for APIError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			APIError::JSONError(_) => "JSONError",
			APIError::ReqwestError(_) => "ReqwestError",
			APIError::SteamNameNotResolveable(_) => "SteamNameNotResolveable",
			APIError::SteamIDNotFound(_) => "SteamNameNotResolveable",
			APIError::InvalidSteamID(_) => "InvalidSteamID",
			APIError::InvalidModName(_) => "InvalidModName",
			APIError::InvalidModID(_) => "InvalidModID",
			APIError::ScrapeError(_) => "ScrapeError",
		})
	}
}

impl From<reqwest::Error> for APIError {
    fn from(e: reqwest::Error) -> Self {
		APIError::ReqwestError(e.to_string())
    }
}

impl From<rocket::serde::json::serde_json::Error> for APIError {
    fn from(e: rocket::serde::json::serde_json::Error) -> Self {
		APIError::JSONError(e.to_string())
    }
}

impl From<scraper::error::SelectorErrorKind<'_>> for APIError {
	fn from(e: scraper::error::SelectorErrorKind) -> Self {
		APIError::ScrapeError(e.to_string())
	}
}

impl<'r> Responder<'r, 'static> for APIError {
	fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
		let (status, message) = match &self {
			APIError::ScrapeError(msg) => (Status::InternalServerError, format!("Could not scrape html: '{}'", msg)),
			APIError::JSONError(msg) => (Status::InternalServerError, format!("Could not parse request: '{}'", msg)),
			APIError::ReqwestError(msg) => (Status::InternalServerError, format!("Could not parse json: '{}'", msg)),
			APIError::SteamNameNotResolveable(name) => (Status::BadRequest, format!("No steamid found for the specified steam name of: '{}'", name)),
			APIError::SteamIDNotFound(steamid) => (Status::BadRequest, format!("No steam user found for the specified steam id of: '{}'", steamid)),
			APIError::InvalidSteamID(steamid) => (Status::BadRequest, format!("The steamid '{}' is invalid", steamid)),
			APIError::InvalidModName(name) => (Status::BadRequest, format!("Could not find a mod with the provided name: '{}'", name)),
			APIError::InvalidModID(id) => (Status::BadRequest, format!("Could not find a mod with the id '{}'", id)),
		};

		let body = Json(ErrorResponse {
			error: format!("{}", self),
			message,
		});

		Response::build_from(body.respond_to(req)?)
			.status(status)
			.ok()
	}
}
