extern crate reqwest;

use std::collections::{HashMap};

use rocket::State;
use rocket::serde::json::serde_json::{self, Value};
use rocket_cache_response::CacheResponse;
use sqlx::{PgPool, Postgres, Transaction};
use crate::api14::db::ModsRow;
use crate::{APIError, cache, cached_json, steamapi};
use super::{responses::*, Api14State};

#[get("/count")]
pub async fn count_1_4(state: &State<Api14State>) -> Result<Value, APIError> {
	let count = steamapi::get_mod_count(&state.steam_api_key).await?;
	Ok(serde_json::json!(count))
}

#[get("/author/<steamid>", rank=1)]
pub async fn author_1_4(steamid: u64, state: &State<Api14State>) -> Result<CacheResponse<Value>, APIError> {
	steamapi::validate_steamid64(steamid)?;
	get_author_info(steamid, state).await
}

#[get("/author/<steamname>", rank=2)]
pub async fn author_1_4_str(steamname: &str, state: &State<Api14State>) -> Result<CacheResponse<Value>, APIError> {
	let steamid = steamapi::steamname_to_steamid(steamname, &state.steam_api_key).await?;
	get_author_info(steamid, state).await
}

async fn get_author_info(steamid: u64, state: &State<Api14State>) -> Result<CacheResponse<Value>, APIError> {
	let author = match cache::lock_and_get(&state.author_cache, steamid, 3600) {
		Some(cached_value) => cached_value,
		None => {
			let steam_user = steamapi::get_user_info(steamid, &state.steam_api_key).await?;
			let author_data = steamapi::get_user_mods(steamid, &state.steam_api_key).await?;

			let mut mods: Vec<ModInfo> = Vec::new();
			let mut total_downloads: u64 = 0;
			let mut total_favorites: u64 = 0;
			let mut total_views: u64 = 0;

			// go through each mod
			for publishedfiledetail in author_data.publishedfiledetails.unwrap_or_default() {
				// increment total counts
				total_downloads += publishedfiledetail.subscriptions.unwrap_or_default() as u64;
				total_favorites += publishedfiledetail.favorited.unwrap_or_default() as u64;
				total_views += publishedfiledetail.views.unwrap_or_default() as u64;

				// filter mod data and add to Vec
				mods.push(get_filtered_mod_info(&publishedfiledetail));
			}

			let author = AuthorInfo {
				steam_id: steamid,
				steam_name: steam_user.personaname,
				mods,
				total: author_data.total,
				total_downloads,
				total_favorites,
				total_views,
			};

			cache::lock_and_update(&state.author_cache, steamid, author)
		}
	};

	return cached_json!(author, 3600, false);
}

