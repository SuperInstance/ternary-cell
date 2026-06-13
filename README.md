# Ternary Cell

The **atomic unit of scale** — a 3-byte ternary cell that is stack-allocated, zero-allocation, and instantiable by the million. Every cell tracks three things: its state {-1, 0, +1}, how long it has been in that state (dwell), and how many times it has changed (flips). Everything else is computed from populations.

## Why It Matters

The `Cell` struct is deliberately minimal: 3 bytes, no heap allocation, no generics, no traits beyond `Debug + Clone + Copy + PartialEq + Eq`. This means:

- **1 million cells = 3 MB** — fits in L3 cache on any modern CPU
- **100 million cells = 300 MB** — fits in RAM, still cache-friendly
- Population operations (census, entropy, gamma) operate on `&[Cell]` slices with zero allocation

This design follows the **data-oriented design** (DOD) philosophy: separate data from behavior, lay out data for cache efficiency, and process populations in bulk. The cell is the struct-of-arrays (SoA) friendly atom that makes ternary simulation practical at scale.

## How It Works

### Cell Layout

```rust
pub struct Cell {
    pub state: i8,   // -1, 0, or +1   (1 byte)
    pub dwell: u8,   // ticks in current state, wraps at 255  (1 byte)
    pub flips: u8,   // lifetime transition count, wraps at 255  (1 byte)
}
// Total: 3 bytes. No padding (all fields are u8-compatible).
```

The `assert!(std::mem::size_of::<Cell>() <= 3)` test enforces this at compile time.

### State Transitions

All transitions are O(1):

| Operation | Effect |
|-----------|--------|
| `set(new_state)` | If different: update state, reset dwell, increment flips |
| `tick()` | Increment dwell (saturating) |
| `flip()` | Negate state (+1 ↔ -1, 0 stays 0), increment flips |
| `tunnel(target)` | If state == 0: transition to target (forgiveness) |
| `trap()` | If state ≠ 0: force to 0 (inhibition) |

### Population Statistics

For a population of N cells, the key statistics are:

**Census**: $(n_{-1}, n_0, n_{+1})$ — counts in O(N).

**Gamma** (signed charge): $\gamma = \sum_i s_i$ — the net ternary value. O(N).

**Shannon entropy**: $H = -\sum_{v \in \{-1,0,1\}} p_v \log_2 p_v$ where $p_v = n_v / N$. Maximum $H = \log_2 3 \approx 1.585$. O(N).

**Flip rate**: $\bar{f} = \frac{1}{N}\sum_i \text{flips}_i$ — measures population volatility. O(N).

**Health classification**:
| State | Condition |
|-------|-----------|
| Dead | >95% at zero |
| Critical | >80% at zero |
| Settled | Flip rate < 0.5 |
| Alive | Normal activity |
| Chaotic | Flip rate > 5.0 |

### Complexity

| Operation | Time | Space |
|-----------|------|-------|
| Single cell operation | O(1) | O(1) |
| `census(&[Cell])` | O(N) | O(1) |
| `gamma(&[Cell])` | O(N) | O(1) |
| `entropy(&[Cell])` | O(N) | O(1) |
| `tick_all(&mut [Cell])` | O(N) | O(1) |
| `apply_rule(&mut [Cell], fn)` | O(N) | O(1) |
| `apply_neighbor_rule(&mut [Cell], fn)` | O(N) | O(N) temp |
| `from_bytes(n, seed)` | O(N) | O(N) |

## Quick Start

```rust
use ternary_cell::{Cell, census, gamma, entropy, health, Health, tick_all};

// Create cells
let mut cells = vec![Cell::pos(), Cell::neg(), Cell::zero(), Cell::pos()];

// Population statistics
let (neg, zero, pos) = census(&cells);
assert_eq!((neg, zero, pos), (1, 1, 2));
assert_eq!(gamma(&cells), 1); // 2 positive - 1 negative
let h = entropy(&cells); // ~1.5 bits (near maximum)

// Tick and evolve
tick_all(&mut cells);
cells[0].set(-1); // flip first cell to negative

// Health diagnostic
match health(&cells) {
    Health::Alive => println!("Population is healthy"),
    Health::Dead => println!("Population has collapsed"),
    _ => {}
}

// Random population from entropy
use ternary_cell::{Rng, random_pop};
let mut rng = Rng::new(42);
let pop = random_pop(1000, 0.33, 0.34, &mut rng);
```

## API

### Cell

| Method | Description |
|--------|-------------|
| `new(state) / pos() / zero() / neg()` | Constructors |
| `from_byte(u8)` | Deterministic random from any byte |
| `set(i8) → bool` | Update state, returns whether it changed |
| `tick()` | Increment dwell time |
| `flip()` | Negate signed state |
| `tunnel(target) → bool` | 0 → target (forgiveness) |
| `trap() → bool` | Any → 0 (inhibition) |
| `is_settled(threshold) / is_active() / is_spindle() / is_oscillating(threshold)` | Predicates |
| `charge() / abs_charge()` | Signed / absolute ternary charge |

### Population Functions

| Function | Description |
|--------|-------------|
| `census(&[Cell]) → (neg, zero, pos)` | State counts |
| `gamma(&[Cell]) → i64` | Net signed charge |
| `abs_gamma(&[Cell]) → i64` | Total absolute charge |
| `fractions(&[Cell]) → (f64, f64, f64)` | State fractions |
| `entropy(&[Cell]) → f64` | Shannon entropy (base 2) |
| `flip_rate(&[Cell]) → f64` | Mean flips per cell |
| `mean_dwell(&[Cell]) → f64` | Mean dwell time |
| `health(&[Cell]) → Health` | Population diagnostic |
| `tick_all(&mut [Cell])` | Tick entire population |
| `apply_rule(&mut [Cell], fn)` | Apply global rule |
| `apply_neighbor_rule(&mut [Cell], fn)` | Apply cellular automaton rule |

## Architecture Notes

The cell is the irreducible unit of the **γ + η = C** conservation principle:

- **γ (structure)**: the state assignment — which cells are -1, 0, +1
- **η (dynamics)**: the stream of ticks, flips, tunnels, and traps that perturb the population
- **C (conservation)**: the total charge invariant — $\gamma = \sum s_i$ is conserved under symmetric perturbations (equal +1→0 and -1→0 transitions)

The `tunnel` operation represents **forgiveness** — allowing a neutral cell to commit to a direction. The `trap` operation represents **inhibition** — forcing an active cell back to neutral. Together, they implement a ternary relaxation mechanism that prevents pathological lock-in: the 0 state acts as a universal screen, ensuring Z₃ cyclic dominance rather than binary polarization.

## References

- Nystrom, J. (2022). *Data-Oriented Design*. — Cache-friendly data layout principles.
| Wolfram, S. (2002). *A New Kind of Science*. — Cellular automata and computational universality.
| Toffoli, T. & Margolus, N. (1987). *Cellular Automata Machines*. MIT Press.
| Gardner, M. (1970). *Mathematical Games: The Fantastic Combinations of John Conway's New Solitaire Game "Life"*. Scientific American.

## License: MIT
