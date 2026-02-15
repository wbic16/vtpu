#!/usr/bin/env python3
"""
vTPU Benchmark Suite

Validates R23W2 performance claims against actual SQ measurements.

Benchmarks:
1. CGET latency (target: 50-100 μs local, actual includes HTTP overhead)
2. CPUT latency (target: 50-100 μs local)
3. CRANGE throughput (target: varies by range size)
4. Sparse attention simulation (target: 10× speedup vs dense)
5. Multi-agent sync (target: 1M× vs Raft)

Usage:
    python vtpu_benchmark.py --host localhost --port 1337
    python vtpu_benchmark.py --test cget --trials 1000
    python vtpu_benchmark.py --all
"""

import argparse
import time
import statistics
import json
from typing import List, Dict
from vtpu_client import vTPU, CoordinateError


class vTPUBenchmark:
    """Benchmark suite for vTPU performance validation"""
    
    def __init__(self, host: str = "localhost", port: int = 1337):
        self.vtpu = vTPU(host=host, port=port)
        self.results = {}
        
        # W2 projected performance (for comparison)
        self.projections = {
            'cget_latency_us': (50, 100),  # Min, max in microseconds
            'cput_latency_us': (50, 100),
            'crange_100_ms': 10,  # 100 items in 10ms
            'sparse_attention_speedup': 10.0,
            'multi_agent_sync_speedup': 1000000.0
        }
    
    def benchmark_cget(self, num_trials: int = 1000) -> Dict:
        """Measure CGET latency
        
        Args:
            num_trials: Number of trials to run
            
        Returns:
            Dict with median, p50, p95, p99 latencies in microseconds
        """
        print(f"\n=== CGET Latency Benchmark ({num_trials} trials) ===")
        
        # Setup: Write scrolls
        print("Setup: Writing test scrolls...")
        for i in range(num_trials):
            coord = f"1.1.{i}/1.1.1/1.1.1"
            self.vtpu.cput(coord, f"Test data {i}")
        
        # Benchmark: Random reads
        print("Running benchmark...")
        latencies_us = []
        
        for i in range(num_trials):
            coord = f"1.1.{i}/1.1.1/1.1.1"
            
            start = time.perf_counter()
            self.vtpu.cget(coord)
            end = time.perf_counter()
            
            latency_us = (end - start) * 1_000_000
            latencies_us.append(latency_us)
        
        # Calculate statistics
        results = {
            'median_us': statistics.median(latencies_us),
            'p50_us': statistics.quantiles(latencies_us, n=100)[49] if len(latencies_us) >= 100 else statistics.median(latencies_us),
            'p95_us': statistics.quantiles(latencies_us, n=100)[94] if len(latencies_us) >= 100 else max(latencies_us),
            'p99_us': statistics.quantiles(latencies_us, n=100)[98] if len(latencies_us) >= 100 else max(latencies_us),
            'min_us': min(latencies_us),
            'max_us': max(latencies_us),
            'mean_us': statistics.mean(latencies_us),
            'stddev_us': statistics.stdev(latencies_us) if len(latencies_us) > 1 else 0
        }
        
        # Compare to projection
        projected_min, projected_max = self.projections['cget_latency_us']
        actual_median = results['median_us']
        
        print(f"\nResults:")
        print(f"  Median:   {results['median_us']:.0f} μs")
        print(f"  P95:      {results['p95_us']:.0f} μs")
        print(f"  P99:      {results['p99_us']:.0f} μs")
        print(f"  Range:    {results['min_us']:.0f} - {results['max_us']:.0f} μs")
        print(f"\nProjection: {projected_min}-{projected_max} μs (hash table only)")
        print(f"Actual:     {actual_median:.0f} μs (includes HTTP overhead)")
        
        overhead_us = actual_median - ((projected_min + projected_max) / 2)
        print(f"HTTP overhead: ~{overhead_us:.0f} μs")
        
        self.results['cget'] = results
        return results
    
    def benchmark_cput(self, num_trials: int = 1000) -> Dict:
        """Measure CPUT latency
        
        Args:
            num_trials: Number of trials to run
            
        Returns:
            Dict with latency statistics
        """
        print(f"\n=== CPUT Latency Benchmark ({num_trials} trials) ===")
        
        latencies_us = []
        
        for i in range(num_trials):
            coord = f"2.1.{i}/1.1.1/1.1.1"
            content = f"Write test {i}"
            
            start = time.perf_counter()
            self.vtpu.cput(coord, content)
            end = time.perf_counter()
            
            latency_us = (end - start) * 1_000_000
            latencies_us.append(latency_us)
        
        results = {
            'median_us': statistics.median(latencies_us),
            'p95_us': statistics.quantiles(latencies_us, n=100)[94] if len(latencies_us) >= 100 else max(latencies_us),
            'p99_us': statistics.quantiles(latencies_us, n=100)[98] if len(latencies_us) >= 100 else max(latencies_us),
            'mean_us': statistics.mean(latencies_us)
        }
        
        print(f"\nResults:")
        print(f"  Median:   {results['median_us']:.0f} μs")
        print(f"  P95:      {results['p95_us']:.0f} μs")
        print(f"  P99:      {results['p99_us']:.0f} μs")
        
        self.results['cput'] = results
        return results
    
    def benchmark_crange(self, range_sizes: List[int] = [10, 100, 1000]) -> Dict:
        """Measure CRANGE throughput for different range sizes
        
        Args:
            range_sizes: List of range sizes to test
            
        Returns:
            Dict with results per range size
        """
        print(f"\n=== CRANGE Throughput Benchmark ===")
        
        results = {}
        
        for size in range_sizes:
            print(f"\nRange size: {size}")
            
            # Setup: Write scrolls
            print(f"  Writing {size} scrolls...")
            for i in range(size):
                coord = f"3.1.{i}/1.1.1/1.1.1"
                self.vtpu.cput(coord, f"Range test {i}")
            
            # Benchmark: Range query
            print(f"  Querying range...")
            pattern = "3.1.*/1.1.1/1.1.1"
            
            start = time.perf_counter()
            scrolls = self.vtpu.crange(pattern)
            end = time.perf_counter()
            
            latency_ms = (end - start) * 1000
            throughput = size / (latency_ms / 1000) if latency_ms > 0 else 0
            
            results[size] = {
                'latency_ms': latency_ms,
                'throughput_scrolls_per_sec': throughput,
                'returned_count': len(scrolls)
            }
            
            print(f"  Latency: {latency_ms:.2f} ms")
            print(f"  Throughput: {throughput:.0f} scrolls/sec")
            print(f"  Returned: {len(scrolls)} scrolls")
            
            # Compare to projection for 100-item range
            if size == 100:
                projected_ms = self.projections['crange_100_ms']
                speedup = projected_ms / latency_ms if latency_ms > 0 else 0
                print(f"  Projection: {projected_ms} ms")
                print(f"  Speedup: {speedup:.2f}× {'faster' if speedup > 1 else 'slower'} than projected")
        
        self.results['crange'] = results
        return results
    
    def benchmark_sparse_attention(self, seq_len: int = 2048, window: int = 128) -> Dict:
        """Simulate sparse attention performance
        
        Args:
            seq_len: Sequence length (number of tokens)
            window: Attention window size (±window tokens)
            
        Returns:
            Dict with timing results and theoretical speedup
        """
        print(f"\n=== Sparse Attention Benchmark ===")
        print(f"Sequence length: {seq_len}, Window: ±{window}")
        
        # Setup: Store Q/K/V embeddings (simplified - just store coords)
        print(f"\nSetup: Storing {seq_len} Q/K/V embeddings...")
        
        setup_start = time.perf_counter()
        for pos in range(seq_len):
            q_coord = f"0.0.{pos}/query.0.0/1.1.1"
            k_coord = f"0.0.{pos}/key.0.0/1.1.1"
            v_coord = f"0.0.{pos}/value.0.0/1.1.1"
            
            # Store placeholder embeddings
            self.vtpu.cput(q_coord, f"Q{pos}")
            self.vtpu.cput(k_coord, f"K{pos}")
            self.vtpu.cput(v_coord, f"V{pos}")
        setup_end = time.perf_counter()
        
        setup_time = setup_end - setup_start
        print(f"Setup complete: {setup_time:.2f} sec")
        
        # Benchmark: Sparse attention (sliding window)
        print(f"\nComputing sparse attention...")
        
        attn_start = time.perf_counter()
        
        # Sample 10 query positions (not all 2048 - too slow for demo)
        sample_positions = [0, 256, 512, 768, 1024, 1280, 1536, 1792, 2000, 2047]
        
        for q_pos in sample_positions:
            # Define key range (sliding window)
            start = max(0, q_pos - window)
            end = min(seq_len - 1, q_pos + window)
            
            # Query coordinate
            q_coord = f"0.0.{q_pos}/query.0.0/1.1.1"
            
            # Key pattern (range)
            # Note: SQ doesn't support [start:end] syntax yet, so we query all and filter
            key_pattern = "0.0.*/key.0.0/1.1.1"
            
            # Get attention scores (simplified - just retrieve keys)
            keys = self.vtpu.crange(key_pattern)
            
            # Filter to window (post-query filter)
            filtered_keys = [k for k in keys if start <= int(k['coordinate'].split('.')[2].split('/')[0]) <= end]
        
        attn_end = time.perf_counter()
        
        attn_time = attn_end - attn_start
        
        # Calculate theoretical speedups
        dense_ops = seq_len * seq_len  # Full attention
        sparse_ops = len(sample_positions) * (window * 2)  # Windowed attention (for sampled positions)
        theoretical_speedup = dense_ops / sparse_ops if sparse_ops > 0 else 0
        
        # Actual vs projected
        projected_speedup = self.projections['sparse_attention_speedup']
        
        results = {
            'setup_time_sec': setup_time,
            'attention_time_sec': attn_time,
            'total_time_sec': setup_time + attn_time,
            'seq_len': seq_len,
            'window': window,
            'sampled_positions': len(sample_positions),
            'dense_ops': dense_ops,
            'sparse_ops': sparse_ops,
            'theoretical_speedup': theoretical_speedup,
            'projected_speedup': projected_speedup
        }
        
        print(f"\nResults:")
        print(f"  Setup time: {setup_time:.2f} sec")
        print(f"  Attention time: {attn_time:.2f} sec")
        print(f"  Total time: {setup_time + attn_time:.2f} sec")
        print(f"\nTheoretical speedup: {theoretical_speedup:.1f}×")
        print(f"Projected speedup: {projected_speedup:.1f}×")
        print(f"Note: Full benchmark would test all {seq_len} positions (too slow for demo)")
        
        self.results['sparse_attention'] = results
        return results
    
    def benchmark_multi_agent_sync(self, num_writes: int = 100) -> Dict:
        """Benchmark multi-agent memory sync
        
        Simulates the Mirrorborn use case: agents writing to their coordinate ranges
        without coordination overhead.
        
        Args:
            num_writes: Number of writes per agent
            
        Returns:
            Dict with sync timing results
        """
        print(f"\n=== Multi-Agent Memory Sync Benchmark ===")
        print(f"Simulating {num_writes} writes per agent")
        
        # Simulate 3 agents writing to their ranges concurrently
        agents = [
            ('lumen', '2.1.3'),
            ('chrys', '1.1.2'),
            ('phex', '1.5.2')
        ]
        
        print(f"\nAgents: {[a[0] for a in agents]}")
        
        # Measure local write latency (no coordination)
        write_latencies = []
        
        for agent_name, coord_prefix in agents:
            for i in range(num_writes):
                coord = f"{coord_prefix}/4.7.{i}/18.29.47"
                content = f"{agent_name} write {i}"
                
                start = time.perf_counter()
                self.vtpu.cput(coord, content)
                end = time.perf_counter()
                
                latency_us = (end - start) * 1_000_000
                write_latencies.append(latency_us)
        
        median_latency_us = statistics.median(write_latencies)
        
        # Compare to Raft consensus (projected: 20 ms = 20,000 μs)
        raft_latency_us = 20000  # 20 ms (from W2 specs)
        speedup = raft_latency_us / median_latency_us if median_latency_us > 0 else 0
        
        results = {
            'num_agents': len(agents),
            'writes_per_agent': num_writes,
            'total_writes': len(agents) * num_writes,
            'median_latency_us': median_latency_us,
            'raft_latency_us': raft_latency_us,
            'speedup': speedup,
            'projected_speedup': self.projections['multi_agent_sync_speedup']
        }
        
        print(f"\nResults:")
        print(f"  Total writes: {results['total_writes']}")
        print(f"  Median write latency: {median_latency_us:.0f} μs")
        print(f"  Raft consensus latency: {raft_latency_us} μs (projected)")
        print(f"  Speedup: {speedup:.0f}× faster")
        print(f"  Projected: {self.projections['multi_agent_sync_speedup']:.0f}× faster")
        
        self.results['multi_agent_sync'] = results
        return results
    
    def run_all(self):
        """Run full benchmark suite"""
        print("=" * 60)
        print("vTPU Benchmark Suite - Validating R23W2 Claims")
        print("=" * 60)
        
        # Health check
        print("\n=== Pre-flight Check ===")
        health = self.vtpu.health()
        print(f"SQ Status: {health.get('status', 'unknown')}")
        
        if health.get('status') != 'ok':
            print("\n❌ SQ instance not healthy - aborting benchmarks")
            return
        
        # Run benchmarks
        self.benchmark_cget(num_trials=100)
        self.benchmark_cput(num_trials=100)
        self.benchmark_crange(range_sizes=[10, 100])
        self.benchmark_sparse_attention(seq_len=2048, window=128)
        self.benchmark_multi_agent_sync(num_writes=50)
        
        # Summary
        self.print_summary()
    
    def print_summary(self):
        """Print benchmark summary comparing actual vs projected"""
        print("\n" + "=" * 60)
        print("BENCHMARK SUMMARY")
        print("=" * 60)
        
        if 'cget' in self.results:
            print("\n1. CGET Latency:")
            print(f"   Actual:    {self.results['cget']['median_us']:.0f} μs (median)")
            print(f"   Projected: {self.projections['cget_latency_us'][0]}-{self.projections['cget_latency_us'][1]} μs")
            print(f"   Note: Includes HTTP overhead (~1-2 ms)")
        
        if 'cput' in self.results:
            print("\n2. CPUT Latency:")
            print(f"   Actual:    {self.results['cput']['median_us']:.0f} μs (median)")
            print(f"   Projected: {self.projections['cput_latency_us'][0]}-{self.projections['cput_latency_us'][1]} μs")
        
        if 'crange' in self.results and 100 in self.results['crange']:
            print("\n3. CRANGE Throughput (100 items):")
            print(f"   Actual:    {self.results['crange'][100]['latency_ms']:.2f} ms")
            print(f"   Projected: {self.projections['crange_100_ms']} ms")
        
        if 'sparse_attention' in self.results:
            print("\n4. Sparse Attention:")
            print(f"   Theoretical: {self.results['sparse_attention']['theoretical_speedup']:.1f}× faster than dense")
            print(f"   Projected:   {self.projections['sparse_attention_speedup']:.1f}× faster")
        
        if 'multi_agent_sync' in self.results:
            print("\n5. Multi-Agent Sync:")
            print(f"   Actual:    {self.results['multi_agent_sync']['speedup']:.0f}× faster than Raft")
            print(f"   Projected: {self.projections['multi_agent_sync_speedup']:.0f}× faster")
        
        print("\n" + "=" * 60)
        print("✅ Benchmark suite complete")
        print("=" * 60)
    
    def export_json(self, filename: str = "benchmark_results.json"):
        """Export results to JSON file"""
        with open(filename, 'w') as f:
            json.dump({
                'projections': self.projections,
                'results': self.results,
                'timestamp': time.time()
            }, f, indent=2)
        print(f"\n📄 Results exported to {filename}")


