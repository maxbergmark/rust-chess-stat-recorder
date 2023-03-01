import numpy as np
import matplotlib.pyplot as plt
from collections import defaultdict
import numpy_indexed as npi

game_player_data = np.dtype([
    ('elo', np.int16),
    ('missed_mates', np.int16),
    ('en_passant_mates', np.uint8),
    ('missed_en_passant_mates', np.uint8),
    ('en_passants', np.uint8),
    ('declined_en_passants', np.uint8),
])

game_player_data = np.dtype([
    ('white_player_data', game_player_data),
    ('black_player_data', game_player_data)
])

arr = np.fromfile("data.bin", dtype=game_player_data)

count = defaultdict(int)
hist = defaultdict(float)

arr["white_player_data"]["elo"] //= 10
arr["white_player_data"]["elo"] *= 10
key, value = npi.group_by(arr["white_player_data"]["elo"]).mean(arr["white_player_data"]["declined_en_passants"])
plt.plot(key, value, '.')
plt.title(f"Average missed mates per game ({arr.size} games)")
plt.ylabel("Average missed mates")
plt.xlabel("White player ELO")
plt.show()
quit()


for white_elo, black_elo, missed_mates in arr:
    count[white_elo] += 1
    hist[white_elo] += missed_mates

for k in count.keys():
    hist[k] /= count[k]

x = sorted(hist.keys())
y = [hist[k] for k in x]
plt.bar(x, y)
plt.show()