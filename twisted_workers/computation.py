#!/usr/bin/python

import time
import random

class ComputationEngine:
	def configure(self, blob):
		self.multiplier = int(blob)

	def compute(self):
		time.sleep(random.uniform(0.05, 0.1))
		result = self.multiplier * random.randrange(10)
		return str(result).encode("ascii")

