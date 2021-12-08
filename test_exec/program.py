#!/usr/bin/python

print("Launched!")

import os, sys
for k, v in os.environ.items():
    print(f"{k} = {v}")
print(sys.argv)

