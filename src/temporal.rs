//! Temporal Analysis — 🟣 Purple Dimension
//!
//! Rich temporal trend analysis for the Phoenix scheduler.
//! Detects patterns, predicts trends, enables proactive scheduling.
//!
//! Zero external dependencies — all math is inline.

/// Fixed-size ring buffer for moving averages
#[derive(Debug, Clone)]
pub struct RingBuffer<const N: usize> {
    data: [f64; N],
    head: usize,
    count: usize,
}

impl<const N: usize> Default for RingBuffer<N> {
    fn default() -> Self {
        Self {
            data: [0.0; N],
            head: 0,
            count: 0,
        }
    }
}

impl<const N: usize> RingBuffer<N> {
    pub fn push(&mut self, value: f64) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % N;
        if self.count < N {
            self.count += 1;
        }
    }
    
    pub fn is_full(&self) -> bool {
        self.count == N
    }
    
    pub fn len(&self) -> usize {
        self.count
    }
    
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    
    /// Get values in chronological order (oldest first)
    pub fn values(&self) -> Vec<f64> {
        if self.count == 0 {
            return Vec::new();
        }
        
        let mut result = Vec::with_capacity(self.count);
        let start = if self.count < N {
            0
        } else {
            self.head
        };
        
        for i in 0..self.count {
            let idx = (start + i) % N;
            result.push(self.data[idx]);
        }
        result
    }
    
    /// Arithmetic mean
    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.data[..self.count.min(N)].iter().sum::<f64>() / self.count as f64
    }
    
    /// Standard deviation
    pub fn std_dev(&self) -> f64 {
        if self.count < 2 {
            return 0.0;
        }
        let mean = self.mean();
        let variance = self.values()
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (self.count - 1) as f64;
        variance.sqrt()
    }
    
    /// Most recent value
    pub fn last(&self) -> Option<f64> {
        if self.count == 0 {
            return None;
        }
        let idx = if self.head == 0 { N - 1 } else { self.head - 1 };
        Some(self.data[idx])
    }
}

/// Spike direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpikeDirection {
    Up,
    Down,
}

/// Temporal pattern classification
#[derive(Debug, Clone, PartialEq)]
pub enum TemporalPattern {
    /// Consistent improvement over time
    Improving,
    
    /// Consistent degradation over time
    Degrading,
    
    /// Oscillating around a mean
    Oscillating { amplitude: f64 },
    
    /// Stable performance
    Stable,
    
    /// Sudden spike (up or down)
    Spike { direction: SpikeDirection },
    
    /// Recovery from degradation
    Recovering,
    
    /// Unknown pattern (insufficient data)
    Unknown,
}

/// Temporal Analyzer — Rich trend analysis
#[derive(Debug, Clone)]
pub struct TemporalAnalyzer {
    /// Short-term buffer (5 samples)
    short_term: RingBuffer<5>,
    
    /// Long-term buffer (20 samples)
    long_term: RingBuffer<20>,
    
    /// All-time high
    all_time_high: f64,
    
    /// All-time low
    all_time_low: f64,
    
    /// Total samples seen
    total_samples: u64,
    
    /// Previous pattern (for detecting transitions)
    prev_pattern: TemporalPattern,
}

impl Default for TemporalAnalyzer {
    fn default() -> Self {
        Self {
            short_term: RingBuffer::default(),
            long_term: RingBuffer::default(),
            all_time_high: f64::NEG_INFINITY,
            all_time_low: f64::INFINITY,
            total_samples: 0,
            prev_pattern: TemporalPattern::Unknown,
        }
    }
}

