# ternary-cell

**Ternary cell — the atomic unit**

[![ternary](https://img.shields.io/badge/ecosystem-ternary-blue)](https://github.com/orgs/SuperInstance/repositories?q=ternary)
[![tests](https://img.shields.io/badge/tests-0-green)]()

## Overview

Ternary cell — the atomic unit. 3 bytes. Fits in a cache line.
Instantiate a million of these without thinking.

The cell knows three things:
- What it is (state: -1, 0, +1)
- How long it's been that way (dwell: how many ticks in current state)
- How many times it's changed (flips: lifetime transition count)

That's it. Everything else is computed from populations of these.

## Architecture

- **`Cell`** — core data structure
- **`Rng`** — core data structure
- **`Health`** — state enumeration

### Key Functions

- `new()`
- `pos()`
- `zero()`
- `neg()`
- `from_byte()`
- `set()`
- `tick()`
- `flip()`
- `tunnel()`
- `trap()`
- ... and 22 more

## Why Ternary?

The balanced ternary system {-1, 0, +1} (also known as Z₃) is the mathematically optimal discrete encoding:
- **More expressive than binary**: three states capture positive, neutral, and negative
- **Natural for decisions**: accept/reject/abstain, buy/hold/sell, agree/disagree/neutral
- **Self-balancing**: the 0 state acts as a universal screen, preventing pathological lock-in
- **Z₃ cyclic dynamics**: rock-paper-scissors is the only natural coordination mechanism

## Stats

| Metric | Value |
|--------|-------|
| Lines of Rust | 312 |
| Test count | 0 |
| Public types | 3 |
| Public functions | 32 |

## Ecosystem

This crate is part of the **[SuperInstance Ternary Fleet](https://github.com/orgs/SuperInstance/repositories?q=ternary)**:

- **[ternary-core](https://github.com/SuperInstance/ternary-core)** — shared traits and Z₃ arithmetic
- **[ternary-grid](https://github.com/SuperInstance/ternary-grid)** — spatial grid with {-1, 0, +1} cells
- **[ternary-graph](https://github.com/SuperInstance/ternary-graph)** — ternary-weighted graph algorithms
- **[ternary-automata](https://github.com/SuperInstance/ternary-automata)** — three-state cellular automata
- **[ternary-compiler](https://github.com/SuperInstance/ternary-compiler)** — expression compiler and optimizer

200+ crates. 4,300+ tests. One pattern.

## Research Context

The ternary approach connects to several active research areas:
- **Ternary Neural Networks** (TNNs): weights constrained to {-1, 0, +1} for efficient inference
- **Huawei's ternary chip**: 7nm ternary silicon with 60% less power consumption
- **Active inference**: free energy minimization naturally maps to ternary action selection
- **Cyclic dominance**: RPS dynamics maintain biodiversity in spatial ecology
- **Z₃ group theory**: the only algebraic group on three elements is cyclic addition mod 3

## Usage

```toml
[dependencies]
ternary-cell = "0.1.0"
```

```rust
use ternary_cell;
```

## License

MIT
