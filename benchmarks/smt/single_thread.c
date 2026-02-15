// Single-Thread Baseline - R23 Wave 5
//
// Measures single-thread vTPU performance to establish baseline.

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <time.h>
#include <sched.h>
#include <unistd.h>

#define ITERATIONS 10000000
#define ARRAY_SIZE 1024

// Simulate D-Pipe operation (ALU-heavy)
static inline uint64_t d_pipe_op(uint64_t a, uint64_t b, uint64_t c) {
    return (a * b) + c;  // FMA
}

// Simulate S-Pipe operation (memory-heavy)
static inline uint64_t s_pipe_op(uint64_t *array, size_t index) {
    return array[index % ARRAY_SIZE];
}

// Simulate C-Pipe operation (coordination)
static inline uint64_t c_pipe_op(uint64_t a, uint64_t b) {
    return (a ^ b) | (a & b);  // Simple bit manipulation
}

// Balanced workload (33% D, 33% S, 33% C)
uint64_t benchmark_balanced(uint64_t *array) {
    uint64_t result = 0;
    uint64_t reg_a = 1, reg_b = 2, reg_c = 3;
    
    for (uint32_t i = 0; i < ITERATIONS; i++) {
        // D-Pipe
        reg_a = d_pipe_op(reg_a, reg_b, reg_c);
        
        // S-Pipe
        uint64_t mem_val = s_pipe_op(array, i);
        
        // C-Pipe
        reg_b = c_pipe_op(reg_a, mem_val);
        
        result += reg_a + reg_b;
    }
    
    return result;
}

// Pin thread to specific CPU core
void pin_to_core(int core_id) {
    cpu_set_t cpuset;
    CPU_ZERO(&cpuset);
    CPU_SET(core_id, &cpuset);
    
    if (sched_setaffinity(0, sizeof(cpuset), &cpuset) != 0) {
        perror("sched_setaffinity");
    }
}

static double time_diff_ms(struct timespec *start, struct timespec *end) {
    return (end->tv_sec - start->tv_sec) * 1000.0 +
           (end->tv_nsec - start->tv_nsec) / 1000000.0;
}

int main(int argc, char **argv) {
    int core_id = 0;
    if (argc > 1) {
        core_id = atoi(argv[1]);
    }
    
    // Pin to core
    pin_to_core(core_id);
    printf("Pinned to core %d\n", core_id);
    
    // Allocate array
    uint64_t *array = malloc(ARRAY_SIZE * sizeof(uint64_t));
    for (size_t i = 0; i < ARRAY_SIZE; i++) {
        array[i] = i * 42;
    }
    
    printf("Single-Thread Baseline Benchmark\n");
    printf("=================================\n");
    printf("Iterations: %u\n", ITERATIONS);
    printf("Workload: Balanced (33%% D, 33%% S, 33%% C)\n\n");
    
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    uint64_t result = benchmark_balanced(array);
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    double elapsed_ms = time_diff_ms(&start, &end);
    double ops_per_sec = (3.0 * ITERATIONS) / (elapsed_ms / 1000.0);  // 3 ops per iteration
    
    printf("Time: %.2f ms\n", elapsed_ms);
    printf("Throughput: %.2f M ops/sec\n", ops_per_sec / 1000000.0);
    printf("Result: %lu (prevent optimization)\n", result);
    
    free(array);
    return 0;
}
