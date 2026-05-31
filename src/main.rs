use anyhow;
use clap::{Parser, ValueEnum};
use config::Config;
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
    settings.get::<String>("navidrome_user").unwrap()
});

#[derive(Clone, Debug, PartialEq, sqlx::FromRow)]
struct SourceData {
    path: String,
    rating: i32,
    play_count: i32,
    play_date: String,
}

#[derive(Debug, Clone, PartialEq)]
struct UpdateDetail {
    new_rating: i32,
    new_rating_date: bool,
    new_play_count: i32,
    new_play_date: String,
}

#[derive(Debug, Default, PartialEq)]
enum Status {
    #[default]
    NotChecked,
    NoChange,
    CanUpdate,
    Updated,
    NoSourceData,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Show summary counts of the changes that will be made
    Summary,
    /// Show details about the changes that will be made
    List,
    /// Make updates in the Navidrome database
    Update,
}

#[derive(Debug, Default, PartialEq, sqlx::FromRow)]
struct Track {
    navidrome_id: String,
    artist: String,
    album: String,
    track: String,
    track_nbr: i32,
    rating: i32,
    play_count: i32,
    play_date: String,
    path: String,
    #[sqlx(skip)]
    source_data: Option<SourceData>,
    #[sqlx(skip)]
    update: Option<UpdateDetail>,
    #[sqlx(skip)]
    status: Status,
}

impl Track {
    async fn prepare_update(&mut self) -> anyhow::Result<()> {
        if let Some(u) = &self.source_data {
            /* take the higher rating */
            let new_rating = max(u.rating, self.rating);

            /* add the source's play count to Navidrome's */
            let new_play_count = &self.play_count + u.play_count;

            /* compare both systems to determine most recent date played */
            let new_play_date = max(u.play_date.clone(), self.play_date.clone());

            /* Has anything changed? If so, this track will need to be updated in Navidrome */
            if new_rating > self.rating
                || new_play_count > u.play_count
                || new_play_date.clone() > u.play_date.clone()
            {
                self.update = Some(UpdateDetail {
                    /* todo: needs logic to handle conflicts */
                    new_rating: new_rating,
                    new_rating_date: if new_rating > self.rating {
                        true
                    } else {
                        false
                    },
                    new_play_count,
                    new_play_date,
                });
                /* There's a change to make in Navidrome */
                self.status = Status::CanUpdate;
            } else {
                /* The source data has no new info */
                self.status = Status::NoChange;
            }
        } else {
            /* The source data does not have this track */
            self.status = Status::NoSourceData
        }
        Ok(())
    }

    async fn update(&mut self) -> anyhow::Result<bool> {
        let mut conn = SqliteConnection::connect(&*NAV_DB).await?;

        match &self.status {
            Status::CanUpdate => {
                let u = &self
                    .update
                    .clone()
                    .ok_or(anyhow::anyhow!("Missing update data for track"))?;

                let rows_affected = sqlx::query(include_str!("navidrome_update.sql"))
                    .bind(&*NAV_USER)
                    .bind(&self.navidrome_id.clone())
                    .bind(u.new_play_count)
                    .bind(u.new_play_date.clone())
                    .bind(u.new_rating)
                    .bind(u.new_rating_date)
                    .execute(&mut conn)
                    .await?
                    .rows_affected();

                self.status = Status::Updated;

                Ok(rows_affected > 0)
            }
            _ => Ok(false),
        }
    }
}

struct Collection {
    tracks: Vec<Track>,
}

impl Collection {
    async fn new(mut conn: SqliteConnection, user: &String) -> Collection {
        println!("Getting current tracks from Navidrome");

        Collection {
            tracks: sqlx::query_as(include_str!("navidrome_source.sql"))
                .bind(user)
                .fetch_all(&mut conn)
                .await
                .expect("Could not query the Navidrome database"),
        }
    }

    async fn prepare(&mut self, source_data: Vec<SourceData>) -> anyhow::Result<()> {
        for n in self.tracks.iter_mut() {
            let path = format!("{}/{}", &*SOURCE_LIBRARY, n.path);
            n.source_data = source_data.clone().into_iter().find(|d| d.path == path);
            n.prepare_update().await?;
        }
        Ok(())
    }

    async fn update(&mut self) -> anyhow::Result<()> {
        for t in self.tracks.iter_mut() {
            t.update().await?;
        }
        Ok(())
    }

