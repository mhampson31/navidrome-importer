use chrono::NaiveDateTime;
use clap::Parser;
use config::Config;
use serde::Deserialize;
use sqlx::{Connection, SqliteConnection};
use std::{env, include_str, sync::LazyLock};

#[derive(Clone, Debug, sqlx::FromRow)]
struct NavidromeData {
    path: String,
    item_id: String,
    rating: i8,
}

#[derive(Debug)]
enum Comparison {
    New,
    Conflicts,
    NoChange,
    NoTrack,
}

#[derive(Debug, Default, sqlx::FromRow)]
struct Track {
    artist: String,
    album: String,
    track: String,
    track_nbr: i32,
    rating: f32,
    play_count: i32,
    play_date: Option<String>,
    path: Option<String>,
    #[sqlx(skip)]
    navidrome_data: Option<NavidromeData>,
    #[sqlx(skip)]
    comparison: Option<Comparison>,
}

impl Track {
    async fn get_navidrome_rating(&mut self) -> Result<(), sqlx::error::Error> {
        if let Some(p) = &self.path {
            let mut path = p.trim_start_matches(&*SOURCE_LIBRARY);
            path = path.trim_start_matches("/");
            let mut conn = SqliteConnection::connect(&*NAV_DB).await.unwrap();
            println!("{:#?}", &path);

            let rating: Option<NavidromeData> =
                sqlx::query_as(include_str!("navidrome_source.sql"))
                    .bind(path)
                    .bind(&*NAV_USER)
                    .fetch_optional(&mut conn)
                    .await?;
            self.navidrome_data = rating.clone();
            if let Some(n) = rating {
                self.comparison = {
                    let source_rating = (&self.rating / 2.0).round() as i8;

                    if n.rating == source_rating {
                        Some(Comparison::NoChange)
                    } else if n.rating > 0 && source_rating > 0 {
                        Some(Comparison::Conflicts)
                    } else {
                        Some(Comparison::New)
                    }
                }
            } else {
                self.comparison = Some(Comparison::NoTrack);
            };
        };

        Ok(())
    }

    fn print(&self) {
        println!(
            "{}, {}, {}, {}, {}, {}, {:#?}, {:#?}",
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

#[derive(Debug, sqlx::Type)]
enum Source {
    Navidrome = 1,
    Plex = 2,
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

static NAV_USER: LazyLock<String> = LazyLock::new(|| {
    let settings = get_settings();
    settings.get::<String>("nav_user").unwrap()
});

static NAV_DB: LazyLock<String> = LazyLock::new(|| {
    let settings = get_settings();
    settings.get::<String>("navidrome_db").unwrap()
});

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

    ratings[0].get_navidrome_rating().await?;

    println!("{:#?}", ratings[0].print());

    Ok(())
}