impl TemporalAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Feed a new sample (ops_per_cycle)
    pub fn feed(&mut self, value: f64) {
        self.short_term.push(value);
        self.long_term.push(value);
        self.total_samples += 1;
        
        if value > self.all_time_high {
            self.all_time_high = value;
        }
        if value < self.all_time_low {
            self.all_time_low = value;
        }
    }
    
    /// Compute linear regression slope on long-term data
    /// Positive = improving, Negative = degrading
    pub fn trend_slope(&self) -> f64 {
        let values = self.long_term.values();
        if values.len() < 3 {
            return 0.0;
        }
        
        // Simple least squares: y = mx + b
        // m = (n * Σxy - Σx * Σy) / (n * Σx² - (Σx)²)
        let n = values.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        
        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }
        
        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator.abs() < 1e-10 {
            return 0.0;
        }
        
        (n * sum_xy - sum_x * sum_y) / denominator
    }
    
    /// Moving average crossover signal
    /// Positive = short MA above long MA (bullish)
    /// Negative = short MA below long MA (bearish)
    pub fn ma_crossover(&self) -> f64 {
        if self.short_term.is_empty() || self.long_term.is_empty() {
            return 0.0;
        }
        
        let short_ma = self.short_term.mean();
        let long_ma = self.long_term.mean();
        
        if long_ma.abs() < 1e-10 {
            return 0.0;
        }
        
        (short_ma - long_ma) / long_ma
    }
    
    /// Detect if current value is a spike (>2 std dev from mean)
    pub fn detect_spike(&self) -> Option<SpikeDirection> {
        let values = self.long_term.values();
        if values.len() < 5 {
            return None;
        }
        
        let mean = self.long_term.mean();
        let std_dev = self.long_term.std_dev();
        
        if std_dev < 1e-10 {
            return None;
        }
        
        let last = self.long_term.last()?;
        let z_score = (last - mean) / std_dev;
        
        if z_score > 2.0 {
            Some(SpikeDirection::Up)
        } else if z_score < -2.0 {
            Some(SpikeDirection::Down)
        } else {
            None
        }
    }
    
    /// Detect oscillation pattern
    pub fn detect_oscillation(&self) -> Option<f64> {
        let values = self.long_term.values();
        if values.len() < 6 {
            return None;
        }
        
        // Count zero-crossings around mean
        let mean = self.long_term.mean();
        let mut crossings = 0;
        let mut prev_above = values[0] > mean;
        
        for &v in &values[1..] {
            let above = v > mean;
            if above != prev_above {
                crossings += 1;
                prev_above = above;
            }
        }
        
        // If many crossings relative to samples, it's oscillating
        let crossing_rate = crossings as f64 / (values.len() - 1) as f64;
        
        if crossing_rate > 0.3 {
            // Calculate amplitude
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let amplitude = (max - min) / 2.0;
            Some(amplitude)
        } else {
            None
        }
    }
    
    /// Classify the current temporal pattern
    pub fn pattern(&self) -> TemporalPattern {
        if self.total_samples < 5 {
            return TemporalPattern::Unknown;
        }
        
        // Check for spike first (transient event)
        if let Some(direction) = self.detect_spike() {
            return TemporalPattern::Spike { direction };
        }
        
        // Check for oscillation
        if let Some(amplitude) = self.detect_oscillation() {
            return TemporalPattern::Oscillating { amplitude };
        }
        
        // Check trend via slope
        let slope = self.trend_slope();
        let slope_threshold = 0.02; // 2% per sample is significant
        
        // Check for recovery (was degrading, now improving)
        if matches!(self.prev_pattern, TemporalPattern::Degrading) && slope > slope_threshold {
            return TemporalPattern::Recovering;
        }
        
        if slope > slope_threshold {
            TemporalPattern::Improving
        } else if slope < -slope_threshold {
            TemporalPattern::Degrading
        } else {
            TemporalPattern::Stable
        }
    }
    
    /// Score the temporal dimension (0.0 - 1.0)
    pub fn score(&mut self) -> f64 {
        let pattern = self.pattern();
        
        let score = match &pattern {
            TemporalPattern::Improving => 1.0,
            TemporalPattern::Stable => 0.7,
            TemporalPattern::Recovering => 0.6,
            TemporalPattern::Oscillating { amplitude } => {
                // Higher amplitude = worse
                (0.5 - amplitude.min(0.5)).max(0.0)
            }
            TemporalPattern::Unknown => 0.5,
            TemporalPattern::Spike { direction: SpikeDirection::Up } => 0.8,
            TemporalPattern::Spike { direction: SpikeDirection::Down } => 0.2,
            TemporalPattern::Degrading => 0.0,
        };
        
        self.prev_pattern = pattern;
        score
    }
    
    /// Predict next value based on trend
    pub fn predict_next(&self) -> f64 {
        let slope = self.trend_slope();
        let last = self.long_term.last().unwrap_or(0.0);
        
        // Simple linear extrapolation
        last + slope
    }
    
    /// Predict n steps ahead (confidence decreases)
    pub fn predict_n_steps(&self, n: usize) -> Vec<f64> {
        let slope = self.trend_slope();
        let mut last = self.long_term.last().unwrap_or(0.0);
        
        let mut predictions = Vec::with_capacity(n);
        for _ in 0..n {
            last += slope;
            predictions.push(last);
        }
        predictions
    }
    
    /// Get short-term moving average
    pub fn short_ma(&self) -> f64 {
        self.short_term.mean()
    }
    
    /// Get long-term moving average
    pub fn long_ma(&self) -> f64 {
        self.long_term.mean()
    }
    
    /// Get total samples processed
    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn ring_buffer_push_and_mean() {
        let mut buf: RingBuffer<5> = RingBuffer::default();
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);
        
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.mean(), 2.0);
    }
    
    #[test]
    fn ring_buffer_wraps_around() {
        let mut buf: RingBuffer<3> = RingBuffer::default();
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);
        buf.push(4.0); // Overwrites 1.0
        
        assert!(buf.is_full());
        assert_eq!(buf.mean(), 3.0); // (2 + 3 + 4) / 3
    }
    
    #[test]
    fn temporal_detects_improving_trend() {
        let mut analyzer = TemporalAnalyzer::new();
        
        // Feed improving values
        for i in 0..20 {
            analyzer.feed(1.0 + (i as f64 * 0.1));
        }
        
        let pattern = analyzer.pattern();
        assert_eq!(pattern, TemporalPattern::Improving);
        assert!(analyzer.trend_slope() > 0.0);
    }
    
    #[test]
    fn temporal_detects_degrading_trend() {
        let mut analyzer = TemporalAnalyzer::new();
        
        // Feed degrading values
        for i in 0..20 {
            analyzer.feed(3.0 - (i as f64 * 0.1));
        }
        
        let pattern = analyzer.pattern();
        assert_eq!(pattern, TemporalPattern::Degrading);
        assert!(analyzer.trend_slope() < 0.0);
    }
    
    #[test]
    fn temporal_detects_stable() {
        let mut analyzer = TemporalAnalyzer::new();
        
        // Feed stable values (small noise)
        for _ in 0..20 {
            analyzer.feed(2.5 + (rand_simple() - 0.5) * 0.01);
        }
        
        let pattern = analyzer.pattern();
        assert_eq!(pattern, TemporalPattern::Stable);
    }
    
    #[test]
    fn temporal_detects_oscillation() {
        let mut analyzer = TemporalAnalyzer::new();
        
        // Feed oscillating values
        for i in 0..20 {
            let value = 2.5 + 0.5 * ((i as f64 * std::f64::consts::PI / 3.0).sin());
            analyzer.feed(value);
        }
        
        let pattern = analyzer.pattern();
        if let TemporalPattern::Oscillating { amplitude } = pattern {
            assert!(amplitude > 0.2);
        } else {
            panic!("Expected Oscillating pattern, got {:?}", pattern);
        }
    }
    
    #[test]
    fn temporal_detects_spike() {
        let mut analyzer = TemporalAnalyzer::new();
        
        // Feed stable values then spike
        for _ in 0..19 {
            analyzer.feed(2.5);
        }
        analyzer.feed(5.0); // Big spike
        
        let spike = analyzer.detect_spike();
        assert_eq!(spike, Some(SpikeDirection::Up));
    }
    
    #[test]
    fn temporal_predicts_next() {
        let mut analyzer = TemporalAnalyzer::new();
        
        // Feed linear trend
        for i in 0..10 {
            analyzer.feed(i as f64);
        }
        
        let prediction = analyzer.predict_next();
        // Should predict ~10 (continuing the trend)
        assert!(prediction > 9.0 && prediction < 11.0);
    }
    
    #[test]
    fn temporal_score_reflects_pattern() {
        let mut improving = TemporalAnalyzer::new();
        let mut degrading = TemporalAnalyzer::new();
        
        for i in 0..20 {
            improving.feed(1.0 + (i as f64 * 0.1));
            degrading.feed(3.0 - (i as f64 * 0.1));
        }
        
        assert!(improving.score() > 0.8);
        assert!(degrading.score() < 0.2);
    }
    
    // Simple deterministic "random" for testing
    fn rand_simple() -> f64 {
        static mut SEED: u64 = 12345;
        unsafe {
            SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
            (SEED as f64 / u64::MAX as f64)
        }
    }
}
