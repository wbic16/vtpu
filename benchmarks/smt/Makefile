CC = gcc
CFLAGS = -O2 -Wall -march=native -std=c11 -pthread
LDFLAGS = -lm -pthread

TARGETS = single_thread dual_thread complementary_workloads

.PHONY: all clean run

all: $(TARGETS)

single_thread: single_thread.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

dual_thread: dual_thread.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

complementary_workloads: complementary_workloads.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

run: all
	@echo "=== Single-Thread Baseline ==="
	./single_thread
	@echo
	@echo "=== Dual-Thread (Same Workload) ==="
	./dual_thread
	@echo
	@echo "=== Complementary Workloads ==="
	./complementary_workloads

clean:
	rm -f $(TARGETS) *.o
