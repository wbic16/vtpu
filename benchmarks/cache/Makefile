CC = gcc
CFLAGS = -O2 -Wall -march=native -std=c11
LDFLAGS = -lm

TARGETS = sequential_access random_access coordinate_patterns locality_comparison

.PHONY: all clean run profile

all: $(TARGETS)

sequential_access: sequential_access.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

random_access: random_access.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

coordinate_patterns: coordinate_patterns.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

locality_comparison: locality_comparison.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

run: all
	@echo "=== Sequential Access (L1/L2/L3 sweep) ==="
	./sequential_access
	@echo ""
	@echo "=== Random Access (cache pressure) ==="
	./random_access
	@echo ""
	@echo "=== Coordinate Patterns (dimensional locality) ==="
	./coordinate_patterns

profile: all
	@echo "=== Profiling sequential_access ==="
	perf stat -e cache-references,cache-misses,L1-dcache-loads,L1-dcache-load-misses,cycles,instructions ./sequential_access
	@echo ""
	@echo "=== Profiling random_access ==="
	perf stat -e cache-references,cache-misses,L1-dcache-loads,L1-dcache-load-misses,cycles,instructions ./random_access
	@echo ""
	@echo "=== Profiling coordinate_patterns ==="
	perf stat -e cache-references,cache-misses,L1-dcache-loads,L1-dcache-load-misses,cycles,instructions ./coordinate_patterns

clean:
	rm -f $(TARGETS) *.o
