# ternary-cell

Cellular computing with a six-phase tick cycle (predict → perceive → surprise → vibe → gc → conservation), cell signaling, division, apoptosis, and grid-level tissue coordination.

## Why This Exists

Biological cells don't just process — they predict, perceive mismatches, and adapt. This crate models that lifecycle as a ternary compute primitive. Each cell holds a ternary value {-1, 0, +1}, receives signals from neighbors, predicts what it should become, measures surprise (prediction error), and adjusts its energy accordingly. Cells with no energy undergo apoptosis; cells with surplus energy divide. The result is a self-organizing grid where local signaling produces emergent global behavior. It maps directly to the construct-core compute model.

## Core Concepts

- **TernaryMessenger** — The signal type passed between cells: Signal (+1, promotes growth/activation), Silence (0, maintain), Suppress (−1, inhibit/retract).
- **Six-phase tick cycle** — Every cell step runs predict → perceive → surprise → vibe → gc → conservation in order. This mirrors the biological cell cycle adapted for ternary computation.
- **Surprise** — Prediction error: `|ternary_value − prediction|`. Zero surprise means the cell correctly predicted its next state; nonzero surprise drains energy.
- **Energy and apoptosis** — Cells start with 10 energy (max 20). Surprise costs energy; correct predictions gain energy. At zero energy, a cell enters the Apoptotic state and is removed from the grid.
- **Cell division** — When energy ≥ 10, a cell can divide: energy is halved between parent and daughter, the daughter inherits the parent's ternary value, and the daughter's generation increments.
- **Tissue** — A grid-level coordinator. Tracks convergence (all alive cells sharing the same ternary value) and consensus (majority ternary value across the grid).
- **CellGrid** — A 2D grid with 4-connected neighborhoods, signal propagation, and bulk tick execution.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-cell = "0.1"
```

```rust
use ternary_cell::*;

fn main() {
    // Create a 3x3 tissue
    let mut tissue = Tissue::new(3, 3);
    tissue.fill_pattern(&[1, 1, -1, 0, 1, -1, -1, 0, 1]);

    // Run 5 ticks of the lifecycle
    let alive = tissue.run(5);
    println!("Alive cells after 5 ticks: {}", alive);
    println!("Converged: {}", tissue.is_converged());
    println!("Consensus value: {}", tissue.consensus());

    // Single cell lifecycle
    let mut cell = TernaryCell::with_value(0, 1);
    cell.receive(TernaryMessenger::Signal);
    cell.receive(TernaryMessenger::Signal);
    let surprise = cell.tick();
    println!("Surprise: {}, Energy: {}", surprise, cell.energy);
}
```

## API Overview

| Type | Description |
|------|-------------|
| `TernaryMessenger` | Signal/Silence/Suppress — inter-cell messaging with ternary encoding |
| `TernaryCell` | A single cell with energy, ternary value, inbox, and six-phase tick |
| `CellState` | Active, Apoptotic, or Dividing lifecycle state |
| `CellGrid` | 2D grid of cells with signal propagation and bulk tick |
| `Tissue` | Grid-level coordinator with convergence and consensus analysis |

## How It Works

**Signal propagation** runs before tick. Each alive cell emits its current ternary value as a `TernaryMessenger` to its 4-connected neighbors (up/down/left/right). Emissions are collected first, then delivered, to avoid order-dependent artifacts.

**Predict phase:** Sum incoming signals. If the sum is positive, predict +1; if negative, predict −1; if zero, keep the current value as the prediction.

**Perceive phase:** Sum incoming signals again. If nonzero, set the cell's ternary value to the clamped sum. This is where the cell actually changes state based on neighbor signals.

**Surprise phase:** Compute `|current_value − prediction|`. This is the prediction error — how surprised the cell is by its own state change.

**Vibe phase:** Subtract surprise from energy. If surprise was zero (perfect prediction), add 1 energy as a reward for being in sync with neighbors.

**GC (garbage collect) phase:** Clear the inbox.

**Conservation phase:** Clamp energy to [0, 20]. If energy hits 0, mark the cell Apoptotic. Increment the tick counter.

**Division:** A cell with energy ≥ 10 can produce a daughter cell. The parent's energy is halved, the parent enters Dividing state, and the daughter starts Active with the same ternary value and an incremented generation counter.

**Tissue convergence** checks whether all alive cells share the same ternary value. **Tissue consensus** returns the majority value (or 0 on ties).

## Known Limitations

- **4-connected neighborhoods only.** No diagonal (8-connected) or hexagonal grid support. Adding these would require changes to `CellGrid::neighbors`.
- **No persistent state between ticks beyond energy and ternary value.** Cells can't learn or adapt their prediction strategy — prediction is always a simple sum of incoming signals.
- **Signal propagation is synchronous.** All cells emit, then all cells receive, then all cells tick. There's no asynchronous or stochastic signaling mode.
- **`TernaryMessenger::combine` uses max-wins, not sum.** This means two Suppressions (−1) can't overcome a single Signal (+1). This is a design choice for signal dominance, but may not suit all models.

## Use Cases

- **Cellular automata experiments** — Build ternary Game-of-Life variants where cells predict, compete, and die based on prediction accuracy.
- **Distributed consensus simulation** — Model how local majority-vote signaling leads to global convergence or fragmentation.
- **Edge computing** — Each cell represents a sensor node that classifies its reading as {-1, 0, +1}, signals neighbors, and adapts based on agreement or disagreement.

## Ecosystem Context

Part of the SuperInstance ternary crate family. `ternary-cell` is the compute primitive layer. It can be driven by `ternary-compiler-v2` for programmatic cell behavior, visualized with `ternary-visualization`, and diffed over time with `ternary-diff`. The `TernaryMessenger` type maps to the standard {-1, 0, +1} encoding used throughout the ecosystem.

## Known Limitations

- **Signal count doesn't matter, only sign.** `TernaryCell::perceive()` clamps the combined signal sum to [-1, 1]. A cell receiving 100 Signal messages and a cell receiving 1 Signal message both end up with `ternary_value = 1`.
- **Synchronous tick is O(n²).** Signal propagation collects all emissions then delivers them, making each tick quadratic in grid population.
- **No asynchronous or stochastic mode.** All cells emit, receive, and tick in lockstep.
- **Consensus uses plurality, which can return Zero on ties.** When Pos and Neg counts are equal (even if Zero count is low), `Tissue::consensus()` returns 0 (neutral).
- **Fixed-size grid with no sparse optimization.** `CellGrid` uses `Vec<Option<TernaryCell>>`, so memory is proportional to width × height regardless of how many cells are alive.

## License

MIT
