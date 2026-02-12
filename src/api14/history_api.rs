use rocket::State;
use rocket::response::content::RawHtml;
use rocket::serde::json::serde_json::Value;
use sqlx::PgPool;
use crate::api_error::APIError;
use crate::steamapi;

use super::Api14State;


#[get("/history")]
pub fn index_history() -> RawHtml<&'static str> {
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
pub fn index_history_mod() -> RawHtml<&'static str> {
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
pub fn index_history_author() -> RawHtml<&'static str> {
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

async fn get_mod_history(modid: u64, db: &PgPool) -> Result<Value, APIError> {
	let row = sqlx::query!(
		r#"
		SELECT json_agg(
			json_build_object(
				'date', date,
				'mod_id', mod_id,
				'author_id', author_id,
				'downloads_total', downloads_total,
				'views', views,
				'followers', followers,
				'favorited', favorited,
				'vote_data', json_build_object(
					'votes_up', votes_up,
					'votes_down', votes_down,
					'score', score
				),
				'num_comments', num_comments,
				'playtime', playtime,
				'time_updated', time_updated,
				'version', version
			)
			ORDER BY date DESC
		) AS "history: Value"
		FROM mod_history
		WHERE mod_id = $1
		"#,
		modid as i64
	)
	.fetch_one(db)
	.await?;

	return Ok(row.history.unwrap_or(Value::Array(vec![])));
}

#[get("/history/mod/<modid>", rank=1)]
pub async fn history_mod(modid: u64, state: &State<Api14State>) -> Result<Value, APIError> {
	get_mod_history(modid, &state.db).await
}

#[get("/history/mod/<modname>", rank=2)]
pub async fn history_mod_str(modname: &str, state: &State<Api14State>) -> Result<Value, APIError> {
	let mod_id = steamapi::modname_to_modid(modname, &state.steam_api_key).await?;
	return get_mod_history(mod_id, &state.db).await;
}

#[get("/history/global")]
pub async fn history_global(state: &State<Api14State>) -> Result<Value, APIError> {
	let db: &PgPool = &state.db;
	let row = sqlx::query!(
		r#"
		SELECT 
			json_build_object('date', date,
				'downloads_total', SUM(downloads_total),
				'views_total', SUM(views),
				'followers_total', SUM(followers),
				'favorited_total', SUM(favorited),
				'playtime_total', SUM(playtime),
				'comments_total', SUM(num_comments) 
			) AS "history: Value" 
		from mod_history GROUP BY date
		ORDER BY date DESC
		"#
	)
	.fetch_all(db)
	.await?;

	return Ok(Value::Array(
		row.into_iter().filter_map(|f| f.history).collect()
	));
}

async fn get_author_history(steamid: u64, db: &PgPool) -> Result<Value, APIError> {
	let row = sqlx::query!(
		r#"
		SELECT json_agg(
			json_build_object(
				'date', date,
				'mod_id', mod_id,
				'author_id', author_id,
				'downloads_total', downloads_total,
				'views', views,
				'followers', followers,
				'favorited', favorited,
				'vote_data', json_build_object(
					'votes_up', votes_up,
					'votes_down', votes_down,
					'score', score
				),
				'num_comments', num_comments,
				'playtime', playtime,
				'time_updated', time_updated,
				'version', version
			)
			ORDER BY date DESC
		) AS "history: Value"
		FROM mod_history
		WHERE author_id = $1
		"#,
		steamid as i64
	)
	.fetch_one(db)
	.await?;

	return Ok(row.history.unwrap_or(Value::Array(vec![])));
}

#[get("/history/author/<steamid>", rank=1)]
pub async fn history_author(steamid: u64, state: &State<Api14State>) -> Result<Value, APIError> {
	steamapi::validate_steamid64(steamid)?;
	return get_author_history(steamid, &state.db).await;
}

#[get("/history/author/<steamname>", rank=2)]
pub async fn history_author_str(steamname: &str, state: &State<Api14State>) -> Result<Value, APIError> {
	let steamid = steamapi::steamname_to_steamid(steamname, &state.steam_api_key).await?;
	return get_author_history(steamid, &state.db).await;
}
