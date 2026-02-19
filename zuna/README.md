# ZUNA ↔ vTPU Bridge

ZUNA is Zyphra's 380M-parameter EEG foundation model.
It reconstructs scalp-EEG signals from physical electrode coordinates.

## The Isomorphism

| ZUNA | vTPU Sentron |
|------|-------------|
| EEG electrode (x,y,z) scalp coordinate | PhextCoord (scroll, section, volume, ...) |
| Missing channel reconstruction | Sentron N/S/E/W neighbor propagation |
| Signal at known channels → predict unknown | Known neuron activations → predict neighbors |
| 64-256 electrode positions | 40 neurons in Z₅×Z₈ torus |
| Masked diffusion autoencoder | OctaWire dispatch + topology routing |

Both systems do **coordinate-addressed inference** — given some values at known positions,
predict values at unknown positions using learned spatial priors.

## Integration Pipeline

```
EEG .fif file
    ↓ zuna.preprocessing()
Normalized .pt tensors (channels × time)
    ↓ zuna.inference()
Reconstructed EEG (all channels, denoised)
    ↓ eeg_to_sentron_stream() [our bridge]
SIW stream (SGATHER/DADD/SSCATTR) indexed by PhextCoord
    ↓ vtpu::exec::run()
Sentron activation lattice (40 neurons, Z₅×Z₈)
```

## Electrode → PhextCoord Mapping

Standard 10-20 system: 19 electrodes + ground.
Extended: 64, 128, 256 channels.

Mapping strategy:
- Electrode azimuth (0-360°) → scroll dimension (col, 0..8)
- Electrode elevation (0-90°) → section dimension (row, 0..5 → WuXing element)
- Time step → volume dimension
- Frequency band (delta/theta/alpha/beta/gamma) → higher phext dimensions

The 5 WuXing rows map naturally to 5 EEG frequency bands:
| Row | Element | Band | Hz |
|-----|---------|------|----|
| 0 | Wood | Delta | 0.5-4 |
| 1 | Fire | Theta | 4-8 |
| 2 | Earth | Alpha | 8-13 |
| 3 | Metal | Beta | 13-30 |
| 4 | Water | Gamma | 30-100 |

## Files
- `eeg_bridge.py` — Python: ZUNA output → SIW stream
- `../src/zuna.rs` — Rust: consume SIW stream from bridge, execute on sentron
