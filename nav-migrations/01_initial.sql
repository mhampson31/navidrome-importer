/* Only create the tables and columns that get used */

create table if not exists media_file (
    id varchar(255) not null primary key,
    path varchar(255) default '' not null,
    title varchar(255) default '' not null,
    album varchar(255) default '' not null,
    artist varchar(255) default '' not null,
    track_number integer default 0 not null
);

create table if not exists annotation (
    user_id varchar(255) not null primary key,
    item_id varchar(255) default '' not null,
	item_type varchar(255) default '' not null,
	play_count integer default 0,
	play_date datetime,
	rating integer default 0
);

insert into media_file values
("TkclRuUT3Ju381lf2Utlmd", "Band/Album/01 - Test Song.flac", "Test Song", "Album", "Band", 1),
("HSSNWzhymYyyqDEmFkK8I9", "Band/Album/02 - Test Song 2.flac", "Test Song 2", "Album", "Band", 2);

insert into annotation values
("kiNuIyhPxNjUKmhxY9DXty", "TkclRuUT3Ju381lf2Utlmd", "media_file", 3, "2026-04-23 02:02:39.804+00:00", 5),
("ddV5zjYx4IZJEj2QCaKXdw", "TkclRuUT3Ju381lf2Utlmd", "media_file", 7, "2026-04-24 18:38:47.945+00:00", 0);
