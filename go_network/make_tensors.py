import glob
import numpy as np
from tqdm.contrib.concurrent import process_map
from sgfmill import sgf, sgf_moves
import model

sgf_files = glob.glob("*/*/*.sgf")
print(f"Found {len(sgf_files)} SGF files")

def process_file(path):
    with open(path, "rb") as f:
        sgf_content = f.read()
    try:
        sgf_game = sgf.Sgf_game.from_bytes(sgf_content)
    except ValueError as e:
        msg, = e.args
        assert "bad SZ property" in msg, "Unexpected error: %s" % msg
        return

    root = sgf_game.get_root()
    board_size = root.get("SZ")
    if board_size != 19:
        return

    board, plays = sgf_moves.get_setup_and_moves(sgf_game)
    training_pairs = []
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
    return training_pairs

outputs = process_map(process_file, sgf_files)
outputs = [x for x in outputs if x is not None]
game_count = len(outputs)
training_pairs = [pair for pairs in outputs for pair in pairs]

print(f"Processed {game_count} games, with {len(training_pairs)} training pairs")

input_tensors = np.array([x for x, _ in training_pairs])
target_tensors = np.array([19*r + c for _, (r, c) in training_pairs])

with open("training_tensors.npy", "wb") as f:
    np.savez(f, inputs=input_tensors, targets=target_tensors)
