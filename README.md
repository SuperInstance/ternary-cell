# ternary-cell

Cellular computing with ternary state machines — cells with predict-perceive-surprise-vibe lifecycles, grid-based signal propagation, apoptosis, division, and tissue-level coordination.

## Why This Exists

Biological cells process information through cycles of sensing, predicting, reacting, and maintaining homeostasis. This crate models that cycle in a ternary framework: each cell holds a ternary state {-1, 0, +1}, receives ternary messenger signals from neighbors, predicts what it expects, measures surprise when reality diverges, and adjusts its energy accordingly. Cells that predict well thrive; cells that can't keep up undergo apoptosis.

This predict-perceive-surprise loop mirrors the **free energy principle** from neuroscience — organisms minimize surprise to stay alive — while the ternary state space {-1, 0, +1} maps naturally to inhibition/neutral/excitation in biological signaling. The result is a cellular automaton where behavior emerges from local ternary interactions rather than global rules.

This crate is part of the **Negative Space Intelligence** ecosystem.

## Core Concepts

- **TernaryMessenger** — A ternary signal: `Signal` (+1, promote), `Silence` (0, maintain), or `Suppress` (-1, inhibit).
- **TernaryCell** — A cell with internal ternary state, energy, inbox, and a 6-phase tick lifecycle:
  1. **Predict** — Forecast next value based on inbox signals.
  2. **Perceive** — Update ternary value from combined signal strength.
  3. **Surprise** — Compute prediction error (|actual − predicted|).
  4. **Vibe** — Adjust energy: lose energy from surprise, gain from accurate predictions.
  5. **GC** — Clear inbox, reclaim resources.
  6. **Conservation** — Clamp energy to [0, 20], trigger apoptosis at 0.
- **CellGrid** — A 2D grid of cells with neighbor-based signal propagation and parallel tick execution.
- **Tissue** — High-level coordinator for grid operations: pattern filling, convergence detection, and consensus computation.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-cell = "0.1"
```

```rust
use ternary_cell::*;

// Create a single cell
let mut cell = TernaryCell::with_value(0, 1);
assert!(cell.is_alive());
assert_eq!(cell.ternary_value, 1);

// Send signals and tick
cell.receive(TernaryMessenger::Signal);
cell.receive(TernaryMessenger::Signal);
let surprise = cell.tick();
assert_eq!(cell.ternary_value, 1); // confirmed by signals

// Cell division
if cell.can_divide() {
    let daughter = cell.divide(1).unwrap();
    assert_eq!(daughter.generation, 1);
}

// Grid simulation
let mut tissue = Tissue::new(3, 3);
tissue.fill_pattern(&[1, 0, -1, 0, 1, 0, -1, 0, 1]);
let alive = tissue.run(10);
println!("Alive after 10 ticks: {}", alive);
println!("Converged: {}", tissue.is_converged());
println!("Consensus: {}", tissue.consensus());

// Check tissue balance
let (pos, zero, neg) = tissue.grid.tissue_balance();
println!("+1: {}, 0: {}, -1: {}", pos, zero, neg);
```

## API Overview

### TernaryMessenger
| Method | Description |
|---|---|
| `to_ternary()` | Convert to i8 (-1, 0, +1) |
| `from_ternary(v)` | Convert from i8 |
| `combine(a, b)` | Max of two signals |

### TernaryCell
| Method | Description |
|---|---|
| `new(id)` / `with_value(id, v)` | Create cell |
| `receive(msg)` | Queue incoming signal |
| `tick()` | Run full lifecycle, returns surprise |
| `divide(daughter_id)` | Split into mother + daughter |
| `emit()` | Broadcast current value as messenger |
| `is_alive()` / `can_divide()` | State queries |

### CellGrid
| Method | Description |
|---|---|
| `new(w, h)` | Create empty grid |
| `place(x, y, value)` | Add cell at position |
| `get(x, y)` / `get_mut(x, y)` | Access cells |
| `neighbors(x, y)` | 4-connected neighbor positions |
| `propagate_signals()` | Deliver emissions to neighbors |
| `tick_all()` | Signal + tick all cells, remove dead |
| `tissue_balance()` | Count (pos, zero, neg) cells |

### Tissue
| Method | Description |
|---|---|
| `fill_pattern(values)` | Populate grid from flat array |
| `run(ticks)` | Simulate N ticks, return alive count |
| `is_converged()` | True if all cells agree |
| `consensus()` | Majority ternary value |

## How It Works

Each tick follows a six-phase lifecycle inspired by biological cell behavior and the free energy principle. First, the cell **predicts** its next state by summing incoming messenger signals — if the net signal is positive, it predicts +1; if negative, -1; if zero, it expects to maintain its current value. Then it **perceives** the actual combined signal and updates its ternary state accordingly.

**Surprise** is the absolute difference between prediction and reality. This drives energy dynamics: high surprise drains energy, while zero surprise (accurate prediction) provides a small energy bonus. This creates a natural selection pressure — cells that are well-calibrated to their neighbors survive, while poorly-predicting cells undergo apoptosis.

Signal propagation is synchronous: all cells emit their current value, then all emissions are delivered to neighbors before the next tick. This avoids race conditions and produces deterministic behavior. Cell division halves the parent's energy and creates a daughter with the parent's state, incrementing the generation counter.

## Use Cases

1. **Artificial life simulation** — Create self-organizing cellular systems where cells adapt to neighbors, form stable patterns, or undergo programmed death. The energy-surprise mechanism naturally produces homeostasis.

2. **Distributed ternary computing** — Model computation as emergent behavior of ternary cells on a grid, where global consensus arises from local signaling without centralized control.

3. **Pattern formation research** — Study how ternary signaling produces spatial patterns. The three-valued state space enables richer dynamics than binary cellular automata (compare to elementary CA rule space: 3^27 vs 2^8).

4. **Biological modeling** — Simulate tissue-level phenomena like wound healing, morphogen gradients, or cell competition, where the ternary states map to activation/inhibition/neutral signaling pathways.

## Ecosystem

| Crate | Relationship |
|---|---|
| `ternary-hardware` | Low-level trit/tryte primitives used by cells |
| `ternary-network` | Graph-based analysis of cell signaling patterns |
| `ternary-energy` | Thermodynamic models for cell energy systems |
| `ternary-locks` | Pattern abstraction for analyzing cell state patterns |
| `ternary-logic` | Formal logic for reasoning about cell states |

## License

MIT
