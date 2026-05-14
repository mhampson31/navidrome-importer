select
    track.id as navidrome_id,
    track.artist,
    track.album,
    track.title as track,
    track.track_number as track_nbr,
   	track.path as path,
    annotation.rating as rating,
    annotation.play_count as play_count,
    annotation.play_date as play_date

from media_file track

left join annotation
  on track.id = annotation.item_id
 and annotation.item_type = "media_file"
 and annotation.user_id = $1;
