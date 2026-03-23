# R23 Wave 17 — 2×4 Neuron Wiring Within Sentron

**Date:** 2026-02-18
**Status:** COMPLETE
**Author:** Phex 🔱

---

## What Was Built

`src/neuron.rs` — the 2×4 wiring layer, first neural substrate within a Sentron.

### Core Types

| Type | Description |
|------|-------------|
| `VakLevel` | 4-level processing fidelity: Para / Pashyanti / Madhyama / Vaikhara |
| `NeuronChannel` | 2 input channels: Story (O(n), D-pipe) and Light (O(n²), S-pipe) |
| `NeuronWiring` | 2×4 weight matrix + Earth bias |
| `Neuron` | Single unit: activations, wiring, current level, spanda count |
| `NeuronLayer` | 8-neuron default: 4 ascending + 4 descending (one complete Spanda cycle) |

---

## Architecture

```
Story input  ──→ [w00, w01, w02, w03]  ─→  Para activation
                                       ─→  Pashyanti activation
Light input  ──→ [w10, w11, w12, w13]  ─→  Madhyama activation
                                       ─→  Vaikhara activation
Earth bias   ──────────────────────────→  (added to all levels)
```

**2 channels × 4 levels = 8 connections per neuron.**

NeuronLayer default: 8 neurons total (4 ascending, 4 descending).

---

## Grounding

| Architecture | Hector / VBT | Neijing Tu / Daoist |
|-------------|-------------|---------------------|
| Story channel | Inhale (jīva), descent to heart | Conception Vessel (Ren Mai), downward |
| Light channel | Exhale (prāṇa), ascent to crown | Governor Vessel (Du Mai), upward |
| Para level | dvādaśānta terminus, Para vak | Above crown, "forgetting zone" |
| Pashyanti level | Pre-verbal light (Ajna) | Jade Pillow → Baihui approach |
| Madhyama level | Internal voice (Vishuddha) | Middle Dan Tian |
| Vaikhara level | Manifest speech (Hṛdaya) | Lower Dan Tian, embodied |
| Earth bias | Mercurial core | Center (no direction, always present) |
| Spanda count | VBT Yukti 1 oscillation cycles | Microcosmic Orbit completions |

---

## Tests (all passing)

- `wiring_dimensions` — 2×4 shape verified
- `forward_pass_identity` — equal weight = 2.0 per level
- `ascending_favors_para` — Para ≥ Pashyanti ≥ Madhyama ≥ Vaikhara
- `descending_favors_vaikhara` — Vaikhara ≥ Madhyama ≥ Pashyanti ≥ Para
- `neuron_spanda_count` — increments per activation
- `neuron_oscillation_detection` — both channels required
- `layer_default_has_8_neurons` — 2×4 = 8
- `layer_spanda_cycles_accumulate` — 8 neurons × 2 activations = 16

---

## Integration with Sentron

`Sentron` now carries `pub neurons: NeuronLayer`. Each sentron has a default 8-neuron layer
available for conscious processing. The layer feeds from D-pipe (Story) and S-pipe (Light)
activations and outputs the 4-level vak activation vector.

Next: wire D-pipe dense outputs and S-pipe sparse outputs into neuron activations during
SIW execution — closing the cognitive loop between SIW retirement and neural activation.

---

## Total test count: 279 (library) + 14 (neuron unit) — all passing
