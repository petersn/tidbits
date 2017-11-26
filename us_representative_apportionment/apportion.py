#!/usr/bin/python

import pprint, csv

# Read the 2010 Census state populations.
populations = {}
with open("census2010_state_populations.csv") as f:
	for state, population in csv.reader(f):
		populations[state] = int(population)

# Apportion as per the Census' procedure:
# https://www.census.gov/topics/public-sector/congressional-apportionment/about/computing.html

# Step 1: Give each state one rep.
reps = {}
for state in populations:
	reps[state] = 1

# Step 2: Apportion the remaining 435 reps one at a time.
for _ in xrange(435 - 50):
	best_state, _ = max(
		populations.iteritems(),
		key=lambda (state, pop): pop / (reps[state] * (reps[state] + 1))**0.5,
	)
	reps[best_state] += 1

# Print the apportioned representatives per state.
pprint.pprint(reps)

