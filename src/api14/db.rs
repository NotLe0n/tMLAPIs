use crate::{
	api_error::APIError,
	api14::{api, responses::ModInfo},
};
use chrono::{Timelike, Utc};
use rocket::serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction, postgres::PgPoolOptions};
use std::time::Duration;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(crate = "rocket::serde")]
pub struct ModsRow {
	pub mod_id: i64,
	pub display_name: String,
	pub internal_name: String,
	pub author: String,
	pub author_id: i64,
	pub modside: String,
	pub homepage: String,
	pub mod_references: String,
	pub num_versions: i32,
	pub time_created: i64,
	pub time_updated: i64,
	pub workshop_icon_url: String,
	pub description: Option<String>,
	pub downloads_total: i32,
	pub favorited: i32,
	pub followers: i32,
	pub views: i64,
	pub playtime: String,
	pub num_comments: i32,
	pub score: f64,
	pub votes_up: i32,
	pub votes_down: i32,
	pub youtube: Option<String>,
	pub twitter: Option<String>,
	pub reddit: Option<String>,
	pub facebook: Option<String>,
	pub sketchfab: Option<String>
}

pub async fn create_pool() -> PgPool {
	PgPoolOptions::new()
		.max_connections(10)
		.acquire_timeout(Duration::from_secs(5))
		.connect(&std::env::var("DATABASE_URL").unwrap())
		.await
		.expect("Failed to connect to database")
}

pub async fn update_db(db: &PgPool, steam_api_key: &str) -> Result<(), APIError> {
	let mods = api::get_filtered_mod_list(steam_api_key).await?;

	update_mod_list(&mods, db).await?;
	if Utc::now().hour() < 22 {
		update_mod_history(&mods, db).await?;
	}
	Ok(())
}

pub async fn update_mod_history(mods: &Vec<ModInfo>, db: &PgPool) -> Result<(), APIError> {
	if mods.is_empty() {
		return Err(APIError::DBError("Mod List was empty".to_string()));
	}

	let today = Utc::now().date_naive();
	log::info!("Updating Mod History for {}", today.to_string());

	let capacity = mods.len();
	let mut mod_ids = Vec::with_capacity(capacity);
	let mut author_ids = Vec::with_capacity(capacity);
	let mut downloads = Vec::with_capacity(capacity);
	let mut views = Vec::with_capacity(capacity);
	let mut followers = Vec::with_capacity(capacity);
	let mut favorited = Vec::with_capacity(capacity);
	let mut num_comments = Vec::with_capacity(capacity);
	let mut time_updated = Vec::with_capacity(capacity);
	let mut playtime = Vec::with_capacity(capacity);
	let mut versions = Vec::with_capacity(capacity);
	let mut votes_up = Vec::with_capacity(capacity);
	let mut votes_down = Vec::with_capacity(capacity);
	let mut score = Vec::with_capacity(capacity);

	for m in mods {
		mod_ids.push(m.mod_id as i64);
		author_ids.push(m.author_id as i64);
		downloads.push(m.downloads_total as i32);
		views.push(m.views as i64);
		followers.push(m.followers as i32);
		favorited.push(m.favorited as i32);
		num_comments.push(m.num_comments as i32);
		time_updated.push(m.time_updated as i64);
		playtime.push(str::parse::<i64>(&m.playtime).unwrap_or(0));
		versions.push(m.versions.last().map(|v| v.mod_version.clone()));
		votes_up.push(m.vote_data.as_ref().map(|v| v.votes_up as i32));
		votes_down.push(m.vote_data.as_ref().map(|v| v.votes_down as i32));
		score.push(m.vote_data.as_ref().map(|v| v.score));
	}

	let dates: Vec<_> = std::iter::repeat(today).take(capacity).collect();

	sqlx::query!(
		r#"
		INSERT INTO mod_history (
			mod_id, 
			author_id,
			date, 
			downloads_total,
			views,
			followers,
			favorited,
			votes_up,
			votes_down,
			score,
			num_comments,
			playtime,
			time_updated,
			version
		)
		SELECT * FROM UNNEST(
			$1::bigint[], 
			$2::bigint[],
			$3::date[], 
			$4::int[], 
			$5::bigint[], 
			$6::int[],
			$7::int[],
			$8::int[],
			$9::int[],
			$10::float8[],
			$11::int[],
			$12::bigint[],
			$13::bigint[],
			$14::text[]
		)
		"#,
		&mod_ids,
		&author_ids,
		&dates,
		&downloads,
		&views,
		&followers,
		&favorited,
		&votes_up as &[Option<i32>],
		&votes_down as &[Option<i32>],
		&score as &[Option<f64>],
		&num_comments,
		&playtime,
		&time_updated,
		&versions as &[Option<String>]
	)
	.execute(db)
	.await?;

	Ok(())
}

