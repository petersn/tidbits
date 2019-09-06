// Random malloc.

#define _GNU_SOURCE

#include <dlfcn.h>
#include <unistd.h>
#include <string.h>
#include <iostream>
#include <cstdlib>
#include <cassert>
#include <ctime>
#include <mutex>

constexpr size_t BUFFER_SIZE = 1 << 30;

static void* (*real_malloc)(size_t) = nullptr;
static void* main_memory = nullptr;
static size_t malloc_count = 0;
static size_t free_count = 0;
static size_t used_index = 0;
std::mutex global_lock;

static void setup() {
	srand(time(nullptr));
	real_malloc = reinterpret_cast<decltype(real_malloc)>(dlsym(RTLD_NEXT, "malloc"));
	main_memory = real_malloc(BUFFER_SIZE);
	fprintf(stderr, "Initialized: %p\n", main_memory);
}

extern "C" void* random_malloc(size_t size) {
	std::lock_guard<std::mutex> guard(global_lock);
	malloc_count++;
	if (malloc_count % 100 == 0 or malloc_count < 10)
		fprintf(stderr, "Mallocs: %llu   Frees: %llu\n", malloc_count, free_count);
	if (!real_malloc)
		setup();

	size_t valid_offsets = BUFFER_SIZE - size;
	return main_memory + (rand() % valid_offsets);
}

extern "C" void* malloc_no_lock(size_t size) {
	malloc_count++;
	if (malloc_count % 100 == 0 or malloc_count < 10)
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

extern "C" void* malloc(size_t size) {
	std::lock_guard<std::mutex> guard(global_lock);
	return malloc_no_lock(size);
}

extern "C" void* realloc(void* ptr, size_t new_size) {
	std::lock_guard<std::mutex> guard(global_lock);
	if (ptr == nullptr)
		return malloc_no_lock(new_size);

	void* new_buffer = malloc_no_lock(new_size);
	// This copies too much, but oh well!
	memmove(new_buffer, ptr, new_size);
	return new_buffer;
}

extern "C" void free(void* ptr) {
	std::lock_guard<std::mutex> guard(global_lock);
	free_count++;
}

