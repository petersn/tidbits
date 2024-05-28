import random
import numpy as np
import torch
from tqdm import tqdm
from torch.utils.data import DataLoader, TensorDataset

BATCH_SIZE = 1024
BLOCKS = 10
CHANNEL_COUNT = 128
INPUT_CHANNELS = 2
LEARNING_RATE = 1e-3
WARM_UP_STEPS = 500
LR_HALFLIFE = 10_000

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

def encode_board(board, color_to_play):
    assert color_to_play in ("b", "w")
    inp = np.zeros((2, 19, 19), dtype=np.int8)
    for color, (r, c) in board.list_occupied_points():
        assert color in ("b", "w")
        which = +(color == color_to_play)
        inp[which, r, c] = 1
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

def apply_symmetry_to_move(symmetry_index, move):
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
    inputs = torch.tensor(inputs, dtype=torch.float32)
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
            inp = apply_symmetry_to_board(symmetry, inp).cuda()
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
