# Room-Cell Bridge — Connecting ternary-cell to room-cell

> How ternary-cell's production Rust tick engine maps back to room-cell's conceptual architecture,
> and the bidirectional integration path.

## 1. Architecture Comparison

| Aspect | ternary-cell | room-cell |
|--------|-------------|-----------|
| Value type | Discrete {-1, 0, +1} | Continuous f64 embeddings |
| State | Compact (energy + ternary) | Rich (vibe[16] + perception_db + prediction_db) |
| Neighbors | CellGrid 2D, 4-connected | None (singleton) |
| Signaling | TernaryMessenger push | None |
| Lifecycle | Active/Dividing/Apototic | Immortal |
| Coordination | Tissue (grid-level ops) | None |
| Prediction | Inbox aggregation | JEPA moving average |
| Surprise | Absolute difference | Cosine distance |
| GC | Clear inbox | Surprise-ranked pruning |
| Conservation | Energy bounds + apoptosis | Perception/prediction count balance |
| Memory | Stateless between ticks | Full history with GC |

## 2. Tick Phase Mapping (ternary-cell → room-cell)

### Phase 1: Predict

```rust
// ternary-cell
fn predict(&mut self) {
    let combined: i32 = self.inbox.iter().map(|m| m.to_ternary() as i32).sum();
    self.prediction = if combined > 0 { 1 } else if combined < 0 { -1 } else { self.ternary_value };
}

// room-cell equivalent
fn predict(&self) -> [f64; D] {
    // JEPA: average last 5 embeddings
}
```

**Mapping:** ternary-cell's inbox-driven prediction is a discrete version of room-cell's JEPA prediction. Instead of averaging continuous embeddings, ternary-cell sums discrete signals. The ternary prediction is essentially a "majority vote" from neighbors, while JEPA is a "temporal average" of past data.

**Integration path:** ternary-cell should add JEPA-style temporal smoothing. Currently, if inbox is empty, prediction defaults to current value. With JEPA, it would default to the moving average of past values.

### Phase 2: Perceive

```rust
// ternary-cell
fn perceive(&mut self) {
    let combined: i32 = self.inbox.iter().map(|m| m.to_ternary() as i32).sum();
    if combined != 0 { self.ternary_value = combined.clamp(-1, 1) as i8; }
}

// room-cell equivalent
fn perceive(&mut self, embedding: Embedding<D>) -> f64 { ... }
```

**Mapping:** ternary-cell's perceive is purely social (what neighbors say); room-cell's perceive is purely observational (what the world shows). A unified perceive takes both: external data + social signals.

### Phase 3: Surprise

```rust
// ternary-cell: integer absolute difference
fn compute_surprise(&mut self) -> i32 {
    self.surprise = (self.ternary_value as i32 - self.prediction as i32).abs();
    self.surprise
}

// room-cell: cosine distance (continuous)
fn surprise(actual: &[f64; D], predicted: &[f64; D]) -> f64 {
    1.0 - cosine_similarity(actual, predicted)
}
```

**Mapping:** Both measure prediction error. ternary-cell's is coarse-grained (0, 1, or 2). room-cell's is fine-grained (0.0 to 2.0 continuous). ternary-cell's surprise could be upgraded to a weighted surprise that accounts for neighbor agreement percentage.

### Phase 4: Vibe

```rust
// ternary-cell: energy adjustment
fn vibe(&mut self) {
    self.energy -= self.surprise;
    if self.surprise == 0 { self.energy += 1; }
}

// room-cell: 16-dim vector update
fn update_vibe(&mut self) {
    // Maps perception to vibe dims via strided projection with learning rate
}
```

**Mapping:** ternary-cell's vibe is scalar survival; room-cell's is vector expression. ternary-cell loses the *direction* of surprise (only magnitude). Adding a vibe vector would capture which *dimension* of state is most surprising.

### Phase 5: GC

```rust
// ternary-cell: dump everything
fn gc(&mut self) { self.inbox.clear(); }

// room-cell: keep the most informative
fn gc(&mut self) {
    self.perception_db.sort_by(|a, b| b.surprise.cmp(&a.surprise));
    self.perception_db.truncate(self.gc_threshold);
}
```

**Mapping:** This is the biggest gap. ternary-cell's GC is wasteful — it discards all signals every tick. room-cell's GC is intelligent — it preserves the most surprising memories. ternary-cell should adopt surprise-ranked inbox pruning.

