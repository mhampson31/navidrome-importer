select artist.title as artist,
    album.title as album,
    track.title as track,
    track."index" as track_nbr,
    part.file as path,
    /* Plex rating scale goes to 10; we need to scale to 5 *
	 * We want to round up, but SQLX doesn't seem to recognize ceil().
	 */
    cast((settings.rating / 2) + 0.5 as int) as rating,
    count(views.id) as play_count,
    datetime(max(views.viewed_at), 'unixepoch') as play_date

    from metadata_items artist

    join metadata_items album
        on artist.id = album.parent_id

    join metadata_items track
        on album.id = track.parent_id

    left join metadata_item_settings settings
        on settings.guid = track.guid
       and settings.account_id = (
               select a.id
               from accounts a
               where a.name = $2
            )

    /* Contains play information, one row per user/play.
     * Get the most recent play date and the total number of plays for the user.
     * Might not exist; user could have rated a song without playing it.
     */
    left join metadata_item_views views
        on views.guid = settings.guid
       and views.account_id = (
               select a.id
               from accounts a
               where a.name = $2
            )

    /* We don't need anything from media_items.
     * It just lets us link metadata_items to media_parts
     */
    join media_items media
        on track.id = media.metadata_item_id

    /* Contains the file path information */
    join media_parts part
        on part.media_item_id = media.id

    /* Only check for media items in the user's music library */
    where track.library_section_id in (
        select sl.library_section_id
        from section_locations sl
        where sl.root_path = $1
    )

    and (settings.rating is not null or views.account_id is not null)

    group by
        artist.title,
        album.title,
        track.title,
        track."index",
        part.file,
        settings.rating

    order by artist.title, album.title, track."index";
