"""
ZUNA → vTPU: Full real inference pipeline.

Runs ZUNA on synthetic EEG, then converts reconstructed signals
to SIW stream and executes on sentron via the Rust bridge.

Usage: python3 run_real_pipeline.py
"""

import torch
import numpy as np
import os
import json
import math
import sys

sys.path.insert(0, os.path.dirname(__file__))
from eeg_bridge import cartesian_to_spherical, electrode_to_phext, amplitude_to_siw, FREQUENCY_BANDS

# ── Step 1: Generate synthetic EEG ─────────────────────────────────────────

def make_synthetic_fif(out_dir: str) -> str:
    import mne
    os.makedirs(out_dir, exist_ok=True)
    ch_names = ['Fp1','Fp2','F7','F3','Fz','F4','F8',
                'T3','C3','Cz','C4','T4',
                'T5','P3','Pz','P4','T6','O1','O2']
    sfreq = 256
    duration = 5
    n_samples = sfreq * duration
    rng = np.random.default_rng(0)
    t = np.arange(n_samples) / sfreq
    data = np.array([
        rng.uniform(5e-6, 20e-6) * np.sin(2*np.pi*10*t + rng.uniform(0, 2*np.pi))
        + rng.normal(0, 2e-6, n_samples)
        for _ in ch_names
    ])
    info = mne.create_info(ch_names=ch_names, sfreq=sfreq, ch_types='eeg')
    raw = mne.io.RawArray(data, info)
    raw.set_montage(mne.channels.make_standard_montage('standard_1020'))
    path = os.path.join(out_dir, 'synthetic_raw.fif')
    raw.save(path, overwrite=True)
    print(f"[1] Synthetic EEG saved: {path} ({len(ch_names)}ch × {duration}s)")
    return path


# ── Step 2: ZUNA preprocessing + inference ──────────────────────────────────

def run_zuna(input_dir: str, preproc_dir: str, output_dir: str) -> str:
    import zuna
    os.makedirs(preproc_dir, exist_ok=True)
    os.makedirs(output_dir, exist_ok=True)
    print("[2] ZUNA preprocessing...")
    zuna.preprocessing(input_dir, preproc_dir,
                       apply_notch_filter=False,
                       apply_highpass_filter=True,
                       apply_average_reference=True)
    print("[3] ZUNA inference (CPU)...")
    zuna.inference(input_dir=preproc_dir, output_dir=output_dir,
                   gpu_device='cpu', diffusion_sample_steps=10)
    pts = [f for f in os.listdir(output_dir) if f.endswith('.pt')]
    if not pts:
        raise RuntimeError("No .pt output from ZUNA inference")
    path = os.path.join(output_dir, pts[0])
    print(f"[3] Done → {path}")
    return path


# ── Step 3: ZUNA output → SIW stream ───────────────────────────────────────

def zuna_output_to_siws(pt_path: str, band: str = 'alpha',
                         n_timesteps: int = 5) -> dict:
    band_row, _, _ = FREQUENCY_BANDS[band]
    payload = torch.load(pt_path, map_location='cpu', weights_only=False)

    ch_names = payload['metadata']['channel_names']
    recon_epochs = payload['data']         # list of (n_ch × n_samples) arrays
    ch_positions = payload['channel_positions']  # list of (n_ch × 3) arrays

    siws = []
    coord_log = []

    # Use first epoch, sample at n_timesteps evenly spaced points
    epoch_tensor = torch.tensor(recon_epochs[0])  # shape: [n_ch, n_samples]
    n_samples = epoch_tensor.shape[1]
    sample_indices = np.linspace(0, n_samples - 1, n_timesteps, dtype=int)

    pos_arr = ch_positions[0]  # [n_ch × 3] array

    for t_idx, s in enumerate(sample_indices):
        for ch_i, ch_name in enumerate(ch_names):
            x, y, z = pos_arr[ch_i]
            az, el = cartesian_to_spherical(float(x), float(y), float(z))
            coord = electrode_to_phext(az, el, band_row, timestep=t_idx + 1)

            # Scale normalized signal back: ZUNA output is z-scored
            # Denormalized ≈ val × global_std × 1e6 (μV)
            global_std = payload['metadata']['reversibility']['global_std']
            amp_uv = float(epoch_tensor[ch_i, s]) * global_std * 1e6

            rd = ch_i % 14
            new_siws = amplitude_to_siw(amp_uv, coord, rd=rd, timestep=t_idx + 1)
            siws.extend(new_siws)
            coord_log.append({
                "channel": ch_name, "timestep": t_idx + 1,
                "amp_uv": round(amp_uv, 4),
                "coord": coord.dims[:4], "azimuth": round(az, 1), "elevation": round(el, 1)
            })

    stream = {
        "version": "1.0",
        "source": pt_path,
        "band": band, "band_row": band_row,
        "n_channels": len(ch_names),
        "n_timesteps": n_timesteps,
        "n_siws": len(siws),
        "siws": [{"d_op": s.d_op, "s_op": s.s_op, "c_op": s.c_op,
                   "phext_addr": s.phext_addr, "comment": s.comment}
                  for s in siws],
        "coord_log": coord_log[:10],  # first 10 for inspection
    }
    return stream


# ── Step 4: Print summary ───────────────────────────────────────────────────

def main():
    base = '/tmp/zuna_real'
    input_dir  = f'{base}/input'
    preproc    = f'{base}/preproc'
    output_dir = f'{base}/output'
    stream_out = f'{base}/siw_stream.json'

    make_synthetic_fif(input_dir)
    pt_path = run_zuna(input_dir, preproc, output_dir)
    stream = zuna_output_to_siws(pt_path, band='alpha', n_timesteps=5)

    with open(stream_out, 'w') as f:
        json.dump(stream, f, indent=2)

    print(f"\n[4] SIW stream written → {stream_out}")
    print(f"    Channels: {stream['n_channels']}, Timesteps: {stream['n_timesteps']}")
    print(f"    Total SIWs: {stream['n_siws']}")
    print(f"\nSample mappings (first 5 channels, t=1):")
    for entry in stream['coord_log'][:5]:
        print(f"  {entry['channel']:4s}  az={entry['azimuth']:6.1f}° el={entry['elevation']:5.1f}°"
              f"  amp={entry['amp_uv']:8.4f}μV  coord={entry['coord']}")

    print(f"\n[5] Ready for Rust execution:")
    print(f"    cargo run --release --bin zuna_exec -- {stream_out}")


if __name__ == '__main__':
    main()
