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

#[derive(Debug)]
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
}

impl Track {
    async fn prepare_update(&mut self) -> Result<(), sqlx::error::Error> {
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

                let update: Update = Update {
                    new_rating: source_rating,
                    new_play_count: &self.play_count + n.play_count,
                    new_play_date: max(n.play_date, self.play_date.clone()),
                };
                println!("New data: {:#?}", update);
            } else {
                println!("No data found for {:#?} and {:#?}", path, &*NAV_USER);
            }
        };

        Ok(())
    }

    fn print(&self) {
        println!(
            "{}, {}, {}, {}, {}, plays {}, date {:#?}, nav {:#?}",
            &self.artist,
            &self.album,
            &self.track,
            &self.track_nbr,
            &self.rating,
            &self.play_count,
            &self.play_date,
            &self.navidrome_data
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

    let r = 2;

    ratings[r].prepare_update().await?;

    ratings[r].print();

    Ok(())
}