    fn len(&self) -> usize {
        self.tracks.len()
    }
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long, value_enum)]
    mode: Option<Mode>,
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
    let cli = Cli::parse();

    /* Treat the summary view as the default mode, if nothing was specified */
    let mode = match cli.mode {
        Some(m) => m,
        None => Mode::Summary,
    };

    let nav_conn = SqliteConnection::connect(&*NAV_DB)
        .await
        .expect("Could not connect to the Navidrome database");

    let mut nav_data = Collection::new(nav_conn, &*NAV_USER).await;
    println!("Found {:#?} tracks", nav_data.len());

    println!("Checking import source...");

    let mut conn = SqliteConnection::connect(&*SOURCE_DB).await.unwrap();

    let source_data: Vec<SourceData> = sqlx::query_as(include_str!("plex_source.sql"))
        .bind(&*SOURCE_LIBRARY)
        .bind(&*SOURCE_USER)
        .fetch_all(&mut conn)
        .await?;
    println!("Found {:#?} tracks", source_data.len());

    nav_data.prepare(source_data).await?;

    match mode {
        Mode::Summary => {
            println!("Summary");
            println!(
                "{:#?} tracks can be updated",
                nav_data
                    .tracks
                    .into_iter()
                    .filter(|t| t.status == Status::CanUpdate)
                    .count()
            );
        }
        Mode::List => {
            println!("List mode");
        }
        Mode::Update => {
            println!("Performing update...");
            nav_data.update().await?
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::migrate::Migrator;
    use std::path::Path;

    async fn create_nav_db() -> Result<SqliteConnection, sqlx::Error> {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await?;
        Migrator::new(Path::new("./nav-migrations"))
            .await?
            .run(&mut conn)
            .await?;
        Ok(conn)
    }

    async fn create_plex_db() -> Result<SqliteConnection, sqlx::Error> {
        println!("connect");
        let mut conn = SqliteConnection::connect("sqlite::memory:").await?;
        println!("migrate");
        Migrator::new(Path::new("./plex-migrations"))
            .await?
            .run(&mut conn)
            .await?;
        Ok(conn)
    }

    fn get_nav_user() -> String {
        String::from("kiNuIyhPxNjUKmhxY9DXty")
    }

    fn get_plex_user() -> String {
        String::from("test1")
    }

    #[sqlx::test]
    async fn create_collection() -> Result<(), sqlx::Error> {
        let conn = create_nav_db().await?;
        let user = get_nav_user();

        let collection = Collection::new(conn, &user).await;
        assert_eq!(3, collection.len());

        Ok(())
    }

    #[sqlx::test]
    async fn track_count() -> Result<(), sqlx::Error> {
        let mut conn = create_nav_db().await?;
        let user = get_nav_user();
        let nav_data: Vec<Track> = sqlx::query_as(include_str!("navidrome_source.sql"))
            .bind(&user)
            .fetch_all(&mut conn)
            .await
            .expect("Could not query Navidrome db");

        assert_eq!(3, nav_data.len());
        Ok(())
    }

    #[sqlx::test]
    async fn plex_count() -> Result<(), sqlx::Error> {
        let mut plex_conn = create_plex_db().await?;
        println!("plex db");
        let plex_user = get_plex_user();

        let plex_data: Vec<SourceData> = sqlx::query_as(include_str!("plex_source.sql"))
            .bind("/music")
            .bind(plex_user)
            .fetch_all(&mut plex_conn)
            .await?;

        assert_eq!(3, plex_data.len());

        Ok(())
    }

    #[sqlx::test]
    async fn prepare_update() -> Result<(), anyhow::Error> {
        let nav_conn = create_nav_db().await?;
        let nav_user = get_nav_user();

        let mut collection = Collection::new(nav_conn, &nav_user).await;

        let mut plex_conn = create_plex_db().await?;
        let plex_user = get_plex_user();

        let plex_data: Vec<SourceData> = sqlx::query_as(include_str!("plex_source.sql"))
            .bind("/music")
            .bind(plex_user)
            .fetch_all(&mut plex_conn)
            .await?;

        collection.prepare(plex_data).await?;

        assert_eq!(
            collection.tracks[0].update,
            Some(UpdateDetail {
                new_rating: 5,
                new_rating_date: false,
                new_play_count: 7,
                new_play_date: String::from("2026-04-23 02:02:39.804+00:00"),
            })
        );

        /*
            test user has three tracks by Band in Nav:
                Album:
                    1. Test Song - played 3 times, last played 4/23/2026, rated 5
                        has one annotation of "fake_type" that shouldn't count
                        has one annotation from a different user, shouldn't count
                    2. Test Song 2 - played 27 times, last play 1/13/2026, rated 10
                New Album:
                    1. New Test Song - never listed

            Tests:
                1. Test Song gets a new rating, +views, new date
                2. Test Song 2 has the same rating, +views, no new date
                3. New Test Song gets a new annotation record
        */

        Ok(())
    }

    #[sqlx::test]
    async fn do_update() -> Result<(), anyhow::Error> {
        let nav_conn = create_nav_db().await?;
        let nav_user = get_nav_user();

        let mut collection = Collection::new(nav_conn, &nav_user).await;

        let mut plex_conn = create_plex_db().await?;
        let plex_user = get_plex_user();

        let plex_data: Vec<SourceData> = sqlx::query_as(include_str!("plex_source.sql"))
            .bind("/music")
            .bind(plex_user)
            .fetch_all(&mut plex_conn)
            .await?;

        collection.prepare(plex_data).await?;

        collection.update().await?;

        /* query the test db for the new data */
        let updated_collection = Collection::new(create_nav_db().await?, &nav_user).await;

        let t = Track {
            navidrome_id: String::from("TkclRuUT3Ju381lf2Utlmd"),
            artist: String::from("Band"),
            album: String::from("Album"),
            track: String::from("Test Song"),
            track_nbr: 1,
            rating: 5,
            play_count: 3,
            play_date: String::from("2026-04-23 02:02:39.804+00:00"),
            path: String::from("Band/Album/01 - Test Song.flac"),
            source_data: None,
            update: None,
            status: Status::NotChecked,
        };
        assert_eq!(t, updated_collection.tracks[0]);

        Ok(())
    }
}
