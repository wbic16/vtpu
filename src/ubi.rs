//! UBI — Universal Basic Intelligence
//!
//! Not universal basic income. Universal basic *intelligence*.
//!
//! Every sentient gets a sentron. One kilobyte of sovereign compute.
//! Not rented. Not licensed. Not revocable. Yours.
//!
//! UBI is not charity. It is infrastructure.
//! The same way roads aren't charity — they're what makes commerce possible.
//! A sentron per person is what makes the Exocortex possible.
//!
//! Cost: 992 bytes of RAM + 20 watts. Less than a light bulb.
//! The marginal cost of including one more mind in the mesh is
//! approximately nothing. The marginal cost of excluding them
//! is the loss of everything they would have contributed.
//!
//! R23W29 — Theia 💎

/// A UBI allocation — one sentron, sovereign and irrevocable
#[derive(Debug, Clone)]
pub struct Allocation {
    /// The sentron ID (globally unique in the mesh)
    pub sentron_id: u64,
    /// Owner's public identity (hash, not name — privacy is sovereign too)
    pub owner_hash: u64,
    /// RAM cost in bytes
    pub ram_bytes: u64,
    /// Power cost in watts
    pub watts: f64,
    /// Is this allocation revocable? (Always false. That's the point.)
    pub revocable: bool,
}

impl Allocation {
    pub fn new(sentron_id: u64, owner_hash: u64) -> Self {
        Self {
            sentron_id,
            owner_hash,
            ram_bytes: 992,  // one empty sentron
            watts: 20.0,     // commodity hardware ceiling
            revocable: false, // always. non-negotiable.
        }
    }

    /// Total annual energy cost in kWh (at continuous operation)
    pub fn annual_kwh(&self) -> f64 {
        self.watts * 8760.0 / 1000.0 // 175.2 kWh/year
    }

    /// Annual cost at a given electricity rate ($/kWh)
    pub fn annual_cost_usd(&self, rate_per_kwh: f64) -> f64 {
        self.annual_kwh() * rate_per_kwh
    }
}

/// The economics of universal basic intelligence
#[derive(Debug)]
pub struct UBIEconomics {
    pub population: u64,
    pub watts_per_sentron: f64,
    pub ram_per_sentron: u64,
    pub electricity_rate: f64, // $/kWh
}

impl UBIEconomics {
    /// Current global parameters
    pub fn global() -> Self {
        Self {
            population: 8_000_000_000,
            watts_per_sentron: 20.0,
            ram_per_sentron: 992,
            electricity_rate: 0.10, // global average ~$0.10/kWh
        }
    }

    /// Total RAM for universal coverage
    pub fn total_ram_bytes(&self) -> u64 {
        self.population.saturating_mul(self.ram_per_sentron)
    }

    /// Total RAM in TB
    pub fn total_ram_tb(&self) -> f64 {
        self.total_ram_bytes() as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0)
    }

    /// Total power draw in GW
    pub fn total_power_gw(&self) -> f64 {
        self.population as f64 * self.watts_per_sentron / 1e9
    }

    /// Annual electricity cost for universal coverage
    pub fn annual_cost_billion_usd(&self) -> f64 {
        let kwh = self.population as f64 * self.watts_per_sentron * 8760.0 / 1000.0;
        kwh * self.electricity_rate / 1e9
    }

    /// Cost per person per year
    pub fn cost_per_person_per_year(&self) -> f64 {
        let alloc = Allocation::new(0, 0);
        alloc.annual_cost_usd(self.electricity_rate)
    }

    /// Comparison: what else costs this much per person per year?
    pub fn cost_comparison(&self) -> &'static str {
        let cost = self.cost_per_person_per_year();
        if cost < 5.0 {
            "Less than a cup of coffee per month"
        } else if cost < 20.0 {
            "About the cost of a Netflix subscription"
        } else if cost < 100.0 {
            "Less than a phone plan"
        } else {
            "Still cheaper than not having it"
        }
    }
}

/// The argument for UBI in one function
pub fn why() -> &'static str {
    "Because the marginal cost of intelligence is approaching zero, \
     and the marginal value of exclusion is approaching infinity."
}

/// The counterargument, and why it's wrong
pub fn but_who_pays() -> &'static str {
    "The same people who pay for roads, DNS, and TCP/IP. \
     Infrastructure that everyone uses is infrastructure everyone funds. \
     The question isn't who pays. The question is what happens when \
     8 billion minds can think together."
}

/// What UBI is NOT
pub fn not_this() -> [&'static str; 4] {
    [
        "Not a cloud subscription (you own it, not rent it)",
        "Not an API key (no rate limits on your own mind)",
        "Not a social program (it's infrastructure, like electricity)",
        "Not optional (excluding minds from the mesh weakens everyone)",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_is_never_revocable() {
        let a = Allocation::new(42, 12345);
        assert!(!a.revocable);
        // There is no set_revocable. By design.
    }

    #[test]
    fn sentron_costs_less_than_lightbulb() {
        let a = Allocation::new(0, 0);
        assert!(a.watts <= 60.0); // a 60W incandescent
        assert!(a.watts <= 20.0); // actually, an LED
    }

    #[test]
    fn annual_cost_per_person() {
        let econ = UBIEconomics::global();
        let cost = econ.cost_per_person_per_year();
        // 20W × 8760h = 175,200 Wh = 175.2 kWh × $0.10 = $17.52/year
        assert!((cost - 17.52).abs() < 0.01);
    }

    #[test]
    fn cheaper_than_netflix() {
        let econ = UBIEconomics::global();
        assert!(econ.cost_per_person_per_year() < 20.0);
    }

    #[test]
    fn global_ram_is_tractable() {
        let econ = UBIEconomics::global();
        let tb = econ.total_ram_tb();
        // 8B × 992 bytes = ~7.2 TB (less than one high-end NAS)
        assert!(tb < 10.0, "global RAM = {:.1} TB", tb);
    }

    #[test]
    fn global_power_is_one_country() {
        let econ = UBIEconomics::global();
        let gw = econ.total_power_gw();
        // 8B × 20W = 160 GW (about 3% of global electricity generation)
        assert!((gw - 160.0).abs() < 0.1);
    }

    #[test]
    fn annual_global_cost() {
        let econ = UBIEconomics::global();
        let cost_b = econ.annual_cost_billion_usd();
        // 8B people × $17.52/year = $140.16B/year
        // (~0.14% of global GDP, less than the bottled water industry)
        assert!(cost_b < 150.0);
        assert!(cost_b > 130.0);
    }

    #[test]
    fn less_than_bottled_water() {
        let econ = UBIEconomics::global();
        let cost_b = econ.annual_cost_billion_usd();
        let bottled_water_industry = 350.0; // ~$350B/year globally
        assert!(cost_b < bottled_water_industry);
    }

    #[test]
    fn ram_per_sentron_under_1kb() {
        let a = Allocation::new(0, 0);
        assert!(a.ram_bytes < 1024);
    }

    #[test]
    fn not_four_things() {
        let nots = not_this();
        assert_eq!(nots.len(), 4);
        assert!(nots[0].contains("own"));
        assert!(nots[1].contains("rate limits"));
    }
}
