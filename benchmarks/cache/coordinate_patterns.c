// Coordinate Access Patterns: Dimensional Locality Test
// Compares cache performance of phext-style dimensional access vs random access
// Compile: gcc -O2 -o coordinate_patterns coordinate_patterns.c
// Run: perf stat -e L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses ./coordinate_patterns

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

#define KB (1024UL)
#define MB (1024UL * 1024UL)

// Mock phext coordinate (11 dimensions, 11 bits each = 121 bits, use 128-bit)
typedef struct {
    uint16_t dims[11];  // Each dimension: 0-2047 (11 bits)
} PhextCoord;

// Mock coordinate hash: simple space-filling curve approximation
// Maps 11D coordinate to 1D address with locality preservation
uint64_t coord_to_address(PhextCoord *c, size_t array_size) {
    uint64_t hash = 0;
    
    // Interleave bits from all dimensions (simplified Z-order/Morton code)
    // In reality, we'd use Hilbert curve for better locality
    for (int bit = 0; bit < 11; bit++) {
        for (int dim = 0; dim < 11; dim++) {
            hash = (hash << 1) | ((c->dims[dim] >> bit) & 1);
        }
    }
    
    return hash % array_size;
}

// Generate coordinate that differs only in one dimension
PhextCoord neighbor_coord(PhextCoord base, int dim, int delta) {
    PhextCoord result = base;
    result.dims[dim] = (result.dims[dim] + delta) % 2048;
    return result;
}

static inline uint64_t rdtsc(void) {
    uint32_t lo, hi;
    __asm__ __volatile__ ("rdtsc" : "=a"(lo), "=d"(hi));
    return ((uint64_t)hi << 32) | lo;
}

// Test 1: Dimensional locality (neighbors in same dimension)
uint64_t test_dimensional_access(uint64_t *array, size_t count) {
    PhextCoord base = {{1,1,1,1,1,1,1,1,1,1,1}};
    uint64_t sum = 0;
    
    const int ACCESSES = 10000;
    for (int i = 0; i < ACCESSES; i++) {
        // Access 10 neighbors along dimension 0 (should be cache-local)
        for (int j = 0; j < 10; j++) {
            PhextCoord coord = neighbor_coord(base, 0, j);
            uint64_t addr = coord_to_address(&coord, count);
            sum += array[addr];
        }
        
        // Move to next section (change dim 1)
        base.dims[1]++;
    }
    
    return sum;
}

// Test 2: Random access (no locality)
uint64_t lcg_state = 12345;
uint64_t lcg_next(uint64_t max) {
    lcg_state = lcg_state * 1103515245 + 12345;
    return lcg_state % max;
}

uint64_t test_random_access(uint64_t *array, size_t count) {
    uint64_t sum = 0;
    const int ACCESSES = 100000;
    
    lcg_state = 12345;
    for (int i = 0; i < ACCESSES; i++) {
        uint64_t addr = lcg_next(count);
        sum += array[addr];
    }
    
    return sum;
}

// Test 3: Strided access (traditional sequential)
uint64_t test_strided_access(uint64_t *array, size_t count) {
    uint64_t sum = 0;
    const int ACCESSES = 100000;
    
    for (int i = 0; i < ACCESSES; i++) {
        size_t addr = (i * 64) % count;  // 64-element stride
        sum += array[addr];
    }
    
    return sum;
}

void run_test(const char *name, uint64_t (*test_func)(uint64_t*, size_t), uint64_t *array, size_t count) {
    // Warmup
    test_func(array, count);
    
    // Timed run
    uint64_t start = rdtsc();
    uint64_t result = test_func(array, count);
    uint64_t end = rdtsc();
    
    uint64_t cycles = end - start;
    
    printf("%-25s: %10lu cycles (result: %lu)\n", name, cycles, result);
}

int main() {
    // Test array spanning L1/L2/L3
    size_t sizes[] = {16*KB, 512*KB, 16*MB};
    const char *names[] = {"L1 (16 KB)", "L2 (512 KB)", "L3 (16 MB)"};
    
    printf("Coordinate Access Patterns vs Traditional Access\n");
    printf("Hypothesis: Dimensional locality → higher cache hit rates\n\n");
    
    for (int s = 0; s < 3; s++) {
        size_t array_size = sizes[s] / sizeof(uint64_t);
        uint64_t *array = malloc(sizes[s]);
        
        for (size_t i = 0; i < array_size; i++) {
            array[i] = i;
        }
        
        printf("=== %s ===\n", names[s]);
        run_test("Dimensional (neighbors)", test_dimensional_access, array, array_size);
        run_test("Random access", test_random_access, array, array_size);
        run_test("Strided access", test_strided_access, array, array_size);
        printf("\n");
        
        free(array);
    }
    
    printf("Expected: Dimensional access should have ~2-3x fewer cycles than random\n");
    printf("Run with perf to see cache miss rates:\n");
    printf("  perf stat -e L1-dcache-loads,L1-dcache-load-misses ./coordinate_patterns\n");
    
    return 0;
}
