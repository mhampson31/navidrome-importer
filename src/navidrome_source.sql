select
   	track.path as path,
    track.id as item_id,
    annotation.rating as rating,
    annotation.play_count as play_count,
    annotation.play_date as play_date

from media_file track

left join annotation
  on track.id = annotation.item_id
 and annotation.item_type = "media_file"
 and annotation.user_id = $1

where track.path = $2;
