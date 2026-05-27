/* Only create the tables and columns that get used */

create table if not exists media_file (
    id varchar(255) not null primary key,
    path varchar(255) not null,
    title varchar(255) not null,
    album varchar(255) not null,
    artist varchar(255) not null,
    track_number integer not null
);

create table if not exists annotation (
    user_id varchar(255) not null,
    item_id varchar(255) not null,
	item_type varchar(255) not null,
	play_count integer default 0,
	play_date datetime,
	rating integer default 0,
	primary key(user_id, item_id, item_type)
);

insert into media_file values
("TkclRuUT3Ju381lf2Utlmd", "Band/Album/01 - Test Song.flac", "Test Song", "Album", "Band", 1),
("HSSNWzhymYyyqDEmFkK8I9", "Band/Album/02 - Test Song 2.flac", "Test Song 2", "Album", "Band", 2),
("MCupEUwRJeG1836F3wEz5G", "Band/New Album/01 - New Test Song.flac", "New Test Song", "New Album", "Band", 1);

insert into annotation values
("kiNuIyhPxNjUKmhxY9DXty", "TkclRuUT3Ju381lf2Utlmd", "media_file", 3, "2026-04-23 02:02:39.804+00:00", 5),
("kiNuIyhPxNjUKmhxY9DXty", "TkclRuUT3Ju381lf2Utlmd", "fake_type", 2, "2026-04-27 02:04:32.804+00:00", 2),
("kiNuIyhPxNjUKmhxY9DXty", "HSSNWzhymYyyqDEmFkK8I9", "media_file", 27, "2026-01-13 02:02:39.804+00:00", 10),
("ddV5zjYx4IZJEj2QCaKXdw", "TkclRuUT3Ju381lf2Utlmd", "media_file", 7, "2026-04-24 18:38:47.945+00:00", 0);

/*
    test user has three tracks by Band:
        Album:
            1. Test Song - played 3 times, last played 4/23/2026, rated 5
                has one annotation of "fake_type" that shouldn't count
                has one annotation from a different user, shouldn't count
            2. Test Song 2 - played 27 times, last play 1/13/2026, rated 10
        New Album:
            1. New Test Song - never listed
*/
