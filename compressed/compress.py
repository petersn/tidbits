#!/usr/bin/python
"""
Demonstration that packing bit data is often not better for compression.
"""

import numpy as np

n = 160

# Make a grid of coordinates.
array = np.transpose(np.array(np.meshgrid(range(n), range(n))))
array -= [n/2, n/2]
# Convert to distances.
array = np.sum(array**2, axis=2)
# Cut out a circle.
a1 = (array < n**2 / 4).astype(np.int8)
a2 = (array < n**2 / 8).astype(np.int8)

a_sum = a1 + a2

print a1
print a2
print a_sum

def cost(a):
	return len(a.astype(np.int8).tostring().encode("bz2"))

# Convert to array of single bytes.
#unpacked = a2.astype(np.int8).tostring()

print cost(a_sum)
print cost(a1)
print cost(a2)
#print len(unpacked.encode("bz2"))

exit()

# Convert as a packed array.
packed = "".join(
	chr(sum(v << i for i, v in enumerate(byte)))
	for byte in zip(*[iter(array.flatten())]*8)
)

print "Unpacked length:", len(unpacked)
print "Packed length:  ", len(packed)

for algo in ("bz2", "zlib"):
	print
	print "Encoding:", algo
	print "Unpacked length:", len(unpacked.encode(algo))
	print "Packed length:  ", len(packed.encode(algo))

