use rocket::serde::json::Json;

#[derive(Responder, Debug)]
pub enum APIError {
	#[response(status = 500, content_type = "json")]
	JSONError(Json<String>),
	#[response(status = 500, content_type = "json")]
	ReqwestError(Json<String>),
	#[response(status = 400, content_type = "json")]
	SteamIDNotFound(Json<String>),
	#[response(status = 400, content_type = "json")]
	InvalidSteamID(Json<String>),
	#[response(status = 400, content_type = "json")]
	InvalidModName(Json<String>)
}

impl From<reqwest::Error> for APIError {
    fn from(e: reqwest::Error) -> Self {
		APIError::ReqwestError(Json(format!("could not parse request: {}", e.to_string())))
    }
}

impl From<rocket::serde::json::serde_json::Error> for APIError {
    fn from(e: rocket::serde::json::serde_json::Error) -> Self {
		APIError::JSONError(Json(format!("could not parse json: {}", e.to_string())))
    }
}