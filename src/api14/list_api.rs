extern crate reqwest;

use std::collections::{HashMap};

use rocket::State;
use rocket::serde::json::serde_json::{self, Value};
use sqlx::{PgPool, Postgres, Transaction};
use crate::api14::db::ModsRow;
use crate::{APIError, steamapi};
use super::{responses::*, Api14State};


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
