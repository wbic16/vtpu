"""
ZUNA → vTPU Bridge
Converts ZUNA's reconstructed EEG output into SIW instruction streams
addressable by PhextCoord, suitable for sentron execution.

Usage:
    python eeg_bridge.py input.fif --output stream.json

Pipeline:
    1. Run ZUNA preprocessing + inference on EEG file
    2. Map electrode coordinates → PhextCoord
    3. Map signal amplitude → DADD/DMOV register operations
    4. Emit SIW stream as JSON (consumed by vtpu via zuna.rs)
"""

import json
import math
import sys
import argparse
from dataclasses import dataclass, asdict
from typing import Optional

# Try importing zuna; graceful fallback for dev without model weights
try:
    import zuna
    ZUNA_AVAILABLE = True
except ImportError:
    ZUNA_AVAILABLE = False
    print("[bridge] zuna not installed — running in stub mode", file=sys.stderr)

# Try importing MNE for EEG coordinate reading
try:
    import mne
    MNE_AVAILABLE = True
except ImportError:
    MNE_AVAILABLE = False

# ── Frequency band → WuXing row mapping ────────────────────────────────────

FREQUENCY_BANDS = {
    "delta": (0,   0.5,  4.0),   # Wood row 0
    "theta": (1,   4.0,  8.0),   # Fire row 1
    "alpha": (2,   8.0, 13.0),   # Earth row 2
    "beta":  (3,  13.0, 30.0),   # Metal row 3
    "gamma": (4,  30.0,100.0),   # Water row 4
}

# ── Coordinate mapping ──────────────────────────────────────────────────────

@dataclass
class PhextCoord:
    """11-dimensional phext coordinate (dimensions 0-10, 1-indexed)."""
    dims: list  # 11 values, each 1..999

    @staticmethod
    def zero():
        return PhextCoord([1]*11)

    def to_dict(self):
        return {"dims": self.dims}


def electrode_to_phext(azimuth_deg: float, elevation_deg: float,
                        band_row: int, timestep: int = 1) -> PhextCoord:
    """
    Map a 3D electrode position + frequency band + time → PhextCoord.

    Dimension layout:
      dim 0 (scroll)   = column from azimuth (1..8 → 8 lateral sentron positions)
      dim 1 (section)  = row from elevation  (1..5 → 5 WuXing elements)
      dim 2 (volume)   = frequency band row  (1..5)
      dim 3 (chapter)  = timestep (1-indexed)
      dims 4-10        = 1 (reserved for future spatial/session dimensions)
    """
    # Normalize azimuth 0-360° → column 1..8
    col = max(1, min(8, int((azimuth_deg % 360) / 360 * 8) + 1))
    # Normalize elevation 0-90° → row 1..5
    row = max(1, min(5, int(elevation_deg / 90 * 5) + 1))
    dims = [1]*11
    dims[0] = col           # scroll = lateral position
    dims[1] = row           # section = depth position
    dims[2] = band_row + 1  # volume = frequency band (1-5)
    dims[3] = max(1, min(999, timestep))  # chapter = time
    return PhextCoord(dims)


def cartesian_to_spherical(x: float, y: float, z: float):
    """Convert 3D cartesian scalp position to (azimuth, elevation) in degrees."""
    r = math.sqrt(x*x + y*y + z*z)
    if r < 1e-9:
        return 0.0, 0.0
    azimuth   = math.degrees(math.atan2(y, x)) % 360
    elevation = math.degrees(math.asin(max(-1, min(1, z / r))))
    return azimuth, elevation


# ── SIW emission ────────────────────────────────────────────────────────────

@dataclass
class SIWRecord:
    """Simplified SIW for JSON serialization."""
    d_op: str          # "DMOV rd imm" | "DADD rd rs1 rs2" | "DNOP"
    s_op: str          # "SSCATTR coord_idx rs width" | "SGATHER rd coord_idx width" | "SNOP"
    c_op: str          # "CNOP"
    phext_addr: dict   # PhextCoord.to_dict()
    comment: str = ""


def amplitude_to_siw(amplitude: float, coord: PhextCoord,
                      rd: int = 0, timestep: int = 1) -> list:
    """
    Convert a scalar EEG amplitude into a 2-SIW sequence:
      1. DMOV rd, <scaled_amplitude>   — load amplitude into register
      2. SSCATTR coord_idx=0, rs=rd    — scatter to phext coord
    """
    # Scale: EEG amplitude typically ±200 μV; scale to i64 range
    scaled = int(amplitude * 1000)  # μV × 1000 → integer nanotesla-equivalent
    scaled = max(-32768, min(32767, scaled))  # clip to i16 range

    load_siw = SIWRecord(
        d_op=f"DMOV {rd} {scaled}",
        s_op="SNOP",
        c_op="CNOP",
        phext_addr=coord.to_dict(),
        comment=f"Load amplitude {amplitude:.3f}μV t={timestep}",
    )
    scatter_siw = SIWRecord(
        d_op="DNOP",
        s_op=f"SSCATTR 0 {rd} 8",
        c_op="CNOP",
        phext_addr=coord.to_dict(),
        comment=f"Scatter to {coord.dims[:4]}",
    )
    return [load_siw, scatter_siw]


# ── Main pipeline ───────────────────────────────────────────────────────────