fn get_filtered_mod_info(publishedfiledetail: &steamapi::PublishedFileDetails) -> ModInfo {
	let publishedfiledetail = publishedfiledetail.clone();
	if publishedfiledetail.result.unwrap_or_default() > 1 {
		log::warn!("Unexpected multiple result at mod {}", publishedfiledetail.publishedfileid.clone().unwrap_or_default())
	}

	// get data from kvtags (tml specific data) field
	let mut internal_name = String::new();
	let mut author = String::new();
	let mut modside = String::new();
	let mut homepage = String::new();
	let mut deprecated_version_mod = String::new();
	let mut deprecated_version_tmodloader = String::new();
	let mut version_summary = String::new();
	let mut mod_references = String::new();
	let mut youtube: Option<String> = None;
	let mut twitter: Option<String> = None;
	let mut reddit: Option<String> = None;
	let mut facebook: Option<String> = None;
	let mut sketchfab: Option<String> = None;

	if let Some(kvtags) = publishedfiledetail.kvtags {
		// `into_iter()` moves the KVTag, so we can move its `String`s without cloning
		for steamapi::KVTag { key, value } in kvtags.into_iter() {
			match key.as_str() {
				"name"              => internal_name = value,
				"Author"            => author = value,
				"modside"           => modside = value,
				"homepage"          => homepage = value,
				"version"           => deprecated_version_mod = value,
				"modloaderversion"  => deprecated_version_tmodloader = value,
				"versionsummary"    => version_summary = value,
				"modreferences"     => mod_references = value,
				"youtube"           => youtube = (!value.is_empty()).then_some(value),
				"twitter"           => twitter = (!value.is_empty()).then_some(value),
				"reddit"            => reddit = (!value.is_empty()).then_some(value),
				"facebook"          => facebook = (!value.is_empty()).then_some(value),
				"sketchfab" 		=> sketchfab = (!value.is_empty()).then_some(value),
				tag => log::warn!("missing KV Tag: {tag}")
			}
		}
	}

	// the kvTags 'version' and 'modloaderversion' are deprecated
	let versions = if version_summary.is_empty() {
		vec![ModVersion {
			mod_version: deprecated_version_mod,
			tmodloader_version: deprecated_version_tmodloader,
		}]
	} else {
		version_summary.split(';').map(|version| {
			let mut c = version.splitn(2, ':');
			ModVersion {
				mod_version: c.next().unwrap().to_string(),
				tmodloader_version: c.next().unwrap().to_string()
			}
		}).collect()
	};

	let socials: Option<ModSocials> = 
		if youtube == None && twitter == None && reddit == None && facebook == None && sketchfab == None {
			None
		} else { 
			Some(ModSocials { 
				youtube,
				twitter,
				reddit,
				facebook,
				sketchfab,
			})
		 };

	let children = publishedfiledetail.children.map(|children| 
		children.iter()
			.filter_map(|c| c.publishedfileid.parse().ok())
			.collect()
		);

	// construct ModInfo struct
	return ModInfo{
		display_name: publishedfiledetail.title.unwrap_or_default(),
		internal_name,
		mod_id: publishedfiledetail.publishedfileid.unwrap_or_default().parse().unwrap_or_default(),
		author,
		author_id: publishedfiledetail.creator.unwrap_or_default().parse().unwrap_or_default(),
		modside,
		homepage,
		versions,
		mod_references,
		num_versions: publishedfiledetail.revision_change_number.unwrap_or_default().parse().unwrap_or_default(),
		tags: publishedfiledetail.tags,
		time_created: publishedfiledetail.time_created.unwrap_or_default(),
		time_updated: publishedfiledetail.time_updated.unwrap_or_default(),
		workshop_icon_url: publishedfiledetail.preview_url.unwrap_or_default(),
		children: children,
		description: publishedfiledetail.file_description,
		downloads_total: publishedfiledetail.subscriptions.unwrap_or_default(),
		favorited: publishedfiledetail.favorited.unwrap_or_default(),
		views: publishedfiledetail.views.unwrap_or_default(),
		playtime: publishedfiledetail.lifetime_playtime.unwrap_or_default(),
		followers: publishedfiledetail.followers.unwrap_or_default(),
		vote_data: publishedfiledetail.vote_data,
		num_comments: publishedfiledetail.num_comments_public.unwrap_or_default(),
		socials,
	}
	
}


#[get("/mod/<modid>", rank=1)]
pub async fn mod_1_4(modid: u64, state: &State<Api14State>) -> Result<CacheResponse<Value>, APIError> {
	let mod_data = get_mod_data(modid, state).await?;

	let filtered_data = get_filtered_mod_info(&mod_data);
	return cached_json!(filtered_data, 3600, false);
}

#[get("/mod/<modname>", rank=2)]
pub async fn mod_1_4_str(modname: &str, state: &State<Api14State>) -> Result<CacheResponse<Value>, APIError> {
	let mod_id = steamapi::modname_to_modid(modname, &state.steam_api_key).await?;
	let mod_data = get_mod_data(mod_id, state).await?;

	let filtered_data = get_filtered_mod_info(&mod_data);
	return cached_json!(filtered_data, 3600, false);
}

async fn get_mod_data(modid: u64, state: &State<Api14State>) -> Result<steamapi::PublishedFileDetails, APIError> {
	return match cache::lock_and_get(&state.mod_cache, modid, 3600) {
		Some(cached_value) => Ok(cached_value),
		None => {
			let details = steamapi::get_mod_info(modid, &state.steam_api_key).await?;

			// update cache value
			Ok(cache::lock_and_update(&state.mod_cache, modid, details))
		}
	}
}

pub async fn get_filtered_mod_list(steam_api_key: &str) -> Result<Vec<ModInfo>, APIError> {
	let client = reqwest::Client::new();

	let mut mods: Vec<ModInfo> = Vec::new();
	let mut next_cursor = String::from("*");
	loop {
		let list = steamapi::get_mod_list(&client, &next_cursor, steam_api_key).await?;
		if list.total == 0 || list.publishedfiledetails.is_none() {
			break;
		}

		let details = &list.publishedfiledetails.unwrap();

		// add filtered mod info to vec
		mods.extend(details.iter().map(get_filtered_mod_info));

		next_cursor = list.next_cursor.unwrap();
	}

	Ok(mods)
}

