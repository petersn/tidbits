import numpy as np
import torch
from tqdm import tqdm
from torch.utils.data import DataLoader, TensorDataset

BATCH_SIZE = 1024

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
    blocks=10,
    feature_count=128,
    input_channels=2,
):
    return torch.nn.Sequential(
        # Convert input to feature_count channels
        torch.nn.Conv2d(input_channels, feature_count, kernel_size=3, padding="same"),
        # Residual blocks
        *[ConvBlock(feature_count) for _ in range(blocks)],
        # Policy head
        torch.nn.Conv2d(feature_count, 1, kernel_size=3, padding="same"),
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

if __name__ == "__main__":
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
    optimizer = torch.optim.AdamW(net.parameters(), lr=5e-4)

    epoch = 0
    while True:
        epoch += 1
        print(f"Epoch {epoch}")
        for i, (inp, target) in enumerate(tqdm(dataloader)):
            inp, target = inp.cuda(), target.cuda()
            optimizer.zero_grad()
            output = net(inp)
            loss = cross_en(output, target)
            loss.backward()
            optimizer.step()
            if i % 100 == 0:
                print(f"Batch {i}: loss = {loss.item()}")
                torch.save(net.state_dict(), f"models/model_{epoch}_{i}.pt")