def stub_eeg_data():
    """
    Generate synthetic EEG-like data for testing without a real .fif file.
    Returns list of (electrode_name, x, y, z, amplitude_mv) tuples.
    Standard 10-20 system subset: 19 electrodes.
    """
    # Approximate 3D positions (unit sphere, head-centered)
    electrodes = [
        # name,        x,      y,      z,      amp_uV
        ("Fp1",   -0.31,  0.54,  0.78,  12.5),
        ("Fp2",    0.31,  0.54,  0.78,  11.2),
        ("F7",    -0.71,  0.40,  0.57,   8.3),
        ("F3",    -0.46,  0.39,  0.79,   9.1),
        ("Fz",     0.00,  0.40,  0.92,  10.4),
        ("F4",     0.46,  0.39,  0.79,   8.8),
        ("F8",     0.71,  0.40,  0.57,   7.6),
        ("T3",    -0.95,  0.00,  0.29,  15.2),
        ("C3",    -0.58,  0.00,  0.82,  13.1),
        ("Cz",     0.00,  0.00,  1.00,  16.7),
        ("C4",     0.58,  0.00,  0.82,  14.3),
        ("T4",     0.95,  0.00,  0.29,  15.8),
        ("T5",    -0.71, -0.40,  0.57,  11.9),
        ("P3",    -0.46, -0.39,  0.79,  10.5),
        ("Pz",     0.00, -0.40,  0.92,  12.3),
        ("P4",     0.46, -0.39,  0.79,  11.1),
        ("T6",     0.71, -0.40,  0.57,  10.7),
        ("O1",    -0.31, -0.54,  0.78,   9.4),
        ("O2",     0.31, -0.54,  0.78,   8.9),
    ]
    return electrodes


def run_bridge(fif_path: Optional[str], output_path: str,
               band: str = "alpha", n_timesteps: int = 10):
    """Main bridge function. Reads EEG (or stubs), emits SIW stream JSON."""

    band_row, band_lo, band_hi = FREQUENCY_BANDS.get(band, FREQUENCY_BANDS["alpha"])
    print(f"[bridge] Band: {band} ({band_lo}-{band_hi} Hz) → WuXing row {band_row}")

    siws = []

    if fif_path and ZUNA_AVAILABLE and MNE_AVAILABLE:
        # Real pipeline: ZUNA preprocessing + inference
        print(f"[bridge] Processing {fif_path} with ZUNA...")
        preprocessed = zuna.preprocessing(fif_path)
        reconstructed = zuna.inference(preprocessed)

        raw = mne.io.read_raw_fif(fif_path, preload=True)
        ch_pos = raw.get_montage().get_positions()["ch_pos"]

        for ch_name, pos_3d in ch_pos.items():
            az, el = cartesian_to_spherical(*pos_3d)
            coord = electrode_to_phext(az, el, band_row, timestep=1)
            # TODO: extract reconstructed amplitude for this channel/time
            amp = 0.0  # placeholder
            siws.extend(amplitude_to_siw(amp, coord, rd=0))
    else:
        # Stub pipeline: synthetic 10-20 data
        if fif_path:
            print(f"[bridge] ZUNA/MNE not available — using stub data")
        else:
            print(f"[bridge] No input file — generating synthetic 10-20 EEG")

        electrodes = stub_eeg_data()
        for t in range(1, n_timesteps + 1):
            for i, (name, x, y, z, amp_uv) in enumerate(electrodes):
                az, el = cartesian_to_spherical(x, y, z)
                coord = electrode_to_phext(az, el, band_row, timestep=t)
                # Simulate mild alpha oscillation (8 Hz)
                import math
                oscillated = amp_uv * math.cos(2 * math.pi * 10 * t / 1000)
                rd = i % 14  # stay within general register range (0-15)
                siws.extend(amplitude_to_siw(oscillated, coord, rd=rd, timestep=t))

    stream = {
        "version": "1.0",
        "source": fif_path or "synthetic-10-20",
        "band": band,
        "band_row": band_row,
        "n_siws": len(siws),
        "siws": [asdict(s) for s in siws],
        "coordinate_mapping": {
            "dim_0": "scroll = azimuth column (1..8)",
            "dim_1": "section = elevation row / WuXing (1..5)",
            "dim_2": "volume = frequency band (1..5)",
            "dim_3": "chapter = timestep",
        },
        "wuxing_bands": {
            str(r): {"element": e, "band": bname, "hz": f"{lo}-{hi}"}
            for bname, e, (r, lo, hi) in [
                ("delta", "Wood",  FREQUENCY_BANDS["delta"]),
                ("theta", "Fire",  FREQUENCY_BANDS["theta"]),
                ("alpha", "Earth", FREQUENCY_BANDS["alpha"]),
                ("beta",  "Metal", FREQUENCY_BANDS["beta"]),
                ("gamma", "Water", FREQUENCY_BANDS["gamma"]),
            ]
        }
    }

    with open(output_path, "w") as f:
        json.dump(stream, f, indent=2)
    print(f"[bridge] Wrote {len(siws)} SIWs → {output_path}")
    return stream


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="ZUNA → vTPU SIW bridge")
    parser.add_argument("input", nargs="?", help=".fif EEG file (optional; stubs if omitted)")
    parser.add_argument("--output", default="eeg_stream.json", help="Output SIW stream JSON")
    parser.add_argument("--band", default="alpha",
                        choices=list(FREQUENCY_BANDS.keys()), help="Frequency band")
    parser.add_argument("--timesteps", type=int, default=10, help="Timesteps (stub mode)")
    args = parser.parse_args()

    stream = run_bridge(args.input, args.output, args.band, args.timesteps)
    print(f"\nSample SIWs (first 4 of {stream['n_siws']}):")
    for siw in stream["siws"][:4]:
        print(f"  {siw['d_op']:20s} | {siw['s_op']:25s} | coord={siw['phext_addr']['dims'][:4]}")
