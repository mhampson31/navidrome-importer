/* Only create the tables and columns that get used */

create table if not exists metadata_items (
    id integer not null primary key,
    parent_id integer,
    title varchar(255) not null,
    "index" integer,
    guid varchar(255),
    library_section_id integer not null
);

create table if not exists metadata_item_settings (
    guid varchar(255),
    account_id interger,
    rating float
);

create table if not exists accounts (
    id integer not null primary key,
    name varchar(255) not null
);

create table if not exists metadata_item_views (
    id integer not null primary key,
    guid varchar(255) not null,
    account_id integer,
    viewed_at dt_integer
);

create table if not exists media_items (
    id integer not null primary key,
    metadata_item_id integer
);

create table if not exists media_parts (
    media_item_id integer,
    file varchar(255)
);

create table if not exists section_locations (
    library_section_id integer not null,
    root_path varchar(255) not null
);

insert into metadata_items values
    ("363", NULL, "Band", "1", "plex://artist/5d07bc12403c6402904b5ef7", "3"), -- artist
    ("364", "363", "Album", "1", "plex://album/5d07ca73403c640290d32464", "3"), -- album
    ("365", "364", "Test Song", "1", "plex://track/61cb8c9f9640ca9b704e4537", "3"), -- test track 1
    ("366", "364", "Test Song 2", "2", "plex://track/61cb8c9f9640ca9b704e4542", "3"), -- test track 2
    ("367", "363", "New Album", "1", "plex://album/613adbfc22a23450583e760b", "3"), -- album 2
    ("368", "367", "New Test Song", "1", "plex://track/613adbfc22a23450583e7b7a", "3"), -- test track 1
    /* noise records that should not get counted */
    ("127", NULL, "TV Show", "1", "plex://show/5d9c08017b5c2e001e651bb9", "2"),
    ("128", "127", "", "1", "plex://season/602e5b69535238002c34a44a", "2"),
    ("129", "128", "The Test Episode", "1", "plex://episode/5d9c0cc1e9d5a1001f4ea5c0", "2")
;

insert into metadata_item_settings values
    ("com.plexapp.agents.none://ede0bf8c-daca-404e-abdf-2203d9ba679d", "1", NULL),
    ("plex://track/61cb8c9f9640ca9b704e4537", "1", "10.0"), -- test track 1
    ("plex://track/61cb8c9f9640ca9b704e4542", "1", "5.0"), -- test track 2
    ("plex://track/61cb8c9f9640ca9b704e4537", "2", "2.0"), -- test track 1, different user,
    ("plex://track/613adbfc22a23450583e7b7a", "1", "6.0"), -- test track 3
    ("plex://album/5d1a9afaf0b09c68c12c70df", "1", NULL)
;

insert into accounts values
    ("0", ""),
    ("1", "test1"),
    ("2", "test2")
;

insert into metadata_item_views values
    /* Four plays of test track 1 */
    ("1", "plex://track/61cb8c9f9640ca9b704e4537", "1", "1673051819"),
    ("2", "plex://track/61cb8c9f9640ca9b704e4537", "1", "1673619219"),
    ("3", "plex://track/61cb8c9f9640ca9b704e4537", "1", "1677176380"),
    ("4", "plex://track/61cb8c9f9640ca9b704e4537", "1", "1753720765"),
    /* plus two plays by a different user */
    ("5", "plex://track/61cb8c9f9640ca9b704e4537", "2", "1677176380"),
    ("6", "plex://track/61cb8c9f9640ca9b704e4537", "2", "1753720765")
;

insert into media_items values
    ("1", "365"),
    ("2", "366"),
    ("3", "368")
;

insert into media_parts values
    ("1", "/music/Band/Album/01 - Test Song.flac"),
    ("2", "/music/Band/Album/02 - Test Song 2.flac"),
    ("3", "/music/Band/New Album/01 - New Test Song.flac")
;

insert into section_locations values
    ("1", "/tv"),
    ("2", "/movies"),
    ("3", "/music")
;