pub async fn update_mod_list(mods: &Vec<ModInfo>, db: &PgPool) -> Result<(), APIError> {
	log::info!("Updating Mod List");

	if mods.is_empty() {
		return Err(APIError::DBError("Mod List was empty".to_string()));
	}

	let len = mods.len();

	// prepare buffers for unnest
	let mut ids = Vec::with_capacity(len);
	let mut display_names = Vec::with_capacity(len);
	let mut internal_names = Vec::with_capacity(len);
	let mut authors = Vec::with_capacity(len);
	let mut author_ids = Vec::with_capacity(len);
	let mut modsides = Vec::with_capacity(len);
	let mut homepages = Vec::with_capacity(len);
	let mut mod_refs = Vec::with_capacity(len);
	let mut num_versions = Vec::with_capacity(len);
	let mut time_created = Vec::with_capacity(len);
	let mut time_updated = Vec::with_capacity(len);
	let mut icons = Vec::with_capacity(len);
	let mut descriptions = Vec::with_capacity(len);
	let mut downloads = Vec::with_capacity(len);
	let mut favorited = Vec::with_capacity(len);
	let mut followers = Vec::with_capacity(len);
	let mut views = Vec::with_capacity(len);
	let mut playtimes = Vec::with_capacity(len);
	let mut comments = Vec::with_capacity(len);
	let mut votes_up = Vec::with_capacity(len);
	let mut votes_down = Vec::with_capacity(len);
	let mut score = Vec::with_capacity(len);

	let mut v_mod_ids = Vec::new();
	let mut versions = Vec::new();
	let mut tml_versions = Vec::new();

	let mut t_mod_ids = Vec::new();
	let mut t_tags = Vec::new();
	let mut t_tag_names = Vec::new();

	let mut parent_ids = Vec::new();
	let mut child_ids = Vec::new();

	let mut s_mod_ids = Vec::new();
	let mut youtube_links = Vec::new();
	let mut reddit_links = Vec::new();
	let mut twitter_links = Vec::new();
	let mut facebook_links = Vec::new();
	let mut sketchfab_links = Vec::new();

	for m in mods.into_iter() {
		ids.push(m.mod_id as i64);
		display_names.push(m.display_name.clone());
		internal_names.push(m.internal_name.clone());
		authors.push(m.author.clone());
		author_ids.push(m.author_id as i64);
		modsides.push(m.modside.clone());
		homepages.push(m.homepage.clone());
		mod_refs.push(m.mod_references.clone());
		num_versions.push(m.num_versions as i32);
		time_created.push(m.time_created as i64);
		time_updated.push(m.time_updated as i64);
		icons.push(m.workshop_icon_url.clone());
		descriptions.push(m.description.clone());
		downloads.push(m.downloads_total as i32);
		favorited.push(m.favorited as i32);
		followers.push(m.followers as i32);
		views.push(m.views as i64);
		playtimes.push(m.playtime.clone());
		comments.push(m.num_comments as i32);
		votes_up.push(m.vote_data.as_ref().map(|v| v.votes_up).unwrap_or_default() as i32);
		votes_down.push(
			m.vote_data
				.as_ref()
				.map(|v| v.votes_down)
				.unwrap_or_default() as i32,
		);
		score.push(m.vote_data.as_ref().map(|v| v.score).unwrap_or_default());

		for v in &m.versions {
			v_mod_ids.push(m.mod_id as i64);
			versions.push(v.mod_version.clone());
			tml_versions.push(v.tmodloader_version.clone());
		}

		if let Some(tags) = &m.tags {
			for t in tags {
				t_mod_ids.push(m.mod_id as i64);
				t_tags.push(t.tag.clone());
				t_tag_names.push(t.display_name.clone());
			}
		}

		if let Some(children) = &m.children {
			for child in children {
				parent_ids.push(m.mod_id as i64);
				child_ids.push(*child as i64);
			}
		}

		if let Some(socials) = &m.socials {
			s_mod_ids.push(m.mod_id as i64);
			youtube_links.push(socials.youtube.clone());
			reddit_links.push(socials.reddit.clone());
			twitter_links.push(socials.twitter.clone());
			facebook_links.push(socials.facebook.clone());
			sketchfab_links.push(socials.sketchfab.clone());
		}
	}

	let mut tx: Transaction<Postgres> = db.begin().await?;

	// delete old data
	sqlx::query!("TRUNCATE mods CASCADE")
		.execute(&mut *tx)
		.await?;

	// insert into mods table
	sqlx::query!(
		r#"
		INSERT INTO mods (
			mod_id, display_name, internal_name, author, author_id,
			modside, homepage, mod_references, num_versions,
			time_created, time_updated, workshop_icon_url, description,
			downloads_total, favorited, followers, views,
			playtime, num_comments, votes_up, votes_down, score
		)
		SELECT *
		FROM UNNEST(
			$1::BIGINT[], $2::TEXT[], $3::TEXT[], $4::TEXT[], $5::BIGINT[],
			$6::TEXT[], $7::TEXT[], $8::TEXT[], $9::INT[],
			$10::BIGINT[], $11::BIGINT[], $12::TEXT[], $13::TEXT[],
			$14::INT[], $15::INT[], $16::INT[], $17::BIGINT[],
			$18::TEXT[], $19::INT[], $20::int[], $21::int[], $22::float8[]
		)
		"#,
		&ids,
		&display_names,
		&internal_names,
		&authors,
		&author_ids,
		&modsides,
		&homepages,
		&mod_refs,
		&num_versions,
		&time_created,
		&time_updated,
		&icons,
		&descriptions as &[Option<String>],
		&downloads,
		&favorited,
		&followers,
		&views,
		&playtimes,
		&comments,
		&votes_up,
		&votes_down,
		&score
	)
	.execute(&mut *tx)
	.await?;

	if !s_mod_ids.is_empty() {
		sqlx::query!(
			r#"
			INSERT INTO mod_socials (mod_id, youtube, twitter, reddit, facebook, sketchfab)
			SELECT *
			FROM UNNEST(
				$1::BIGINT[], $2::TEXT[], $3::TEXT[], $4::TEXT[], $5::TEXT[], $6::TEXT[]
			)
			"#,
			&s_mod_ids,
			&youtube_links as &[Option<String>],
			&twitter_links as &[Option<String>],
			&reddit_links as &[Option<String>],
			&facebook_links as &[Option<String>],
			&sketchfab_links as &[Option<String>]
		)
		.execute(&mut *tx)
		.await?;
	}

	// insert into mod_versions table
	if !v_mod_ids.is_empty() {
		sqlx::query!(
			r#"
			INSERT INTO mod_versions (mod_id, mod_version, tmodloader_version)
			SELECT *
			FROM UNNEST(
				$1::BIGINT[], $2::TEXT[], $3::TEXT[]
			)
			"#,
			&v_mod_ids,
			&versions,
			&tml_versions
		)
		.execute(&mut *tx)
		.await?;
	}

	// insert into mod_tags table
	if !t_mod_ids.is_empty() {
		sqlx::query!(
			r#"
			INSERT INTO mod_tags (mod_id, tag, display_name)
			SELECT *
			FROM UNNEST(
				$1::BIGINT[], $2::TEXT[], $3::TEXT[]
			)
			"#,
			&t_mod_ids,
			&t_tags,
			&t_tag_names
		)
		.execute(&mut *tx)
		.await?;
	}

	// insert into mod_children table
	if !parent_ids.is_empty() {
		sqlx::query!(
			r#"
			INSERT INTO mod_children (parent_mod_id, child_mod_id)
			SELECT c.parent_mod_id, c.child_mod_id
			FROM UNNEST(
				$1::BIGINT[],
				$2::BIGINT[]
			) AS c(parent_mod_id, child_mod_id)
			JOIN mods p ON p.mod_id = c.parent_mod_id
			JOIN mods ch ON ch.mod_id = c.child_mod_id;
			"#,
			&parent_ids,
			&child_ids
		)
		.execute(&mut *tx)
		.await?;
	}

	tx.commit().await?;

	Ok(())
}
