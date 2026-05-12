insert into annotation (
	user_id,
	item_id,
	item_type,
	play_count,
	play_date,
	rating,
	rated_at)

values (
	$1,
	$2,
	"media_file",
	$3,
	trim($4),
	$5,
	$6
)

on conflict (user_id, item_id, item_type)
do update set
	play_count = $3,
	play_date = $4,
	rating = $5,
	rated_at = $6;