### Phase 6: Conservation

```rust
// ternary-cell: energy clamp + death check
fn conservation(&mut self) {
    self.energy = self.energy.clamp(0, 20);
    if self.energy == 0 { self.state = CellState::Apoptotic; }
    self.tick_count += 1;
}

// room-cell: info balance check
fn check_conservation(&self) -> bool {
    (self.perception_db.len() as f64 - self.prediction_db.len() as f64).abs() <= 1.0
}
```

**Mapping:** Both are conservation laws. ternary-cell conserves energy; room-cell conserves information. A unified conservation would track both: energy budget *and* information budget.

## 3. What ternary-cell Has (room-cell Should Adopt)

### 3.1 TernaryMessenger — Discrete Signaling
The `Signal / Silence / Suppress` triad is elegant. room-cell has no signaling mechanism at all. Adding `TernaryMessenger` to room-cell would enable cells to coordinate.

### 3.2 CellGrid — Spatial Organization
`CellGrid` provides 2D topology with `neighbors()`, `place()`, and `get()`. room-cell is a lone wolf. Grids enable emergent behavior.

### 3.3 Cell Lifecycle
`divide()` and apoptosis create population dynamics. room-cell cells live forever regardless of fitness. Natural selection pressure would make room-cell populations evolve.

### 3.4 Tissue Coordinator
`Tissue::run(ticks)`, `Tissue::is_converged()`, `Tissue::consensus()` provide grid-level intelligence. room-cell has no equivalent.

### 3.5 Signal Propagation
`CellGrid::propagate_signals()` — collect emissions, deliver to neighbors — is the communication backbone room-cell lacks.

### 3.6 Energy Economy
ternary-cell's energy system (gain for sync, lose for surprise, die at 0, divide at 10+) creates fitness pressure. room-cell has no resource management.

### 3.7 Generation Tracking
`generation: u32` incremented on division tracks cell lineage. Useful for evolutionary analysis.

## 4. What room-cell Has (ternary-cell Should Backport)

### 4.1 Continuous Embedding Space
room-cell operates on D-dimensional f64 vectors. ternary-cell is limited to 3 values. Continuous embeddings enable nuanced state representation.

### 4.2 JEPA Prediction (Temporal Memory)
room-cell's moving-average predictor uses the last 5 perceptions. ternary-cell has no memory of past ticks beyond current state. JEPA provides temporal continuity.

### 4.3 16-Dimensional Vibe
The vibe vector captures a cell's "emotional state" across 16 dimensions. ternary-cell's scalar energy is a poor substitute for rich internal state.

### 4.4 Surprise-Ranked GC
room-cell keeps the most surprising memories. ternary-cell discards all signals every tick. Ranked GC preserves important information.

### 4.5 MurmurSummary (Gossip)
room-cell can generate compressed state summaries for gossip. ternary-cell has no equivalent for long-range information sharing.

### 4.6 Perception/Prediction Databases
room-cell maintains separate databases for what it saw and what it predicted. ternary-cell has no logging. This enables retrospective analysis.

### 4.7 Configurable GC Threshold
room-cell's `gc_threshold` is user-configurable. ternary-cell's GC is hardcoded (clear all).

## 5. Integration Plan

### Phase 1: Add Memory to ternary-cell (Priority: High)
```rust
pub struct TernaryCell {
    // ... existing ...
    perception_log: Vec<i8>,      // last N perceived ternary values
    prediction_log: Vec<i8>,      // last N predictions
    gc_threshold: usize,          // configurable retention
}
```

### Phase 2: Upgrade GC to Surprise-Ranked (Priority: High)
```rust
fn gc(&mut self) {
    if self.inbox.len() > self.gc_threshold {
        // Keep messages that caused most surprise
        self.inbox.sort_by(|a, b| b.to_ternary().cmp(&a.to_ternary()));
        self.inbox.truncate(self.gc_threshold);
    } else {
        self.inbox.clear();
    }
}
```

### Phase 3: Add Vibe Vector (Priority: Medium)
```rust
pub struct TernaryCell {
    // ... existing ...
    pub vibe: [f64; 16],
}

fn update_vibe(&mut self) {
    // Map ternary state to vibe dimensions
    // Use surprise history for directional update
}
```

### Phase 4: Add JEPA Prediction (Priority: Medium)
```rust
fn jepa_predict(&self) -> i8 {
    if self.perception_log.is_empty() { return self.ternary_value; }
    let window = &self.perception_log[self.perception_log.len().saturating_sub(5)..];
    let sum: i32 = window.iter().map(|&v| v as i32).sum();
    (sum as f64 / window.len() as f64).round() as i8
}
```

