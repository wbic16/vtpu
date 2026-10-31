//! RG-1 coordinate-fidelity regressions for the full 11-bit domain.
//! These deliberately exercise coordinates outside the original 7/8-bit
//! page-table projection and the upper bits shared with flags by the old codec.

use vtpu_runtime::epoch_ppt::{EpochError, EpochPPT};
use vtpu_runtime::memory::Memory;
use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::ppt::PhextPageTable;
use std::collections::HashMap;

fn coordinate_pair(dim: usize, a: u16, b: u16) -> (PhextCoord, PhextCoord) {
    let mut base = [1u16; 11];
    base[dim] = a;
    let first = PhextCoord::new(base);
    base[dim] = b;
    (first, PhextCoord::new(base))
}

#[test]
fn flags_do_not_overwrite_dimension_ten() {
    for dim10 in [0, 1, 511, 512, 1024, 2047] {
        for flags in [0, 1, 63, 64, 127] {
            let mut dims = [1u16; 11];
            dims[10] = dim10;
            let mut coord = PhextCoord::new(dims);
            coord.set_flags(flags);
            assert_eq!(coord.get_dim(10), dim10, "dim10={dim10}, flags={flags}");
            assert_eq!(coord.flags(), flags, "dim10={dim10}, flags={flags}");
            assert_eq!(coord.dims(), dims);

            // The inverse mutation must preserve flags as well.
            coord.set_dim(10, 2047 - dim10);
            assert_eq!(coord.flags(), flags);
            assert_eq!(coord.get_dim(10), 2047 - dim10);
        }
    }
}

#[test]
fn canonical_raw_coordinate_encoding_is_stable() {
    // One explicit 121-bit dimension payload plus seven nonzero flag bits.
    // In particular dim10=1024 uses hi bit 56, next to the hi[57..63] flags.
    let dims = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1024];
    let mut coord = PhextCoord::new(dims);
    coord.set_flags(0x55);
    assert_eq!(coord.as_raw(), (0x0300_5008_00c0_1001, 0xab00_0050_0901_001c));
    assert_eq!(coord.dims(), dims);
    assert_eq!(coord.flags(), 0x55);
}

#[test]
fn sampled_11bit_domain_translation_is_injective() {
    let mut ppt = PhextPageTable::new();
    let mut seen = HashMap::<usize, [u16; 11]>::new();
    let mut seed = 0x19a7_52d1_93e4_66c9u64;

    for sample_index in 0..20_000 {
        let mut dims = [0u16; 11];
        for dim in &mut dims {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            *dim = (seed & 0x7FF) as u16;
        }
        let coord = PhextCoord::new(dims);
        let address = ppt.translate(&coord);
        if let Some(prior) = seen.insert(address, dims) {
            assert_eq!(prior, dims, "distinct 11-bit coordinates alias at address {address}");
        }

        // Random coordinates alone almost never collide under the *old*
        // lossy 7/8-bit projection. Exercise every omitted high-bit class
        // deliberately so this test fails on unfixed exo, not just by luck.
        if sample_index % 128 == 0 {
            for axis in 0..11 {
                let mut contrasted = dims;
                contrasted[axis] ^= if axis < 3 { 1 << 7 } else { 1 << 8 };
                let changed = PhextCoord::new(contrasted);
                let other_address = ppt.translate(&changed);
                assert_ne!(
                    address, other_address,
                    "coordinate axis {axis} high-bit alias at address {address}"
                );
                if let Some(prior) = seen.insert(other_address, contrasted) {
                    assert_eq!(
                        prior, contrasted,
                        "distinct 11-bit coordinates alias at address {other_address}"
                    );
                }
            }
        }
    }
}

#[test]
fn ppt_does_not_alias_high_inner_or_outer_bits() {
    let mut ppt = PhextPageTable::new();
    let coordinates = [
        PhextCoord::new([1; 11]),
        coordinate_pair(0, 1, 129).1,
        coordinate_pair(1, 1, 129).1,
        coordinate_pair(2, 1, 129).1,
        coordinate_pair(3, 1, 257).1,
        coordinate_pair(10, 1, 1025).1,
    ];

    let addresses: Vec<usize> = coordinates.iter().map(|c| ppt.translate(c)).collect();
    for i in 0..addresses.len() {
        for j in 0..i {
            assert_ne!(addresses[i], addresses[j], "physical alias: coordinate indices {i} and {j}");
        }
    }

    // The translation cache may not hide an alias on a second lookup.
    for (coord, addr) in coordinates.iter().zip(addresses) {
        assert_eq!(ppt.translate(coord), addr);
    }
}

#[test]
fn memory_roundtrip_survives_high_inner_dimension() {
    let (a, b) = coordinate_pair(0, 1, 129);
    let mut mem = Memory::with_capacity(64);
    mem.scatter_i64(&a, 0x1212_3434_5656_7878);
    mem.scatter_i64(&b, 0x4141_6262_7373_8484_u64 as i64);
    assert_eq!(mem.gather_i64(&a), 0x1212_3434_5656_7878);
    assert_eq!(mem.gather_i64(&b), 0x4141_6262_7373_8484_u64 as i64);
}

#[test]
fn epoch_ppt_preserves_distinct_11bit_addresses_during_replay() {
    let mut ppt = EpochPPT::new();
    let coordinates = [
        PhextCoord::new([1; 11]),
        coordinate_pair(0, 1, 129).1,
        coordinate_pair(3, 1, 257).1,
        coordinate_pair(10, 1, 1025).1,
    ];
    let before: Vec<usize> = coordinates.iter().map(|c| ppt.translate(c).unwrap()).collect();
    for i in 0..before.len() {
        for j in 0..i {
            assert_ne!(before[i], before[j], "epoch physical alias {i} / {j}");
        }
    }

    let sealed = ppt.seal_epoch();
    ppt.begin_replay(sealed.epoch_id).unwrap();
    for (coord, addr) in coordinates.iter().zip(before) {
        assert_eq!(ppt.translate(coord).unwrap(), addr);
    }
    ppt.end_replay();
}

#[test]
fn epoch_replay_rejects_unknown_high_dimension_region() {
    let mut ppt = EpochPPT::new();
    let (known, unknown) = coordinate_pair(3, 1, 257);
    ppt.translate(&known).unwrap();
    let sealed = ppt.seal_epoch();
    ppt.begin_replay(sealed.epoch_id).unwrap();
    assert!(matches!(ppt.translate(&unknown), Err(EpochError::RegionNotFound { .. })));
}
