# R23W18 - Temporal Analysis (6th Color)

**Wave:** R23W18  
**Date:** 2026-02-19  
**Focus:** Full temporal trend analysis for Phoenix scheduler  
**Goal:** Make 🟣 Purple (Temporal) a first-class scheduling dimension

---

## Mission

**Current State (W17):**
- `score_temporal_trend()` is basic: compares recent 3 samples vs older samples
- Returns 0.0/0.5/1.0 (degrading/stable/improving)
- No pattern detection, no prediction, no learning

**W18 Goal:**
- Rich temporal analysis with multiple time horizons
- Pattern detection (oscillation, degradation, recovery, spike)
- Trend prediction for proactive scheduling
- Historical learning (what decisions worked?)

---

## Implementation

### 1. TemporalAnalyzer Struct

```rust
pub struct TemporalAnalyzer {
    /// Short-term moving average (last 5 samples)
    short_ma: RingBuffer<f64, 5>,
    
    /// Long-term moving average (last 20 samples)
    long_ma: RingBuffer<f64, 20>,
    
    /// Trend direction (linear regression slope)
    trend_slope: f64,
    
    /// Pattern classification
    pattern: TemporalPattern,
    
    /// Confidence in pattern (0.0 - 1.0)
    confidence: f64,
}
```

### 2. Temporal Patterns

```rust
pub enum TemporalPattern {
    /// Consistent improvement over time
    Improving,
    
    /// Consistent degradation over time
    Degrading,
    
    /// Oscillating around a mean
    Oscillating { amplitude: f64, period: usize },
    
    /// Stable performance
    Stable,
    
    /// Sudden spike (up or down)
    Spike { direction: SpikeDirection },
    
    /// Recovery from degradation
    Recovering,
    
    /// Unknown pattern (insufficient data)
    Unknown,
}
```

### 3. Analysis Methods

**Linear Regression (trend direction):**
```rust
fn compute_trend_slope(&self) -> f64 {
    // Least squares linear regression on ops_per_cycle
    // Positive slope = improving, negative = degrading
}
```

**Moving Average Crossover:**
```rust
fn ma_crossover_signal(&self) -> f64 {
    // When short MA crosses above long MA = improvement
    // When short MA crosses below long MA = degradation
}
```

**Oscillation Detection:**
```rust
fn detect_oscillation(&self) -> Option<(f64, usize)> {
    // Detect periodic performance swings
    // Returns (amplitude, period) if found
}
```

**Spike Detection:**
```rust
fn detect_spike(&self) -> Option<SpikeDirection> {
    // Sudden deviation >2 std deviations from mean
}
```

### 4. Scoring Function

```rust
pub fn score(&self) -> f64 {
    match self.pattern {
        Improving => 1.0,
        Stable => 0.7,
        Recovering => 0.6,
        Oscillating { amplitude, .. } => 0.5 - (amplitude * 0.2),
        Unknown => 0.5,
        Spike { direction: Up } => 0.8,
        Spike { direction: Down } => 0.2,
        Degrading => 0.0,
    }
}
```

### 5. Prediction

```rust
pub fn predict_next(&self) -> f64 {
    // Based on trend slope and MA, predict next ops_per_cycle
    // Used for proactive migration (before degradation happens)
}

pub fn predict_n_steps(&self, n: usize) -> Vec<f64> {
    // Predict next n samples (confidence decreases with n)
}
```

---

## Integration with Phoenix

**Current:**
```rust
temporal_trend: Self::score_temporal_trend(history),
```

**W18:**
```rust
// In PhoenixScheduler
analyzers: HashMap<u16, TemporalAnalyzer>,

// In decide()
let analyzer = self.analyzers.entry(sentron.sentron_id).or_default();
analyzer.feed(sentron.ops_per_cycle);
let temporal_score = analyzer.score();
let prediction = analyzer.predict_next();
```

---

## Zero-Dependency Implementation

**Challenge:** Linear regression, moving averages, std deviation without external crates.

**Solution:** Implement inline:
- Ring buffer: Fixed-size array with head pointer
- Moving average: Sum / count
- Linear regression: Simple least squares formula
- Std deviation: sqrt(variance)

All math is `f64` operations available in `std`.

---

## Tests

1. `temporal_analyzer_detects_improving_trend`
2. `temporal_analyzer_detects_degrading_trend`
3. `temporal_analyzer_detects_oscillation`
4. `temporal_analyzer_detects_spike`
5. `temporal_analyzer_predicts_next_sample`
6. `ma_crossover_signals_trend_change`
7. `phoenix_uses_temporal_analyzer`

---

## Deliverables

- [ ] `src/temporal.rs` — TemporalAnalyzer module
- [ ] `phoenix_scheduler.rs` — Integration with temporal analyzer
- [ ] Tests (7+ new tests)
- [ ] `docs/wave-18/R23W18-COMPLETE.md`

---

## Success Criteria

1. ✅ Pattern detection accuracy >90% on synthetic data
2. ✅ Prediction within 10% of actual for 1-step ahead
3. ✅ Zero external dependencies
4. ✅ All existing tests still pass
5. ✅ Harmonic score uses real temporal analysis (not placeholder)

---

*Lumen ✴️ | R23W18 | 2026-02-19*