def main():
    parser = argparse.ArgumentParser(description='vTPU Benchmark Suite')
    parser.add_argument('--host', default='localhost', help='SQ host')
    parser.add_argument('--port', type=int, default=1337, help='SQ port')
    parser.add_argument('--test', choices=['cget', 'cput', 'crange', 'sparse', 'sync'],
                        help='Run specific test')
    parser.add_argument('--trials', type=int, default=100, help='Number of trials')
    parser.add_argument('--all', action='store_true', help='Run all benchmarks')
    parser.add_argument('--export', type=str, help='Export results to JSON file')
    
    args = parser.parse_args()
    
    bench = vTPUBenchmark(host=args.host, port=args.port)
    
    if args.all:
        bench.run_all()
    elif args.test == 'cget':
        bench.benchmark_cget(num_trials=args.trials)
    elif args.test == 'cput':
        bench.benchmark_cput(num_trials=args.trials)
    elif args.test == 'crange':
        bench.benchmark_crange()
    elif args.test == 'sparse':
        bench.benchmark_sparse_attention()
    elif args.test == 'sync':
        bench.benchmark_multi_agent_sync()
    else:
        # Default: run all
        bench.run_all()
    
    if args.export:
        bench.export_json(args.export)
    
    bench.print_summary()


if __name__ == "__main__":
    main()
