for y in {2013..2023}; do
  for m in {1..12}; do
    formatted_month=$(printf "%02d" $m);
#    echo $formatted_month;
    website="https://database.lichess.org/standard/lichess_db_standard_rated_$y-$formatted_month.pgn.zst";
    elissa_filename="lichess_db_standard_rated_$y-$formatted_month.pgn.zst";
    elissa_bin_filename="lichess_db_standard_rated_$y-$formatted_month.remote.bin";
    elissa_move_filename="lichess_db_standard_rated_$y-$formatted_month.remote.moves";
    filename="resources/lichess_db_standard_rated_$y-$formatted_month.remote";
    bin_filename="resources/lichess_db_standard_rated_$y-$formatted_month.remote.bin";
    move_filename="resources/lichess_db_standard_rated_$y-$formatted_month.remote.moves";
    echo $website;
#    echo $elissa_move_filename;
#    echo $website;
#    curl -s $website | cargo run --release $filename # run all files directly from lichess
    ssh elissa cat /home/max/storage/chess/$elissa_filename | pv |  cargo run --release $filename; # run all files from cluster HDD
#    rsync -P --ignore-existing $bin_filename elissa:/home/max/storage/chess/$elissa_bin_filename; # sync files to local computer
#    scp elissa:/home/max/storage/chess/$elissa_bin_filename $bin_filename;
  done
done
