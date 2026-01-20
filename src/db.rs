use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use crate::{api_error::APIError, api14::api::get_filtered_mod_list};
use chrono::Utc;

pub async fn create_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database")
}

pub async fn update_mod_history(db: &PgPool, steam_api_key: &str) -> Result<(), APIError> {
    let today = Utc::now().date_naive();
    log::info!("Updating Mod History for {}", today.to_string());

    let mods = get_filtered_mod_list(steam_api_key).await?;

    let capacity = mods.len();
    let mut mod_ids = Vec::with_capacity(capacity);
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

    for m in &mods {
        mod_ids.push(m.mod_id.clone());
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
            $1::text[], 
            $2::date[], 
            $3::int[], 
            $4::bigint[], 
            $5::int[],
            $6::int[],
            $7::int[],
            $8::int[],
            $9::int[],
            $10::int[],
            $11::bigint[],
            $12::bigint[],
            $13::text[]
        )
        "#,
        &mod_ids,
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