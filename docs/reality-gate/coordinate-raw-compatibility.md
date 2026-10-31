# Coordinate raw layout and allocation compatibility (RG-1B)

Status: **canonical encoding pinned; legacy persisted-byte migration is NOT validated**.

## Canonical 128-bit raw encoding

`PhextCoord::as_raw()` returns `(lo: u64, hi: u64)`. The logical packed bitstream places
11 dimensions of 11 bits each at bit offsets 0, 11, ..., 110, followed by 7 flag bits
at offsets 121..127. In `hi`, dimension 10 occupies bits 46..56 and flags
occupy bits 57..63. The test `canonical_raw_coordinate_encoding_is_stable`
pins a literal raw tuple with a high dimension-10 bit and nonzero flags.
This specifies bit positions, not a cross-platform serialized byte order.
Any on-disk representation must also explicitly version its endianness.

## Older raw coordinates: not an automatic migration

The previous `set_flags` implementation wrote `hi[55..61]`, overlapping
dimension 10 in `hi[55..56]`. Writes through this old setter could destroy
dimension-10 bits even when setting flags to zero. Consequently, a bare
legacy `(lo, hi)` with overlapping bits is **not always invertible**;
the intended original coordinate cannot be reconstructed from those two
words alone. No validated inventory of existing persisted raw-coordinate
records is available. Do **not** assert that no legacy data exists, do **not**
reinterpret legacy bytes silently, and do **not** infer a corrected coordinate
from the raw words without external version/provenance metadata.

Before accepting legacy persistent data: inventory the storage and producers;
identify the encoding version per record or collection; distinguish untouched
legacy dimensions from old `set_flags` writes where possible; recover ambiguous
coordinates from authoritative external metadata or quarantine them; migrate to
a versioned canonical format and verify roundtrip and 11D memory/replay identity.
The `as_raw` tuple regression is an encoding guard, not a migration utility.

## Region count and performance comparability

The region key now includes the upper four bits of **each** of the three
inner dimensions, yielding up to `16^3 = 4096` distinct regions for each
fixed outer-coordinate tuple where the old projection merged them.
Each region reserves `2^21 * 8 = 16 MiB` of logical physical address
space: fully populating those 4096 regions accounts for **64 GiB** of
logical address range. This is not a measured resident-memory or throughput
claim. Previously truncated outer dimensions are now independently mapped too.

Thus `PPTStats.regions_allocated`, `bytes_allocated`,
`EpochPPTStats.working_regions`, and `working_allocated` must not be
compared numerically with pre-fix values as equivalent workloads. The flat
`Memory` implementation can grow substantially for sparse, widely spaced
coordinates; separate sparse backing storage and performance gates remain open.

`EpochPPT` currently hard-codes `inner_page_size = 1 << 21` and has no
configurable page-size constructor. The `PhextPageTable::with_config` minimum
size guard prevents future undersized pages in the configurable path; if
`EpochPPT` gains configurable pages, it must enforce the same invariant.
