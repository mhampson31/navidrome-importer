use clap::{Parser};


struct Track {
    artist: String,
    album: String,
    number: i8,
    rating: i8
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    nav_path: Option<String>,
    source_path: Option<String>
}


fn main() {
    let cli = Cli::parse();

    if let Some(nav_db) = cli.nav_db.as_deref() {
        println!("Navidrome db: {nav_db}");
    }
}