import random
import functools
import numpy as np
import torch
from tqdm import tqdm
import sgfmill.boards
from torch.utils.data import DataLoader, TensorDataset

BATCH_SIZE = 1024
BLOCKS = 10
CHANNEL_COUNT = 128
INPUT_CHANNELS = 1 + 2 + 5
LEARNING_RATE = 5e-4
WARM_UP_STEPS = 200
LR_HALFLIFE = 15_000

class ConvBlock(torch.nn.Module):
    def __init__(self, filters, kernel_size=3):
        super().__init__()
        self.conv1 = torch.nn.Conv2d(filters, filters, kernel_size=kernel_size, padding="same")
        self.conv2 = torch.nn.Conv2d(filters, filters, kernel_size=kernel_size, padding="same")
        # Vageuly inspired by fixup initialization, see:
        # https://github.com/lightvector/KataGo/blob/master/docs/KataGoMethods.md
        # https://arxiv.org/pdf/1901.09321.pdf
        self.conv2.weight.data *= 1e-3
        self.activation = torch.nn.GELU()

    def forward(self, x):
        skip = x
        x = self.activation(x)
        x = self.conv1(x)
        x = self.activation(x)
        x = self.conv2(x)
        return x + skip

def make_network(
    blocks=BLOCKS,
    channel_count=CHANNEL_COUNT,
    input_channels=INPUT_CHANNELS,
):
    return torch.nn.Sequential(
        # Convert input to channel_count channels
        torch.nn.Conv2d(input_channels, channel_count, kernel_size=3, padding="same"),
        # Residual blocks
        *[ConvBlock(channel_count) for _ in range(blocks)],
        # Policy head
        torch.nn.Conv2d(channel_count, 1, kernel_size=3, padding="same"),
        torch.nn.Flatten(),
    )

neighbors = {}
for r in range(19):
    for c in range(19):
        neighbors[r, c] = []
        for nr, nc in ((r - 1, c), (r + 1, c), (r, c - 1), (r, c + 1)):
            if 0 <= nr < 19 and 0 <= nc < 19:
                neighbors[r, c].append((nr, nc))

def find_group(board, r, c, color):
    group = set([(r, c)])
    liberties = set()
    stack = [(r, c)]
    while stack:
        r, c = stack.pop()
        for nr, nc in neighbors[r, c]:
            if (nr, nc) in group:
                continue
            neighboring = board.get(nr, nc)
            if neighboring is None:
                liberties.add((nr, nc))
            elif neighboring == color:
                group.add((nr, nc))
                stack.append((nr, nc))
    return group, liberties

def encode_board(board, color_to_play):
    assert color_to_play in ("b", "w")
    inp = np.zeros((INPUT_CHANNELS, 19, 19), dtype=np.int8)
    # First plane is all 1s
    inp[0] = 1
    # Next two planes are our stones, then opponent stones
    for color, (r, c) in board.list_occupied_points():
        assert color in ("b", "w")
        which = 1 if color == color_to_play else 2
        inp[which, r, c] = 1
    # Compute liberties
    handled = set()
    for stone, (r, c) in board.list_occupied_points():
        assert stone in ("b", "w")
        if (r, c) in handled:
            continue
        group, liberties = find_group(board, r, c, stone)
        for r, c in group:
            assert len(liberties) >= 1
            inp[2 + min(len(liberties), 5), r, c] = 1
            handled.add((r, c))
    return inp

def apply_symmetry_to_board(symmetry_index, inp):
    assert inp.shape[-2:] == (19, 19)
    assert 0 <= symmetry_index < 8
    if symmetry_index & 1:
        inp = torch.flip(inp, [-1])
    if symmetry_index & 2:
        inp = torch.flip(inp, [-2])
    if symmetry_index & 4:
        inp = torch.transpose(inp, -1, -2)
    return inp

def inverse_symmetry(symmetry_index):
    assert 0 <= symmetry_index < 8
    return {5: 6, 6: 5}.get(symmetry_index, symmetry_index)

def apply_symmetry_to_move(symmetry_index, move: int) -> int:
    assert 0 <= symmetry_index < 8
    row = move // 19
    col = move % 19
    if symmetry_index & 1:
        col = 18 - col
    if symmetry_index & 2:
        row = 18 - row
    if symmetry_index & 4:
        row, col = col, row
    return 19*row + col

if __name__ == "__main__":
    import wandb
    wandb.init(project="go-policy", name=f"b{BLOCKS}c{CHANNEL_COUNT}")
    wandb.config.update({
        "BATCH_SIZE": BATCH_SIZE,
        "BLOCKS": BLOCKS,
        "CHANNEL_COUNT": CHANNEL_COUNT,
        "INPUT_CHANNELS": INPUT_CHANNELS,
        "LEARNING_RATE": LEARNING_RATE,
    })

    with open("training_tensors.npy", "rb") as f:
        data = np.load(f)
        inputs = data["inputs"]
        targets = data["targets"]
    print(f"Loaded {len(inputs)} training pairs")

    net = make_network().cuda()
    inputs = torch.tensor(inputs, dtype=torch.int8)
    targets = torch.tensor(targets, dtype=torch.int64)
    dataset = TensorDataset(inputs, targets)
    dataloader = DataLoader(dataset, batch_size=BATCH_SIZE, shuffle=True)
    cross_en = torch.nn.CrossEntropyLoss()
    optimizer = torch.optim.AdamW(net.parameters(), lr=0.0)

    epoch = total_steps = 0
    while True:
        epoch += 1
        print(f"Epoch {epoch}")
        for i, (inp, target) in enumerate(tqdm(dataloader)):
            total_steps += 1
            lr = LEARNING_RATE * 0.5**(total_steps / LR_HALFLIFE)
            lr *= min(1.0, total_steps / WARM_UP_STEPS)
            for g in optimizer.param_groups:
                g["lr"] = lr
            #symmetry = random.randrange(8)
            symmetry = epoch % 8
            inp = apply_symmetry_to_board(symmetry, inp).cuda().float()
            target = apply_symmetry_to_move(symmetry, target).cuda()
            optimizer.zero_grad()
            output = net(inp)
            loss = cross_en(output, target)
            loss.backward()
            optimizer.step()
            wandb.log({
                "loss": loss.item(),
                "lr": lr,
            })
            if i % 100 == 0:
                print(f"Batch {i}: loss = {loss.item()}")
                torch.save(net.state_dict(), f"models/model_{epoch}_{i}.pt")
