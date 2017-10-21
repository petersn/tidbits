// Test all possible inputs.

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <math.h>

#define N 29

static inline uint32_t test(uint32_t beta) {
	uint64_t numerator = ((uint64_t)beta) << N;
	uint32_t y = beta | (1 << 15);
	y |= (beta & 0x000000ffu) << 11;
	y |= (beta & 0x0000ffffu) << 7;
	y |= (beta & 0x00ffffffu) << 3;
	y |= (beta & 0x0fffffffu) << 1;
	int i;
	for (i = 0; i < 7; i++) {
		y += (uint32_t)(numerator / (uint64_t)y);
		y >>= 1;
	}
	// Here we are casting a 32-bit integer to a double, then multiplying by a dyadic rational in double range.
	// Thus, argument is an EXACT representation of the fixed-precision number we represent.
	double argument = ((double)beta) * (1.0 / (1 << N));
	// Here we evaluate sqrt(argument), which is accurate down to an ULP, and then multiply by an integer in double range.
	// Thus, by casting to uint32_t (truncating), correct_answer is the next lower fixed-point representation for the square root.
	uint32_t correct_answer = (sqrt(argument) * (1 << N));
	if (y == correct_answer)
		return 0;
	// Warning: I'm not happy also accepting correct_answer - 1... I'm pretty sure this is technically wrong, except when correct_answer was an integer before rounding...
	if (y == correct_answer + 1 || y == correct_answer - 1) {
		return 1;
	}

	// If we instead want to find the first argument at which this fails, uncomment this block.
#if 0
	printf("Error on: %u -- Got: %u  Wanted: %u\n", beta, y, correct_answer);
	exit(1);
#endif

	return 1000000000;
}

int main() {
	uint32_t test_case = 1;
	uint32_t ulps_of_error = 0;
	while (1) {
		ulps_of_error += test(test_case);
		test_case++;
		// Unsigned integer overflow IS defined behavior, so we
		// can count on the compiler not optimizing this away.
		if (test_case == 0)
			break;
		if ((test_case & ((1<<25)-1)) == 0) {
			printf("Validated up to: %u (%u ULPs of error so far)\n", test_case, ulps_of_error);
		}
	}
	printf("Validated all uint32_t values. Total ULPs of error: %u\n", ulps_of_error);
	return 0;
}

