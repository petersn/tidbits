// Random malloc.

#define _GNU_SOURCE

#include <dlfcn.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <assert.h>
#include <stdio.h>
#include <time.h>
#include <stdint.h>
//#include <mutex>

#define BUFFER_SIZE (1 << 30)

static void* (*real_malloc)(size_t) = NULL;
static void* main_memory = NULL;
static size_t malloc_count = 0;
static size_t free_count = 0;
static size_t used_index = 0;
//std::mutex global_lock;

uint64_t next_rand() {
	static uint64_t counter = 0;
	uint64_t state = ++counter;
	for (int i = 0; i < 6; i++) {
		state *= 0x2d4aae4f5a4e9c5bull;
		state ^= state >> 37;
	}
	return state;
}

static void setup() {
	srand(time(NULL));
	real_malloc = dlsym(RTLD_NEXT, "malloc");
	main_memory = real_malloc(BUFFER_SIZE);
	fprintf(stderr, "Initialized: %p\n", main_memory);
}

void* random_malloc(size_t size) {
	//std::lock_guard<std::mutex> guard(global_lock);
	malloc_count++;
	if (malloc_count % 100 == 0 || malloc_count < 10)
		fprintf(stderr, "Mallocs: %llu   Frees: %llu\n", malloc_count, free_count);
	if (!real_malloc)
		setup();

	size_t valid_offsets = (BUFFER_SIZE - size) / 16;
	size_t off = 16 * (next_rand() % valid_offsets);
	fprintf(stderr, "Offset: %llx (size: %llu)\n", off, size);
	return main_memory + off;
}

void* malloc_no_lock(size_t size) {
	malloc_count++;
	if (malloc_count % 100 == 0 || malloc_count < 10)
		fprintf(stderr, "Mallocs: %llu   Frees: %llu\n", malloc_count, free_count);
	if (!real_malloc)
		setup();

	fprintf(stderr, "PID: %i   Used quantity: %llu   Size: %llu\n", getpid(), used_index, size);
	if (used_index + size > BUFFER_SIZE)
		used_index = 0;
	void* result = main_memory + used_index;
	used_index += size;
	return real_malloc(size);
}

void* malloc(size_t size) {
	//std::lock_guard<std::mutex> guard(global_lock);
	//return malloc_no_lock(size);
	return random_malloc(size);
}

void* realloc(void* ptr, size_t new_size) {
	//std::lock_guard<std::mutex> guard(global_lock);
	if (ptr == NULL)
		return random_malloc(new_size);
		//return malloc_no_lock(new_size);

	//void* new_buffer = malloc_no_lock(new_size);
	void* new_buffer = random_malloc(new_size);
	free_count++;
	// This copies too much, but oh well!
	memmove(new_buffer, ptr, new_size);
	return new_buffer;
}

void free(void* ptr) {
	//std::lock_guard<std::mutex> guard(global_lock);
	free_count++;
}

