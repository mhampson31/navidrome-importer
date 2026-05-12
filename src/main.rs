use anyhow;
use chrono::NaiveDateTime;
use clap::Parser;
use config::Config;
use serde::Deserialize;
use sqlx::{Connection, SqliteConnection};
use std::{cmp::max, env, include_str, sync::LazyLock};

static SOURCE_LIBRARY: LazyLock<String> = LazyLock::new(|| {
    let settings = get_settings();
    settings.get::<String>("source_library").unwrap()
});

static SOURCE_USER: LazyLock<String> = LazyLock::new(|| {
    let settings = get_settings();
    settings.get::<String>("source_user").unwrap()
});

static SOURCE_DB: LazyLock<String> = LazyLock::new(|| {
    let settings = get_settings();
    settings.get::<String>("source_db").unwrap()
});

static NAV_DB: LazyLock<String> = LazyLock::new(|| {
    let settings = get_settings();
    settings.get::<String>("navidrome_db").unwrap()
});

static NAV_USER: LazyLock<String> = LazyLock::new(|| {
    let settings = get_settings();
    settings.get::<String>("nav_user").unwrap()
});

#[derive(Clone, Debug, sqlx::FromRow)]
struct NavidromeData {
    path: String,
    item_id: String,
    rating: i32,
    play_count: i32,
    play_date: String,
}

#[derive(Debug, Clone)]
struct Update {
    new_rating: i32,
    new_play_count: i32,
    new_play_date: String,
}

#[derive(Debug, Default, sqlx::FromRow)]
struct Track {
    artist: String,
    album: String,
    track: String,
    track_nbr: i32,
    rating: f32,
    play_count: i32,
    play_date: String,
    path: Option<String>,
    #[sqlx(skip)]
    navidrome_data: Option<NavidromeData>,
    #[sqlx(skip)]
    update: Option<Update>,
    #[sqlx(skip)]
    status: Status,
}

#[derive(Debug, Default)]
enum Status {
    #[default]
    NotChecked,
    NoChange,
    CanUpdate,
    Updated,
    MissingNavData,
}

impl Track {
    async fn prepare_update(&mut self) -> anyhow::Result<()> {
        if let Some(p) = &self.path {
            let mut path = p.trim_start_matches(&*SOURCE_LIBRARY);
            path = path.trim_start_matches("/");
            let mut conn = SqliteConnection::connect(&*NAV_DB).await.unwrap();

            let nav_data: Option<NavidromeData> =
                sqlx::query_as(include_str!("navidrome_source.sql"))
                    .bind(&*NAV_USER)
                    .bind(path)
                    .fetch_optional(&mut conn)
                    .await?;

            self.navidrome_data = nav_data.clone();

            if let Some(n) = nav_data {
                let source_rating = (&self.rating / 2.0).round() as i32;

                /* add the source's play count to Navidrome's */
                let new_play_count = &self.play_count + n.play_count;

                /* compare both systems to determine most recent date played */
                let new_play_date = max(n.play_date.clone(), self.play_date.clone());

                /* Has anything changed? If so, this track will need to be updated in Navidrome */
                if source_rating > n.rating
                    || new_play_count > n.play_count
                    || new_play_date.clone() > n.play_date.clone()
                {
                    self.status = Status::CanUpdate;
                } else {
                    self.status = Status::NoChange;
                }

                let update: Update = Update {
                    /* todo: needs logic to handle conflicts */
                    new_rating: source_rating,
                    new_play_count,
                    new_play_date,
                };

                self.update = Some(update.clone());

                println!("New data: {:#?}", update);
            } else {
                println!("No data found for {:#?} and {:#?}", path, &*NAV_USER);
            }
        };

        Ok(())
    }

    async fn do_update(&mut self) -> anyhow::Result<bool> {
        let mut conn = SqliteConnection::connect(&*NAV_DB).await?;

        match &self.status {
            Status::CanUpdate => {
                let u = &self
                    .update
                    .clone()
                    .ok_or(anyhow::anyhow!("Missing update data for track"))?;
                let n = &self
                    .navidrome_data
                    .clone()
                    .ok_or(anyhow::anyhow!("Missing Navidrome data for track"))?;

                let new_rating_date = if u.new_rating > n.rating { true } else { false };

                let rows_affected = sqlx::query(include_str!("navidrome_update.sql"))
                    .bind(&*NAV_USER)
                    .bind(n.item_id.clone())
                    .bind(u.new_play_count)
                    .bind(u.new_play_date.clone())
                    .bind(u.new_rating)
                    .bind(new_rating_date)
                    .execute(&mut conn)
                    .await?
                    .rows_affected();

                self.status = Status::Updated;

                Ok(rows_affected > 0)
            }
            Status::MissingNavData => {
                /* TODO: What do we do here? */
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn print(&self) {
        println!(
            "{}, {}, {}, {}, {}, plays {}, date {:#?}, nav {:#?}, status {:#?}",
            &self.artist,
            &self.album,
            &self.track,
            &self.track_nbr,
            &self.rating,
            &self.play_count,
            &self.play_date,
            &self.navidrome_data,
            &self.status
        );
    }
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long)]
    config: Option<String>,
    /*
     * preview
     * new only
     * update all
     * show overwrites
     */
}

fn get_settings() -> Config {
    let mut home = env::home_dir().unwrap();
    home.push(".navidrome-importer");
    home.push("settings.toml");

    Config::builder()
        .add_source(config::File::with_name(home.to_str().unwrap()))
        .build()
        .unwrap()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    println!("Getting ratings from {:?}", &*SOURCE_DB);

    let mut conn = SqliteConnection::connect(&*SOURCE_DB).await?;

    let mut ratings: Vec<Track> = sqlx::query_as(include_str!("plex_source.sql"))
        .bind(&*SOURCE_LIBRARY)
        .bind(&*SOURCE_USER)
        .fetch_all(&mut conn)
        .await
        .expect("Could not query Plex db");
    println!("{:#?}", ratings.len());

    conn.close().await?;

    let r = 1;

    ratings[r].prepare_update().await?;
    ratings[r].do_update().await?;

    ratings[r].print();

    Ok(())
}
