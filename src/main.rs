use clap::Parser;
use config::Config;
use serde::Deserialize;
use sqlx::{Connection, SqliteConnection};
use std::env;

#[derive(Debug, sqlx::FromRow)]
struct Track {
    artist: String,
    album: String,
    track: String,
    track_nbr: i8,
    rating: f32,
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long)]
    config: Option<String>,
}

#[derive(Debug)]
enum Source {
    Plex,
}

fn get_source_query(source: &Source) -> &str {
    match source {
        Source::Plex => {
            r#"
            select artist.title as artist,
               album.title as album,
               track.title as track,
               track."index" as track_nbr,
               s.rating as rating,
               part.file
   
            from metadata_items artist

            join metadata_items album
              on artist.id = album.parent_id
  
            join metadata_items track
              on album.id = track.parent_id
  
            join metadata_item_settings s
              on s.guid = track.guid

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
                /* this should be parameterized */
                where sl.root_path in ("/music")
            )
              and s.rating is not null
  
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

    println!("Getting ratings from {:?}", &source);

    let mut conn = SqliteConnection::connect(&source).await?;

    let source = get_source_query(&Source::Plex);

    let ratings: Vec<Track> = sqlx::query_as(&source).fetch_all(&mut conn).await?;
    println!("{:#?}", ratings.len());

    conn.close().await?;

    Ok(())
}
