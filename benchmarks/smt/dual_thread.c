// Dual-Thread SMT Benchmark - R23 Wave 5
//
// Runs two threads on the same physical core to measure SMT efficiency.

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <time.h>
#include <pthread.h>
#include <sched.h>

#define ITERATIONS 10000000
#define ARRAY_SIZE 1024

typedef struct {
    int core_id;
    uint64_t *array;
    uint64_t result;
    double elapsed_ms;
} thread_data_t;

// D-Pipe operation
static inline uint64_t d_pipe_op(uint64_t a, uint64_t b, uint64_t c) {
    return (a * b) + c;
}

// S-Pipe operation
static inline uint64_t s_pipe_op(uint64_t *array, size_t index) {
    return array[index % ARRAY_SIZE];
}

// C-Pipe operation
static inline uint64_t c_pipe_op(uint64_t a, uint64_t b) {
    return (a ^ b) | (a & b);
}

// Balanced workload
uint64_t benchmark_balanced(uint64_t *array) {
    uint64_t result = 0;
    uint64_t reg_a = 1, reg_b = 2, reg_c = 3;
    
    for (uint32_t i = 0; i < ITERATIONS; i++) {
        reg_a = d_pipe_op(reg_a, reg_b, reg_c);
        uint64_t mem_val = s_pipe_op(array, i);
        reg_b = c_pipe_op(reg_a, mem_val);
        result += reg_a + reg_b;
    }
    
    return result;
}

void* thread_func(void *arg) {
    thread_data_t *data = (thread_data_t*)arg;
    
    // Pin to specific core
    cpu_set_t cpuset;
    CPU_ZERO(&cpuset);
    CPU_SET(data->core_id, &cpuset);
    pthread_setaffinity_np(pthread_self(), sizeof(cpuset), &cpuset);
    
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    data->result = benchmark_balanced(data->array);
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    data->elapsed_ms = (end.tv_sec - start.tv_sec) * 1000.0 +
                       (end.tv_nsec - start.tv_nsec) / 1000000.0;
    
    return NULL;
}

int main(int argc, char **argv) {
    int core_id = 0;
    if (argc > 1) {
        core_id = atoi(argv[1]);
    }
    
    printf("Dual-Thread SMT Benchmark\n");
    printf("=========================\n");
    printf("Core: %d (both threads on same physical core)\n", core_id);
    printf("Iterations per thread: %u\n", ITERATIONS);
    printf("Workload: Balanced (33%% D, 33%% S, 33%% C)\n\n");
    
    // Allocate arrays
    uint64_t *array1 = malloc(ARRAY_SIZE * sizeof(uint64_t));
    uint64_t *array2 = malloc(ARRAY_SIZE * sizeof(uint64_t));
    for (size_t i = 0; i < ARRAY_SIZE; i++) {
        array1[i] = i * 42;
        array2[i] = i * 43;
    }
    
    // Setup thread data
    thread_data_t data1 = { .core_id = core_id, .array = array1 };
    thread_data_t data2 = { .core_id = core_id, .array = array2 };
    
    // Launch threads
    pthread_t thread1, thread2;
    
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    pthread_create(&thread1, NULL, thread_func, &data1);
    pthread_create(&thread2, NULL, thread_func, &data2);
    
    pthread_join(thread1, NULL);
    pthread_join(thread2, NULL);
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    double total_elapsed = (end.tv_sec - start.tv_sec) * 1000.0 +
                           (end.tv_nsec - start.tv_nsec) / 1000000.0;
    
    // Calculate metrics
    double total_ops = 6.0 * ITERATIONS;  // 3 ops × 2 threads
    double throughput = total_ops / (total_elapsed / 1000.0);
    
    printf("Results:\n");
    printf("  Total time: %.2f ms\n", total_elapsed);
    printf("  Thread 1 time: %.2f ms\n", data1.elapsed_ms);
    printf("  Thread 2 time: %.2f ms\n", data2.elapsed_ms);
    printf("  Combined throughput: %.2f M ops/sec\n", throughput / 1000000.0);
    printf("  Thread 1 result: %lu\n", data1.result);
    printf("  Thread 2 result: %lu\n", data2.result);
    printf("\n");
    printf("SMT Efficiency:\n");
    printf("  Theoretical max: 2.0x (2 threads)\n");
    printf("  Expected SMT: 1.4-1.9x (resource sharing)\n");
    printf("  Actual speedup: %.2fx (vs single-thread baseline)\n", 
           throughput / (3.0 * ITERATIONS / (data1.elapsed_ms / 1000.0)));
    
    free(array1);
    free(array2);
    return 0;
}
