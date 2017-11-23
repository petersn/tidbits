#!/usr/bin/python
"""
Demonstration that separating out bit planes is often not better than just letting bz2 handle the raw data.
"""

import numpy as np

n = 160

# Make a grid of coordinates.
array = np.transpose(np.array(np.meshgrid(range(n), range(n))))
array -= [n/2, n/2]
# Convert to distances.
array = np.sum(array**2, axis=2)
# Make two arrays: One cutting out a big circle, and one a small circle.
a1 = (array < n**2 / 4).astype(np.int8)
a2 = (array < n**2 / 8).astype(np.int8)

# Our ultimate array is the sum of these two.
# Thus, its a big circle of 1s with a smaller circle of 2s inside of it, on a background of 0s.
a_sum = a1 + a2

def cost(a):
	return len(a.astype(np.int8).tostring().encode("bz2"))

print "Cost to encode plane of 1s:", cost(a1)
print "Cost to encode plane of 2s:", cost(a2)
print "Sum of above two costs:    ", cost(a1) + cost(a2)
print "Cost to encode raw data:   ", cost(a_sum)

