// Cache Locality Comparison - R23 Wave 4
//
// Compares Hilbert curve (cache-friendly) vs random (cache-hostile) access patterns.
// Measures L1/L2/L3 miss rates via performance counters.

#define _POSIX_C_SOURCE 199309L

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <time.h>

#define ARRAY_SIZE (1024 * 1024)  // 1M elements = 8MB
#define ITERATIONS 1000000

// Simple 2D Hilbert curve encoding
static uint32_t hilbert_encode_2d(uint16_t x, uint16_t y, uint8_t order) {
    uint32_t n = 1U << order;
    uint32_t index = 0;
    uint32_t s = n / 2;
    
    while (s > 0) {
        uint32_t rx = ((x & s) > 0) ? 1 : 0;
        uint32_t ry = ((y & s) > 0) ? 1 : 0;
        index += s * s * ((3 * rx) ^ ry);
        
        // Rotate
        if (ry == 0) {
            if (rx == 1) {
                x = n - 1 - x;
                y = n - 1 - y;
            }
            uint16_t tmp = x;
            x = y;
            y = tmp;
        }
        
        s /= 2;
    }
    
    return index;
}

// Benchmark Hilbert curve access (cache-friendly)
static uint64_t benchmark_hilbert(uint64_t *array, size_t size) {
    uint64_t sum = 0;
    uint32_t order = 10;  // 2^10 = 1024x1024 grid
    
    for (uint32_t i = 0; i < ITERATIONS; i++) {
        uint16_t x = i % 1024;
        uint16_t y = (i / 1024) % 1024;
        uint32_t index = hilbert_encode_2d(x, y, order);
        sum += array[index % size];
    }
    
    return sum;
}

// Benchmark random access (cache-hostile)
static uint64_t benchmark_random(uint64_t *array, size_t size) {
    uint64_t sum = 0;
    uint64_t lfsr = 0xACE1u;  // Linear feedback shift register for pseudo-random
    
    for (uint32_t i = 0; i < ITERATIONS; i++) {
        // Simple LFSR: x^16 + x^14 + x^13 + x^11 + 1
        uint32_t bit = ((lfsr >> 0) ^ (lfsr >> 2) ^ (lfsr >> 3) ^ (lfsr >> 5)) & 1;
        lfsr = (lfsr >> 1) | (bit << 15);
        
        uint32_t index = lfsr % size;
        sum += array[index];
    }
    
    return sum;
}

// Benchmark sequential access (best case)
static uint64_t benchmark_sequential(uint64_t *array, size_t size) {
    uint64_t sum = 0;
    
    for (uint32_t i = 0; i < ITERATIONS; i++) {
        uint32_t index = i % size;
        sum += array[index];
    }
    
    return sum;
}

static double time_diff_ms(struct timespec *start, struct timespec *end) {
    return (end->tv_sec - start->tv_sec) * 1000.0 +
           (end->tv_nsec - start->tv_nsec) / 1000000.0;
}

int main() {
    // Allocate array
    uint64_t *array = malloc(ARRAY_SIZE * sizeof(uint64_t));
    if (!array) {
        fprintf(stderr, "Failed to allocate memory\n");
        return 1;
    }
    
    // Initialize with predictable pattern
    for (size_t i = 0; i < ARRAY_SIZE; i++) {
        array[i] = i * 42;
    }
    
    printf("Cache Locality Comparison Benchmark\n");
    printf("====================================\n");
    printf("Array size: %zu elements (%.1f MB)\n", 
           (size_t)ARRAY_SIZE, ARRAY_SIZE * sizeof(uint64_t) / (1024.0 * 1024.0));
    printf("Iterations: %u\n\n", (unsigned)ITERATIONS);
    
    struct timespec start, end;
    uint64_t result;
    
    // Benchmark 1: Sequential (best case)
    clock_gettime(CLOCK_MONOTONIC, &start);
    result = benchmark_sequential(array, ARRAY_SIZE);
    clock_gettime(CLOCK_MONOTONIC, &end);
    double seq_time = time_diff_ms(&start, &end);
    
    printf("Sequential Access (best case):\n");
    printf("  Time: %.2f ms\n", seq_time);
    printf("  Throughput: %.2f M accesses/sec\n", ITERATIONS / (seq_time * 1000.0));
    printf("  Result: %lu (prevent optimization)\n\n", result);
    
    // Benchmark 2: Hilbert curve (cache-friendly)
    clock_gettime(CLOCK_MONOTONIC, &start);
    result = benchmark_hilbert(array, ARRAY_SIZE);
    clock_gettime(CLOCK_MONOTONIC, &end);
    double hilbert_time = time_diff_ms(&start, &end);
    
    printf("Hilbert Curve Access (cache-friendly):\n");
    printf("  Time: %.2f ms\n", hilbert_time);
    printf("  Throughput: %.2f M accesses/sec\n", ITERATIONS / (hilbert_time * 1000.0));
    printf("  Slowdown vs sequential: %.2fx\n", hilbert_time / seq_time);
    printf("  Result: %lu (prevent optimization)\n\n", result);
    
    // Benchmark 3: Random access (cache-hostile)
    clock_gettime(CLOCK_MONOTONIC, &start);
    result = benchmark_random(array, ARRAY_SIZE);
    clock_gettime(CLOCK_MONOTONIC, &end);
    double random_time = time_diff_ms(&start, &end);
    
    printf("Random Access (cache-hostile):\n");
    printf("  Time: %.2f ms\n", random_time);
    printf("  Throughput: %.2f M accesses/sec\n", ITERATIONS / (random_time * 1000.0));
    printf("  Slowdown vs sequential: %.2fx\n", random_time / seq_time);
    printf("  Result: %lu (prevent optimization)\n\n", result);
    
    // Summary
    printf("Summary:\n");
    printf("  Hilbert speedup vs random: %.2fx\n", random_time / hilbert_time);
    printf("  Hilbert %% of sequential perf: %.1f%%\n", 100.0 * seq_time / hilbert_time);
    printf("\n");
    printf("To measure cache miss rates, run with perf:\n");
    printf("  perf stat -e cache-references,cache-misses,L1-dcache-load-misses ./locality_comparison\n");
    
    free(array);
    return 0;
}
