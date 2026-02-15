// Cache Locality Baseline: Random Access Pattern
// Measures L1/L2/L3 hit rates for unpredictable memory access
// Compile: gcc -O2 -o random_access random_access.c
// Run: perf stat -e L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses ./random_access

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <time.h>

#define KB (1024UL)
#define MB (1024UL * 1024UL)

#define L1_SIZE (16 * KB)
#define L2_SIZE (512 * KB)
#define L3_SIZE (16 * MB)
#define DRAM_SIZE (256 * MB)

#define ITERATIONS 100
#define ACCESSES_PER_ITER 10000  // Random lookups per iteration

static inline uint64_t rdtsc(void) {
    uint32_t lo, hi;
    __asm__ __volatile__ ("rdtsc" : "=a"(lo), "=d"(hi));
    return ((uint64_t)hi << 32) | lo;
}

// Simple LCG for deterministic pseudo-random indices
uint64_t lcg_state = 12345;
uint64_t lcg_next(uint64_t max) {
    lcg_state = lcg_state * 1103515245 + 12345;
    return lcg_state % max;
}

uint64_t random_sum(uint64_t *array, size_t count) {
    uint64_t sum = 0;
    for (int i = 0; i < ACCESSES_PER_ITER; i++) {
        size_t idx = lcg_next(count);
        sum += array[idx];
    }
    return sum;
}

void benchmark(const char *name, size_t bytes) {
    size_t count = bytes / sizeof(uint64_t);
    uint64_t *array = malloc(bytes);
    
    if (!array) {
        fprintf(stderr, "Failed to allocate %lu MB\n", bytes / MB);
        return;
    }
    
    // Initialize
    for (size_t i = 0; i < count; i++) {
        array[i] = i;
    }
    
    // Warmup
    lcg_state = 12345;
    random_sum(array, count);
    
    // Timed run
    lcg_state = 12345;
    uint64_t start = rdtsc();
    uint64_t checksum = 0;
    for (int iter = 0; iter < ITERATIONS; iter++) {
        checksum ^= random_sum(array, count);
    }
    uint64_t end = rdtsc();
    
    uint64_t cycles = (end - start) / ITERATIONS;
    double cycles_per_access = (double)cycles / ACCESSES_PER_ITER;
    
    printf("%s: %lu bytes, %lu cycles/iter, %.2f cycles/access (checksum: %lu)\n",
           name, bytes, cycles, cycles_per_access, checksum);
    
    free(array);
}

int main() {
    printf("Random Access Baseline (expect cache thrashing, high miss rates)\n\n");
    
    benchmark("L1  (16 KB)", L1_SIZE);
    benchmark("L2  (512 KB)", L2_SIZE);
    benchmark("L3  (16 MB)", L3_SIZE);
    benchmark("DRAM (256 MB)", DRAM_SIZE);
    
    return 0;
}