### Phase 5: Add Murmur/Gossip (Priority: Low)
```rust
pub struct MurmurSummary {
    pub vibe_snapshot: [f64; 16],
    pub avg_surprise: f64,
    pub tick: u64,
    pub alive_neighbors: usize,
}
```

## 6. Code Sketch: Upgraded TernaryCell

```rust
pub struct TernaryCellV2 {
    // Core (unchanged)
    pub id: u64,
    pub energy: i32,
    pub state: CellState,
    pub ternary_value: i8,
    pub generation: u32,

    // Prediction (upgraded)
    prediction: i8,
    inbox: Vec<TernaryMessenger>,

    // NEW: Memory
    perception_log: Vec<i8>,
    prediction_log: Vec<i8>,
    surprise_log: Vec<i32>,

    // NEW: Vibe
    pub vibe: [f64; 16],

    // NEW: Configurable
    gc_threshold: usize,
    tick_count: u64,
}

impl TernaryCellV2 {
    pub fn tick(&mut self) -> i32 {
        // 1. Predict: JEPA + inbox hybrid
        let jepa = self.jepa_predict();
        let inbox_pred = self.inbox_predict();
        self.prediction = if self.inbox.is_empty() { jepa } else { inbox_pred };
        self.prediction_log.push(self.prediction);

        // 2. Perceive: signals + temporal
        self.perceive();
        self.perception_log.push(self.ternary_value);

        // 3. Surprise
        let s = self.compute_surprise();
        self.surprise_log.push(s);

        // 4. Vibe: vector + energy
        self.update_vibe();
        self.energy -= s;
        if s == 0 { self.energy += 1; }

        // 5. GC: intelligent
        self.intelligent_gc();

        // 6. Conservation
        self.energy = self.energy.clamp(0, 20);
        if self.energy == 0 { self.state = CellState::Apoptotic; }
        self.tick_count += 1;

        s
    }
}
```

## 7. Cross-System Grid Vision

```rust
/// A grid that can host both room-cells and ternary-cells.
pub struct HybridGrid<const D: usize> {
    cells: Vec<Option<HybridCell<D>>>,
    width: usize,
    height: usize,
}

/// Cells can be either type, unified under a trait.
pub enum HybridCell<D> {
    Room(RoomCellAdapter<D>),
    Ternary(TernaryCellAdapter),
}

// Both implement Cellular trait
pub trait Cellular {
    fn tick(&mut self);
    fn emit(&self) -> TernaryMessenger;
    fn receive(&mut self, msg: TernaryMessenger);
    fn is_alive(&self) -> bool;
    fn summary(&self) -> CellSummary;
}
```

## 8. Migration Path

1. **Immediate**: Port ternary-cell to Python (see `ternary-cell-python/`) for side-by-side experimentation with room-cell
2. **Short-term**: Add `inbox`, `emit()`, `receive()` to room-cell; add `perception_log` to ternary-cell
3. **Medium-term**: Define `Cellular` trait; implement for both systems
4. **Long-term**: Merge into unified `hybrid-cell` crate with pluggable value types

## 9. Quick Reference: Reverse Mapping

| ternary-cell construct | room-cell equivalent | Gap |
|-----------------------|---------------------|-----|
| `TernaryMessenger` | — | room-cell needs signaling |
| `CellState` enum | — | room-cell needs lifecycle |
| `TernaryCell` struct | `Room<D>` struct | Different value spaces |
| `CellGrid` | — | room-cell needs topology |
| `Tissue` | — | room-cell needs coordination |
| `propagate_signals()` | — | room-cell needs communication |
| `divide()` | — | room-cell needs reproduction |
| `emit()` / `receive()` | — | room-cell needs messaging |
| `tissue_balance()` | `check_conservation()` | Different resources |
| `consensus()` | — | room-cell needs agreement |
| `is_converged()` | — | room-cell needs convergence |
| — | `Embedding<D>` | ternary-cell needs continuous state |
| — | `predict()` JEPA | ternary-cell needs temporal memory |
| — | `update_vibe()` 16-dim | ternary-cell needs rich internal state |
| — | `gc()` ranked | ternary-cell needs intelligent GC |
| — | `MurmurSummary` | ternary-cell needs gossip |
