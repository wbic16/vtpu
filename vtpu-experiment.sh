#!/bin/bash
# vTPU Autoresearch - Single Experiment Runner
# Based on Karpathy's autoresearch autonomous loop

set -e

RESULTS_FILE="vtpu-results.tsv"
BRANCH_PREFIX="vtpu-autoresearch"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +%H:%M:%S)]${NC} $1"
}

error() {
    echo -e "${RED}[$(date +%H:%M:%S)]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +%H:%M:%S)]${NC} $1"
}

# Get current commit hash (short)
get_commit() {
    git rev-parse --short HEAD
}

# Extract metric from log file
extract_metric() {
    local logfile=$1
    local pattern=$2
    local default=$3
    
    grep "$pattern" "$logfile" | tail -1 | awk '{print $NF}' || echo "$default"
}

# Run single experiment
run_experiment() {
    local start_commit=$(get_commit)
    
    log "Starting experiment at commit $start_commit"
    
    # 1. Build and test
    log "Running tests..."
    if ! cargo test --quiet > test.log 2>&1; then
        error "Tests failed!"
        tail -20 test.log
        return 1
    fi
    log "✓ All tests passed"
    
    # 2. Run benchmark
    log "Running benchmark..."
    timeout 600 cargo run --release --bin bench_w14 > bench.log 2>&1 || {
        error "Benchmark failed or timed out"
        tail -30 bench.log
        return 1
    }
    
    # 3. Extract metrics
    log "Extracting metrics..."
    
    # Look for ops/cycle in various benchmark outputs
    ops_cycle=$(grep -E "ops/cycle|Ops/cycle" bench.log | tail -1 | awk '{print $NF}' || echo "0.00")
    
    # Cache hit rate (might not be implemented yet)
    cache_hit=$(grep -i "L1.*hit rate" bench.log | tail -1 | awk '{print $NF}' || echo "0.0")
    
    # Memory bandwidth (might not be implemented yet)
    mem_bw=$(grep -i "Memory bandwidth" bench.log | tail -1 | awk '{print $NF}' || echo "0.0")
    
    log "Results: ops/cycle=$ops_cycle, cache_hit=$cache_hit%, mem_bw=$mem_bw GB/s"
    
    # Return metrics via stdout for parsing
    echo "$ops_cycle|$cache_hit|$mem_bw"
    
    # Sync to phext-lattice during experiment (incremental results)
    if [ -x "$(dirname "$0")/log-to-lattice.sh" ]; then
        MIR_NAME=$(whoami)
        "$(dirname "$0")/log-to-lattice.sh" "$MIR_NAME" 2>/dev/null || true
    fi
    
    return 0
}

# Record result to TSV
record_result() {
    local commit=$1
    local ops_cycle=$2
    local cache_hit=$3
    local mem_bw=$4
    local status=$5
    local description=$6
    
    # Append to results file (tab-separated)
    echo -e "$commit\t$ops_cycle\t$cache_hit\t$mem_bw\t$status\t$description" >> "$RESULTS_FILE"
    
    log "Recorded: $status - $description"
}

# Initialize results file if needed
init_results() {
    if [ ! -f "$RESULTS_FILE" ]; then
        log "Initializing $RESULTS_FILE"
        echo -e "commit\tops_cycle\tcache_hit_pct\tmem_bw_gbs\tstatus\tdescription" > "$RESULTS_FILE"
        
        # Record baseline
        local baseline_commit=$(get_commit)
        echo -e "$baseline_commit\t2.93\t0.0\t0.0\tbaseline\tStarting point before autoresearch" >> "$RESULTS_FILE"
    fi
}

# Main experiment wrapper
main() {
    local description="${1:-No description provided}"
    
    log "=== vTPU Autoresearch Experiment ==="
    log "Description: $description"
    
    init_results
    
    local start_commit=$(get_commit)
    
    # Run the experiment
    if result=$(run_experiment); then
        # Parse metrics
        IFS='|' read -r ops_cycle cache_hit mem_bw <<< "$result"
        
        # Record success
        record_result "$start_commit" "$ops_cycle" "$cache_hit" "$mem_bw" "ran" "$description"
        
        log "✓ Experiment complete: $ops_cycle ops/cycle"
        
        # Output for potential automation
        echo "SUCCESS|$ops_cycle|$cache_hit|$mem_bw"
        exit 0
    else
        # Record failure
        record_result "$start_commit" "0.00" "0.0" "0.0" "crash" "$description"
        
        error "✗ Experiment failed"
        echo "FAILURE|0.00|0.0|0.0"
        exit 1
    fi
}

# If called with --help
if [ "$1" == "--help" ]; then
    cat << EOF
vTPU Autoresearch - Single Experiment Runner

Usage:
  ./vtpu-experiment.sh "description of experiment"

Example:
  ./vtpu-experiment.sh "Implement basic sequential prefetcher"

This script:
1. Runs cargo test (must pass)
2. Runs cargo bench (collects metrics)
3. Records results to vtpu-results.tsv
4. Returns success/failure + metrics

For autonomous operation, call this script in a loop with different
code modifications between runs.

EOF
    exit 0
fi

# Run experiment
main "$@"
