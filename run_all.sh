for y in {2013..2013}; do
  for m in {1..12}; do
    formatted_month=$(printf "%02d" $m);
#    echo $formatted_month;
    website="https://database.lichess.org/standard/lichess_db_standard_rated_$y-$formatted_month.pgn.zst";
    elissa_filename="lichess_db_standard_rated_$y-$formatted_month.pgn.zst";
    filename="resources/lichess_db_standard_rated_$y-$formatted_month.remote";
    echo $website;
#    echo $website;
#    curl -s $website | cargo run --release $filename
    ssh elissa cat /home/max/storage/chess/$elissa_filename | pv |  cargo run --release $filename;
  done
done