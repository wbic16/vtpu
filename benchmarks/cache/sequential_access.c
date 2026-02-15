// Cache Locality Baseline: Sequential Access Pattern
// Measures L1/L2/L3 hit rates for predictable memory access
// Compile: gcc -O2 -o sequential_access sequential_access.c
// Run: perf stat -e L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses ./sequential_access

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <time.h>

#define KB (1024UL)
#define MB (1024UL * 1024UL)

// Test sizes spanning L1/L2/L3/DRAM
#define L1_SIZE (16 * KB)     // Fits in L1D (32KB cache)
#define L2_SIZE (512 * KB)    // Fits in L2 (1MB cache)
#define L3_SIZE (16 * MB)     // Fits in L3 (32MB cache)
#define DRAM_SIZE (256 * MB)  // Exceeds all caches

// Number of iterations to ensure measurable cycles
#define ITERATIONS 100

static inline uint64_t rdtsc(void) {
    uint32_t lo, hi;
    __asm__ __volatile__ ("rdtsc" : "=a"(lo), "=d"(hi));
    return ((uint64_t)hi << 32) | lo;
}

uint64_t sequential_sum(uint64_t *array, size_t count) {
    uint64_t sum = 0;
    for (size_t i = 0; i < count; i++) {
        sum += array[i];
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
    
    // Initialize to prevent optimization
    for (size_t i = 0; i < count; i++) {
        array[i] = i;
    }
    
    // Warmup
    sequential_sum(array, count);
    
    // Timed run
    uint64_t start = rdtsc();
    uint64_t checksum = 0;
    for (int iter = 0; iter < ITERATIONS; iter++) {
        checksum ^= sequential_sum(array, count);
    }
    uint64_t end = rdtsc();
    
    uint64_t cycles = (end - start) / ITERATIONS;
    double cycles_per_element = (double)cycles / count;
    
    printf("%s: %lu bytes, %lu cycles, %.2f cycles/element (checksum: %lu)\n",
           name, bytes, cycles, cycles_per_element, checksum);
    
    free(array);
}

int main() {
    printf("Sequential Access Baseline (AMD R9 8945HS expected: L1=4 cycles, L2=12 cycles, L3=40 cycles, DRAM=80 cycles per 64B line)\n\n");
    
    benchmark("L1  (16 KB)", L1_SIZE);
    benchmark("L2  (512 KB)", L2_SIZE);
    benchmark("L3  (16 MB)", L3_SIZE);
    benchmark("DRAM (256 MB)", DRAM_SIZE);
    
    return 0;
}
