

def check_piece_moves(all_moves):
    for piece in "KQRNB":
        for file in "abcdefgh":
            for rank in "12345678":
                for checks in ("#", "+", ""):
                    for capture in ("x", ""):
                        move = f"{piece}{capture}{file}{rank}{checks}"
                        if move not in all_moves:
                            print(f"    {move} has not been played")

def possible_pawn_moves(file, rank):
    yield f"{file}{rank}"
    if file < "h":
        yield f"{chr(ord(file)+1)}x{file}{rank}"
    if file > "a":
        yield f"{chr(ord(file)-1)}x{file}{rank}"

def check_pawn_moves(all_moves):
    moves = []
    for file in "abcdefgh":
        for rank in "12345678":
            for checks in ("#", "+", ""):
                for move in possible_pawn_moves(file, rank):
                    if rank in "18":
                        moves.append(f"{move}=Q{checks}")
                        moves.append(f"{move}=R{checks}")
                        moves.append(f"{move}=N{checks}")
                        moves.append(f"{move}=B{checks}")
                    else:
                        moves.append(f"{move}{checks}")

    for move in moves:
        if move not in all_moves:
            print(f"    {move} has not been played")

def check_all_moves(all_moves):
    check_piece_moves(all_moves)
    check_pawn_moves(all_moves)

def parse_move_files():
    all_moves = defaultdict(lambda: [0, 4000000000, ""])
    for filename in sorted(filter(lambda s: s.endswith(".remote.moves"), os.listdir("resources"))):
        for line in open(f"resources/{filename}", "r"):
            key, value, first_played, game_link = line.strip().split(", ")
            value = int(value)
            first_played = int(first_played)
            if first_played < all_moves[key][1]:
                all_moves[key] = [all_moves[key][0] + value, first_played, game_link]
            else:
                all_moves[key][0] += value

    print()
    print(f"Total moves: {sum(map(lambda e: e[0], all_moves.values())):.2e}")

    print("Unique moves:", len(all_moves))
    print()
    print("Moved that have never been played:")
    check_all_moves(all_moves)
    move_list = sorted([(k, *v) for k, v in all_moves.items()], key=lambda e: e[2], reverse=True)
    print()
    print("10 most recent new moves:")
    for i, move_data in enumerate(move_list[:10]):
        print(f"    {i+1:2d}: {move_data[0]:8s} (played {move_data[1]:2d} times) {datetime.fromtimestamp(move_data[2]).isoformat()} https://lichess.org/{move_data[3]}")
    print()
    print("Moves only played once:")
    for k, (v, first_played, game_link) in all_moves.items():
        if v == 1:
            print(f"    {k:8s} https://lichess.org/{game_link}")

    print()
    x = sorted(map(lambda e: e[1], all_moves.values()))
    y = list(range(1, len(x)+1))
#     plt.semilogx(x, y)
#     plt.show()
