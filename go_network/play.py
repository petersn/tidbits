import os
import sgfmill.boards
import sgfmill.common
import torch
import model

script_dir = os.path.dirname(os.path.realpath(__file__))
net = model.make_network()
net.load_state_dict(torch.load(
    os.path.join(script_dir, "models/model_5_0.pt"),
    map_location=torch.device("cpu"),
))

def parse_gtp_color(color):
    color = color.lower()
    return {
        "b": "b",
        "black": "b",
        "w": "w",
        "white": "w",
    }[color]

class Engine:
    def __init__(self):
        self.reset_board()

    def reset_board(self):
        self.next_move_is_pass = False
        self.past_states = []
        self.board = sgfmill.boards.Board(19)

    def play(self, color, move):
        assert color in ("b", "w")
        self.board.play(move[0], move[1], color[0])
        self.past_states.append(self.board.copy().board)

    def genmove(self, color) -> str:
        inp = torch.tensor(model.encode_board(self.board, color), dtype=torch.float32)
        policy = net(inp).detach().numpy()
        assert policy.shape == (1, 19*19)
        while True:
            # We pass if there are no legal moves.
            if policy.max() <= -1e6:
                return
            argmax_index = policy.argmax()
            move = divmod(argmax_index, 19)
            test_board = self.board.copy()
            try:
                test_board.play(move[0], move[1], color)
                # We implement positional superko.
                if test_board.board in self.past_states:
                    raise ValueError("repeated board state")
            except ValueError:
                # Mark the move as invalid.
                policy[0, argmax_index] = -1e7
                continue
            return move

    def process_command(self, command: str):
        parts = command.strip().split()
        cmd = parts[0].lower()
        if cmd == "protocol_version":
            return "= 2\n"
        elif cmd == "name":
            return "= play.py\n"
        elif cmd == "version":
            return "= 0.1\n"
        elif cmd == "list_commands":
            return "= protocol_version\nname\nversion\nlist_commands\nboardsize\nclear_board\nkomi\nplay\ngenmove\nquit\n"
        elif cmd == "boardsize":
            size = int(parts[1])
            assert size == 19, "Only 19x19 boards are supported"
            self.reset_board()
            return "=\n"
        elif cmd == "clear_board":
            self.reset_board()
            return "=\n"
        elif cmd == "komi":
            return "=\n"
        elif cmd == "play":
            color, move = parse_gtp_color(parts[1]), parts[2]
            move = sgfmill.common.move_from_vertex(move, 19)
            if move is None:
                # We just assume the other engine knows what it's doing.
                self.next_move_is_pass = True
                return "=\n"
            row, col = move
            self.play(color, (row, col))
            return "=\n"
        elif cmd == "genmove":
            color = parse_gtp_color(parts[1])
            if self.next_move_is_pass:
                self.next_move_is_pass = False
                return "= pass\n"
            move = self.genmove(color)
            if move is not None:
                self.play(color, move)
            gtp_move = sgfmill.common.format_vertex(move)
            return f"= {gtp_move}\n"
        return "? unknown command\n"

if __name__ == "__main__":
    engine = Engine()
    while True:
        try:
            command = input()
            response = engine.process_command(command)
            print(response, flush=True)
            if command.strip().lower() == 'quit':
                break
        except EOFError:
            break
