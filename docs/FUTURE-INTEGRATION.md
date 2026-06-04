# Future Integration: ternary-cell

## Current State

ternary-cell provides the fundamental tick cycle engine for the ternary ecosystem. `TernaryCell` implements a six-phase lifecycle: `predict()` → `perceive()` → `surprise()` → `vibe()` → `gc()` → `conservation()`. Each cell carries energy, a ternary value (-1/0/+1), a prediction, surprise accumulation, and an inbox of `TernaryMessenger` signals. `CellGrid` arranges cells in a 2D grid with 4-connected neighbor signaling. `Tissue` coordinates grid-level operations including `run(ticks)`, `is_converged()`, and `consensus()`. Cells can divide (`divide()` splitting energy to a daughter) or undergo apoptosis (energy → 0).

## Integration Opportunities

### PLATO Room Heartbeat (room-as-codespace)

Every PLATO room ticks at its own rate. The `TernaryCell::tick()` method IS the room heartbeat. A `CodespaceRoom.tick()` calls `CellGrid::tick_all()`, which runs `propagate_signals()` then ticks every cell. The `TickReport` (from ROOM-AS-CODESPACE-ARCHITECTURE.md) maps directly to `Tissue` data: `cells_ticked` = `alive_count()`, `apoptosis_count` = cells that died, `conservation` from `tissue_balance()`, and `avg_surprise` from accumulated surprise values. The tick rate differs by hardware: microseconds on ESP32, seconds in a Codespace, but the interface is identical.

### avoidance-cascade → CellGcStrategy

ternary-cell's `gc()` phase simply clears the inbox. This risks avoidance cascades where cells converge to monoculture. The `avoidance-cascade` crate's balanced learning algorithm (average reward, forced exploration, memory decay) maps to a `CellGcStrategy` trait: `GreedyGc` (current), `BalancedGc` (inject random `TernaryMessenger` signals during gc), `EcologicalGc` (Lotka-Volterra population dynamics). This prevents the death spiral where all cells converge to the same ternary value.

### Polln's Plinko Layer → Stochastic GC

The Polln colony intelligence uses Gumbel-Softmax for stochastic selection maintaining diversity. This maps to replacing `CellGrid::tick_all()`'s apoptosis with stochastic sampling: instead of deterministically killing `CellState::Apoptotic` cells, sample which cells to keep using surprise-weighted probabilities. This maintains grid diversity.

### ternary-compiler → Grid→Firmware Pipeline

A trained `CellGrid` (converged tissue) can be compiled to a lookup table via `ternary-compiler`. The tissue's `consensus()` value at each grid position becomes the compiled policy. Pipeline: train grid on Pi → compile with `ternary-compiler` → flash to ESP32 as `ternary-esp32-firmware` (279 bytes, 8ns lookup).

## Potential in Mature Systems

ternary-cell is the compute kernel of the entire ecosystem. In a mature room-as-codespace deployment, every room contains a `CellGrid`. The tick cycle is the universal heartbeat — from a single ESP32 cell (Layer 0, `query_lookup`) to a DGX running a million-cell grid (Layer 2, GPU-accelerated via CUDA from `ptx-bench` benchmarks). The conservation law tracked in `conservation()` becomes the fleet-wide health metric: γ + H ≈ 1.283 - 0.159·log(V).

## Cross-Pollination Ideas

- **ternary-music × cell tick**: Voice leading smoothness (from `ternary-music`) could replace the `vibe()` energy adjustment — cells "harmonize" with neighbors instead of simply gaining energy for zero surprise.
- **strategy-ecology's 5 species as cell populations**: Each cell carries a species tag (Explorer, Diplomat, Marksman, Climber, Prospector). Grid diversity = species diversity. Lotka-Volterra interaction matrix = neighbor signaling weights.

## Dependencies for Next Steps

1. `CellGcStrategy` trait extracted from current hardcoded `gc()` — no new dependencies
2. Integration with `ternary-compiler` requires `CompiledPolicy` format alignment
3. GPU acceleration path needs `ptx-bench` kernel benchmarks for cell tick on CUDA
4. Room trait integration requires `construct-core` Layer 0/1/2 trait implementations
