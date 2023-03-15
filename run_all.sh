for y in {2018..2018}; do
  for m in {1..1}; do
    formatted_month=$(printf "%02d" $m);
#    echo $formatted_month;
    website="https://database.lichess.org/standard/lichess_db_standard_rated_$y-$formatted_month.pgn.zst";
    elissa_filename="lichess_db_standard_rated_$y-$formatted_month.pgn.zst";
    elissa_bin_filename="lichess_db_standard_rated_$y-$formatted_month.remote.bin";
    elissa_move_filename="lichess_db_standard_rated_$y-$formatted_month.remote.moves";
    filename="lichess_db_standard_rated_$y-$formatted_month.remote";
    bin_filename="lichess_db_standard_rated_$y-$formatted_month.remote.bin";
    move_filename="lichess_db_standard_rated_$y-$formatted_month.remote.moves";
    echo $filename;
#    run all files locally directly from lichess
#    curl -s $website | cargo run --release resources/$filename

#    run all locally using files from cluster HDD
    ssh elissa cat /home/max/storage/chess/$elissa_filename | pv | cargo run --release resources/$filename

#    run all directly on cluster
#    cat /home/max/storage/chess/$elissa_filename | pv | cargo run --release /home/max/storage/chess/$filename

#    sync files to local computer
#    rsync -P --ignore-existing resources/$bin_filename elissa:/home/max/storage/chess/$elissa_bin_filename
#    scp elissa:/home/max/storage/chess/$elissa_bin_filename $bin_filename;
  done
done
