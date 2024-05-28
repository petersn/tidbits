import glob
import numpy as np
from tqdm import tqdm
from sgfmill import sgf, sgf_moves
import model

sgf_files = glob.glob("*/*/*.sgf")
print(f"Found {len(sgf_files)} SGF files")

training_pairs = []
game_count = 0
for path in tqdm(sgf_files):
    with open(path, "rb") as f:
        sgf_content = f.read()
    try:
        sgf_game = sgf.Sgf_game.from_bytes(sgf_content)
    except ValueError as e:
        msg, = e.args
        assert "bad SZ property" in msg, "Unexpected error: %s" % msg
        continue

    root = sgf_game.get_root()
    board_size = root.get("SZ")
    if board_size != 19:
        continue

    board, plays = sgf_moves.get_setup_and_moves(sgf_game)
    for color_to_play, move in plays:
        assert color_to_play in ("b", "w")
        if move is None:
            continue
        inp = model.encode_board(board, color_to_play)
        training_pairs.append((inp, move))
        try:
            board.play(move[0], move[1], color_to_play)
        except ValueError:
            raise Exception("illegal move in sgf file")
    game_count += 1

print(f"Processed {game_count} games, with {len(training_pairs)} training pairs")

input_tensors = np.array([x for x, _ in training_pairs])
target_tensors = np.array([19*r + c for _, (r, c) in training_pairs])

with open("training_tensors.npy", "wb") as f:
    np.savez(f, inputs=input_tensors, targets=target_tensors)
