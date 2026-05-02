use clap::{Parser};
use sqlx::{SqliteConnection, Connection};

#[derive(Debug, sqlx::FromRow)]
struct Track {
    artist: String,
    album: String,
    track: String,
    track_nbr: i8,
    rating: f32
}


#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long)]
    source_path: String,
    navidrome_path: Option<String>,
}

#[derive(Debug)]
enum Source {
    Plex,
}


fn get_source_query(source: &Source) -> &str {
    match source {
        Source::Plex =>  r#"
        select artist.title as artist,
           album.title as album,
           track.title as track,
           track.'index' as track_nbr,
           track_data.rating as rating
    
        from metadata_items artist
    
        join metadata_items album
          on artist.id = album.parent_id
    
        join metadata_items track
          on album.id = track.parent_id
      
        join metadata_item_settings track_data
          on track_data.guid = track.guid

        where album.library_section_id = 3
          and track_data.rating is not null
      
        order by artist.title, album.title, track.'index';
        "#
    }
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    println!("Getting ratings from {:?}", cli.source_path);

    let mut conn = SqliteConnection::connect(&cli.source_path).await?;

    let source = get_source_query(&Source::Plex);

    let ratings: Vec<Track> = sqlx::query_as(&source).fetch_all(&mut conn).await?;
    println!("{:#?}", ratings.len());


    if let Some(navidrome_path) = cli.navidrome_path.as_deref() {
        println!("Navidrome db: {navidrome_path}");
    }

    Ok(())
}