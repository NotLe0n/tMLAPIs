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

#[derive(Debug)]
pub enum APIError {
	JSONError(String),
	ReqwestError(String),
	SteamNameNotResolveable(String),
	SteamIDNotFound(u64),
	InvalidSteamID(u64),
	InvalidModName(String),
	InvalidModID(u64),
	ScrapeError(String),
	DBError(String)
}

impl APIError {
	fn get_identifier(&self) -> &str {
		return match self {
			APIError::JSONError(_) => "JSONError",
			APIError::ReqwestError(_) => "ReqwestError",
			APIError::SteamNameNotResolveable(_) => "SteamNameNotResolveable",
			APIError::SteamIDNotFound(_) => "SteamNameNotResolveable",
			APIError::InvalidSteamID(_) => "InvalidSteamID",
			APIError::InvalidModName(_) => "InvalidModName",
			APIError::InvalidModID(_) => "InvalidModID",
			APIError::ScrapeError(_) => "ScrapeError",
			APIError::DBError(_) => "DBError"
		};
	}

	fn get_message(&self) -> String {
		return match self {
			APIError::ScrapeError(msg) => format!("Could not scrape html: '{}'", msg),
			APIError::JSONError(msg) => format!("Could not parse request: '{}'", msg),
			APIError::ReqwestError(msg) => format!("Could not parse json: '{}'", msg),
			APIError::SteamNameNotResolveable(name) => format!("No steamid found for the specified steam name of: '{}'", name),
			APIError::SteamIDNotFound(steamid) => format!("No steam user found for the specified steam id of: '{}'", steamid),
			APIError::InvalidSteamID(steamid) => format!("The steamid '{}' is invalid", steamid),
			APIError::InvalidModName(name) => format!("Could not find a mod with the provided name: '{}'", name),
			APIError::InvalidModID(id) => format!("Could not find a mod with the id '{}'", id),
			APIError::DBError(msg) =>  format!("An Error occured accessing the Database: '{msg}'"),
		}
	}

	fn get_status(&self) -> Status {
		return match self {
			APIError::ScrapeError(_) => Status::InternalServerError,
			APIError::JSONError(_) => Status::InternalServerError,
			APIError::ReqwestError(_) => Status::InternalServerError,
			APIError::SteamNameNotResolveable(_) => Status::BadRequest,
			APIError::SteamIDNotFound(_) => Status::BadRequest,
			APIError::InvalidSteamID(_) => Status::BadRequest,
			APIError::InvalidModName(_) => Status::BadRequest,
			APIError::InvalidModID(_) => Status::BadRequest,
			APIError::DBError(_) => Status::InternalServerError,
		}
	}
}

impl From<reqwest::Error> for APIError {
    fn from(e: reqwest::Error) -> Self {
		log::warn!("{}", e.to_string());
		APIError::ReqwestError(e.without_url().to_string())
    }
}

impl From<rocket::serde::json::serde_json::Error> for APIError {
    fn from(e: rocket::serde::json::serde_json::Error) -> Self {
		log::warn!("{}", e.to_string());
		APIError::JSONError(e.to_string())
    }
}

impl From<scraper::error::SelectorErrorKind<'_>> for APIError {
	fn from(e: scraper::error::SelectorErrorKind) -> Self {
		log::warn!("{}", e.to_string());
		APIError::ScrapeError(e.to_string())
	}
}

impl From<sqlx::Error> for APIError {
	fn from(e: sqlx::Error) -> Self {
		log::warn!("{}", e.to_string());
		APIError::DBError(e.to_string())
	}
}

impl<'r> Responder<'r, 'static> for APIError {
	fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
		let body = Json(ErrorResponse {
			error: format!("{}", self.get_identifier()),
			message: self.get_message(),
		});

		Response::build_from(body.respond_to(req)?)
			.status(self.get_status())
			.ok()
	}
}
