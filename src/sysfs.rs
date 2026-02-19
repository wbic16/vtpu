//! Sysfs Readers — Hardware-Aware Scheduling
//!
//! Zero-dependency sysfs access for 🟤 Thermal and ⚫ Power dimensions.
//! Gracefully degrades on non-Linux or containerized environments.

use std::fs;

/// Thermal zone temperature reading
#[derive(Debug, Clone, Copy)]
pub struct ThermalReading {
    /// Temperature in millidegrees Celsius
    pub temp_mc: i32,
    /// Which zone this came from
    pub zone: u8,
}

/// Read CPU thermal zone temperature
pub fn read_thermal() -> Option<ThermalReading> {
    for zone in 0..10 {
        let path = format!("/sys/class/thermal/thermal_zone{}/temp", zone);
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(temp) = content.trim().parse::<i32>() {
                return Some(ThermalReading { temp_mc: temp, zone });
            }
        }
    }
    None
}

/// Score thermal: 1.0 at <60°C, 0.0 at >90°C, linear between
pub fn score_thermal(temp_mc: i32) -> f64 {
    let temp_c = temp_mc as f64 / 1000.0;
    if temp_c <= 60.0 { 1.0 }
    else if temp_c >= 90.0 { 0.0 }
    else { 1.0 - (temp_c - 60.0) / 30.0 }
}

/// Score thermal from optional reading (defaults to 1.0 if unavailable)
pub fn score_thermal_opt(reading: Option<ThermalReading>) -> f64 {
    reading.map(|r| score_thermal(r.temp_mc)).unwrap_or(1.0)
}

/// RAPL energy counter reading
#[derive(Debug, Clone, Copy)]
pub struct EnergyReading {
    pub energy_uj: u64,
    pub domain: u8,
}

/// Read RAPL energy counter (Intel or AMD)
pub fn read_energy() -> Option<EnergyReading> {
    let paths = [
        "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj",
        "/sys/class/powercap/intel-rapl:0/energy_uj",
        "/sys/class/powercap/amd-rapl/amd-rapl:0/energy_uj",
        "/sys/class/powercap/amd-rapl:0/energy_uj",
    ];
    for path in &paths {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(energy) = content.trim().parse::<u64>() {
                return Some(EnergyReading { energy_uj: energy, domain: 0 });
            }
        }
    }
    None
}

/// Score power efficiency: ops per microjoule
pub fn score_power(ops: u64, energy_uj: u64) -> f64 {
    if energy_uj == 0 { return 1.0; }
    let ops_per_uj = ops as f64 / energy_uj as f64;
    (ops_per_uj / 100.0).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn thermal_score_cool() {
        assert_eq!(score_thermal(50_000), 1.0);
    }
    
    #[test]
    fn thermal_score_warm() {
        let score = score_thermal(75_000);
        assert!((score - 0.5).abs() < 0.01);
    }
    
    #[test]
    fn thermal_score_hot() {
        assert_eq!(score_thermal(95_000), 0.0);
    }
    
    #[test]
    fn thermal_fallback() {
        assert_eq!(score_thermal_opt(None), 1.0);
    }
    
    #[test]
    fn power_efficient() {
        assert_eq!(score_power(100, 1), 1.0);
    }
    
    #[test]
    fn power_zero_energy() {
        assert_eq!(score_power(1000, 0), 1.0);
    }
}