#[get("/list")]
pub async fn list_1_4(state: &State<Api14State>) -> Result<Value, APIError> {
	let db: &PgPool = &state.db;
	
	let mut tx: Transaction<Postgres> = db.begin().await?;

	// get all mods
	let rows: Vec<ModsRow> = sqlx::query_as!(ModsRow,
		r#"
		SELECT * FROM mods 
		LEFT JOIN mod_socials USING (mod_id)
		"#
	).fetch_all(&mut *tx).await?;

	let mod_ids: Vec<i64> = rows.iter().map(|r| r.mod_id).collect();

	// map mod ids to mod list of versions
	let mut versions_map: HashMap<i64, Vec<ModVersion>> = sqlx::query!(
		r#"
		SELECT mod_id, mod_version, tmodloader_version
		FROM mod_versions
		WHERE mod_id = ANY($1)
		ORDER BY mod_version
		"#,
		&mod_ids
	)
	.fetch_all(&mut *tx)
	.await?
	.into_iter()
	.fold(HashMap::new(), |mut acc, row| {
		acc.entry(row.mod_id)
			.or_default()
			.push(ModVersion {
				mod_version: row.mod_version,
				tmodloader_version: row.tmodloader_version,
			});
		acc
	});

	// mod mod ids to list of tags
	let mut tags_map: HashMap<i64, Vec<steamapi::ModTag>> = sqlx::query!(
		r#"
		SELECT mod_id, tag, display_name
		FROM mod_tags
		WHERE mod_id = ANY($1)
		"#,
		&mod_ids
	)
	.fetch_all(&mut *tx)
	.await?
	.into_iter()
	.fold(HashMap::new(), |mut acc, row| {
		acc.entry(row.mod_id)
			.or_default()
			.push(steamapi::ModTag {
				tag: row.tag,
				display_name: row.display_name,
			});
		acc
	});

	// map mod ids to list of children
	let mut children_map: HashMap<i64, Vec<u64>> = sqlx::query!(
		r#"
		SELECT parent_mod_id, child_mod_id
		FROM mod_children
		WHERE parent_mod_id = ANY($1)
		"#,
		&mod_ids
	)
	.fetch_all(&mut *tx).await?
	.into_iter()
	.fold(HashMap::new(), |mut acc, row| {
		acc.entry(row.parent_mod_id)
			.or_default()
			.push(row.child_mod_id as u64);
		acc
	});

	let mut mods = Vec::with_capacity(rows.len());

	for row in rows {	
		mods.push(ModInfo {
			display_name: row.display_name,
			internal_name: row.internal_name,
			mod_id: row.mod_id as u64,
			author: row.author,
			author_id: row.author_id as u64,
			modside: row.modside,
			homepage: row.homepage,
			versions: versions_map.remove(&row.mod_id).unwrap_or_default(),
			tags: tags_map.remove(&row.mod_id),
			children: children_map.remove(&row.mod_id),
			socials: [&row.youtube, &row.twitter, &row.reddit, &row.facebook, &row.sketchfab]
				.iter().any(|f| f.as_deref().is_some()).then_some(ModSocials {
				youtube: row.youtube,
				twitter: row.twitter,
				reddit: row.reddit,
				facebook: row.facebook,
				sketchfab: row.sketchfab
			}),
			mod_references: row.mod_references,
			num_versions: row.num_versions as u32,
			time_created: row.time_created as u64,
			time_updated: row.time_updated as u64,
			workshop_icon_url: row.workshop_icon_url,
			description: row.description,
			downloads_total: row.downloads_total as u32,
			favorited: row.favorited as u32,
			followers: row.followers as u32,
			views: row.views as u64,
			playtime: row.playtime,
			num_comments: row.num_comments as u32,

			vote_data: Some(crate::steamapi::VoteData {
				score: row.score,
				votes_up: row.votes_up as u32,
				votes_down: row.votes_down as u32,
			})
		})
	}

	tx.commit().await?;

	Ok(serde_json::json!(mods))
}

#[get("/list_authors")]
pub async fn list_authors(state: &State<Api14State>) -> Result<Value, APIError> {
	let db: &PgPool = &state.db;

	let rows = sqlx::query!(
		r#"
		SELECT
			json_build_object(
				'author_id', author_id,
				'author_names', array_agg(DISTINCT author),
				'mods', json_agg(
					json_build_object(
						'mod_id', mod_id,
						'display_name', display_name,
						'internal_name', internal_name
					)
					ORDER BY display_name
				),
				'total_downloads', SUM(downloads_total)::BIGINT,
				'total_views', SUM(views)::BIGINT,
				'total_favorited', SUM(favorited)::BIGINT
			) AS result
		FROM mods
		GROUP BY author_id
		ORDER BY SUM(downloads_total) DESC
		"#
	)
	.fetch_all(db)
	.await?;

	Ok(Value::Array(
		rows.into_iter().filter_map(|r| r.result).collect()
	))
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
		"#
	)
	.fetch_one(db)
	.await?;

	return Ok(row.history.unwrap_or(Value::Array(vec![])));
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