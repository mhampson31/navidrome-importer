# Navidrome Importer

This is a command-line program to help move metadata like ratings and play counts from your previous media system into your Navidrome installation.

## Description

Navidrome Importer helps you migrate your music library to Navidrome from an older media system by collecting your existing metadata and updating the Navidrome database directly.

Currently, it will import your existing ratings, play counts, and most recent play dates for each track in your source system, and attempt to integrate that with any data that already exists in your Navidrome system. 
- Ratings: Takes the higher of the two ratings -- this behavior will probably become customizable at some point.
- Play count: Takes the sum of the two play counts. If you listened to a track 10 times in your old system and 5 times in Navidrome, its play count will be updated to 15.
- Play date: Takes the most recent play date between the two systems.

Tracks are matched between systems based on their file paths. The assumption here is that you have a single collection of media files and you're just changing your software.

This project isn't affiliated with Navidrome, Plex, or any other organizations. This is just a small little app I'm putting together in my free time for my own use.

### Source Systems

Currently, Navidrome Importer can only import data from Plex.

## Warning

This software is _highly experimental_, and you use at your own risk. I can't guarantee it'll work as expected. Make a backup copy of your Navidrome database first!

## Getting Started

### Dependencies

* You must have access to a Linux terminal where you can download and run this program.
* The source database and Navidrome database must both be local and accessible on the same filesystem.
* Your Navidrome database must be writeable.
 * You can copy both database files into a local directory, make your updates, and copy the Navidrome file back.
* The application does not need root access.

### Installing

* Download the latest release
* Create a text file in the location ~/.navidrome-importer/settings.toml and configure the following settings:
```markdown 
source_db = "/absolute/path/to/source.db"
source_user = "source_user_id"
source_library = "/music"
navidrome_db = "/absolute/path/to/navidrome.db"
navidrome_user = "navidrome_user_id"
```

The database files must be the full paths the the Sqlite3 DB files, not relative to where you are running the importer.

`source_user`: the account ID that owns the metadata you're importing. 
  - Plex: Should be the username you sign in with. Or, log in and navigate to your account page, and it should be visible in the URL: `www.yourplex.domain/web/index.html#!/u/<user ID>`

`source_library`: the root folder of your music collection

`navidrome_user`: the Navidrome account you want to add the metadata to.
  - Log into Navidrome, navigate to the Users page, and click the user you want. The ID will be a long random string like "kiNuIyhPxNjUKmhxY9DXty", visible in the URL: `www.yournavidrome.domain/app/#/user/<user ID>`


### Executing program

* How to run the program
* Step-by-step bullets

## Help

Any advise for common problems or issues.

## AI Disclaimer

This application is human-generated. No AI was used for any purpose in this project.
