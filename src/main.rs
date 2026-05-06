use clap::Parser;
use config::Config;
use serde::Deserialize;
use sqlx::{Connection, SqliteConnection};
use std::env;

#[derive(Debug, sqlx::Decode, sqlx::FromRow)]
struct NavidromeData {
    path: String,
    rating: i8,
}

#[derive(Debug, sqlx::Decode, sqlx::FromRow)]
struct Track {
    source: Source,
    artist: String,
    album: String,
    track: String,
    track_nbr: i8,
    rating: f32,
    path: Option<String>,
    navidrome_data: Option<NavidromeData>,
}

impl Track {
    async fn get_navidrome_rating(&self) -> Option<Track> {
        let source = get_source_query(&Source::Navidrome);

        let mut home = env::home_dir().unwrap();
        home.push(".navidrome-importer");
        home.push("settings.toml");

        let settings = Config::builder()
            .add_source(config::File::with_name(home.to_str().unwrap()))
            .build()
            .unwrap();

        let library = settings.get::<String>("library").unwrap();
        let nav_user = settings.get::<String>("nav_user").unwrap();
        let nav_db = settings.get::<String>("navidrome").unwrap();

        let m = match &self.path {
            None => None,
            Some(p) => {
                let mut path = p.trim_start_matches(&library);
                path = path.trim_start_matches("/");
                let mut conn = SqliteConnection::connect(&nav_db).await.unwrap();
                println!("{:#?}", &path);
                let rating: Option<NavidromeData> = sqlx::query_as(source)
                    .bind(Source::Plex)
                    .bind(path)
                    .bind(nav_user)
                    .fetch_optional(&mut conn)
                    .await
                    .unwrap();
                rating
            }
        };

        m
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

fn get_source_query(source: &Source) -> &str {
    match source {
        Source::Navidrome => {
            r#"
            select
               	track.path as path,
                annotation.rating as rating

            from media_file track

            join annotation
              on track.id = annotation.item_id

            where track.path = $2
              and annotation.user_id = (
                  select user_id from user u where u.user_name = $3
              );

            "#
        }

        Source::Plex => {
            r#"
            select $1 as source,
               artist.title as artist,
               album.title as album,
               track.title as track,
               track."index" as track_nbr,
               part.file
               settings.rating as rating,

            from metadata_items artist

            join metadata_items album
              on artist.id = album.parent_id

            join metadata_items track
              on album.id = track.parent_id

            join metadata_item_settings settings
              on settings.guid = track.guid

             /* We don't need anything from media_items.
                It just lets us link metadata_items to media_parts
              */
            join media_items media
              on track.id = media.metadata_item_id

            join media_parts part
              on part.media_item_id = media.id

            where track.library_section_id in (
                select sl.library_section_id
                from section_locations sl
                where sl.root_path = $2
            )
              and settings.rating is not null

            order by artist.title, album.title, track."index";
        "#
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let mut home = env::home_dir().unwrap();
    home.push(".navidrome-importer");
    home.push("settings.toml");

    let settings = Config::builder()
        .add_source(config::File::with_name(home.to_str().unwrap()))
        .build()
        .unwrap();

    let source = settings.get::<String>("source").unwrap();
    let library = settings.get::<String>("library").unwrap();

    println!("Getting ratings from {:?}", &source);

    let mut conn = SqliteConnection::connect(&source).await?;

    let source = get_source_query(&Source::Plex);

    let ratings: Vec<Track> = sqlx::query_as(source)
        .bind(Source::Plex)
        .bind(&library)
        .fetch_all(&mut conn)
        .await
        .expect("Could not query Plex db");
    println!("{:#?}", ratings.len());

    conn.close().await?;

    println!("{:#?}", ratings[0].get_navidrome_rating().await);

    Ok(())
}
