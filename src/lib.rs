#![forbid(unsafe_code)]
//! Ternary cell — the atomic unit. 3 bytes. Fits in a cache line.
//! Instantiate a million of these without thinking.
//!
//! The cell knows three things:
//! - What it is (state: -1, 0, +1)
//! - How long it's been that way (dwell: how many ticks in current state)
//! - How many times it's changed (flips: lifetime transition count)
//!
//! That's it. Everything else is computed from populations of these.

/// A single ternary cell. 3 bytes. No heap. No strings. No generics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub state: i8,    // -1, 0, or +1
    pub dwell: u8,    // ticks in current state (wraps at 255)
    pub flips: u8,    // lifetime transition count (wraps at 255)
}

impl Cell {
    /// Create a cell in any state.
    #[inline]
    pub fn new(state: i8) -> Self {
        Self { state: state.clamp(-1, 1), dwell: 0, flips: 0 }
    }

    /// Create at +1.
    #[inline]
    pub fn pos() -> Self { Self::new(1) }

    /// Create at 0.
    #[inline]
    pub fn zero() -> Self { Self::new(0) }

    /// Create at -1.
    #[inline]
    pub fn neg() -> Self { Self::new(-1) }

    /// Create a random cell from a u8 seed (use any bit of entropy).
    #[inline]
    pub fn from_byte(b: u8) -> Self {
        match b % 3 {
            0 => Self::neg(),
            1 => Self::zero(),
            _ => Self::pos(),
        }
    }

    /// Set state. Returns true if state actually changed.
    #[inline]
    pub fn set(&mut self, new_state: i8) -> bool {
        let s = new_state.clamp(-1, 1);
        if s != self.state {
            self.state = s;
            self.dwell = 0;
            self.flips = self.flips.saturating_add(1);
            true
        } else {
            false
        }
    }

    /// Tick — increment dwell time.
    #[inline]
    pub fn tick(&mut self) {
        self.dwell = self.dwell.saturating_add(1);
    }

    /// Flip to opposite signed state (+1 ↔ -1). 0 stays 0.
    #[inline]
    pub fn flip(&mut self) {
        self.state = -self.state;
        self.flips = self.flips.saturating_add(1);
        self.dwell = 0;
    }

    /// Tunnel from 0 to target state (forgiveness).
    #[inline]
    pub fn tunnel(&mut self, target: i8) -> bool {
        if self.state == 0 {
            self.set(target);
            true
        } else {
            false
        }
    }

    /// Trap into 0 state.
    #[inline]
    pub fn trap(&mut self) -> bool {
        if self.state != 0 {
            self.set(0);
            true
        } else {
            false
        }
    }

    /// Is settled (dwelling for a long time)?
    #[inline]
    pub fn is_settled(&self, threshold: u8) -> bool {
        self.dwell >= threshold
    }

    /// Is active (not 0)?
    #[inline]
    pub fn is_active(&self) -> bool { self.state != 0 }

    /// Is at spindle (0)?
    #[inline]
    pub fn is_spindle(&self) -> bool { self.state == 0 }

    /// Is oscillating (high flip count)?
    #[inline]
    pub fn is_oscillating(&self, threshold: u8) -> bool {
        self.flips >= threshold
    }

    /// Signed charge.
    #[inline]
    pub fn charge(&self) -> i8 { self.state }

    /// Absolute charge.
    #[inline]
    pub fn abs_charge(&self) -> i8 { self.state.abs() }
}

// ============================================================
// Population functions — operate on slices, no allocations
// ============================================================

/// Count each state in a population. Returns (neg, zero, pos).
pub fn census(pop: &[Cell]) -> (usize, usize, usize) {
    let mut n = 0usize;
    let mut z = 0usize;
    let mut p = 0usize;
    for c in pop {
        match c.state {
            -1 => n += 1,
            0 => z += 1,
            _ => p += 1,
        }
    }
    (n, z, p)
}

/// Sum of charges (gamma). Fast, no allocation.
pub fn gamma(pop: &[Cell]) -> i64 {
    pop.iter().map(|c| c.state as i64).sum()
}

/// Absolute gamma (|gamma|). Measures total charge ignoring sign.
pub fn abs_gamma(pop: &[Cell]) -> i64 {
    pop.iter().map(|c| c.state.abs() as i64).sum()
}

/// Fraction in each state. Returns (frac_neg, frac_zero, frac_pos).
pub fn fractions(pop: &[Cell]) -> (f64, f64, f64) {
    let (n, z, p) = census(pop);
    let total = pop.len() as f64;
    (n as f64 / total, z as f64 / total, p as f64 / total)
}

/// Mean flip rate (how often cells are changing).
pub fn flip_rate(pop: &[Cell]) -> f64 {
    if pop.is_empty() { return 0.0; }
    pop.iter().map(|c| c.flips as f64).sum::<f64>() / pop.len() as f64
}

/// Mean dwell time (how settled the population is).
pub fn mean_dwell(pop: &[Cell]) -> f64 {
    if pop.is_empty() { return 0.0; }
    pop.iter().map(|c| c.dwell as f64).sum::<f64>() / pop.len() as f64
}

/// Entropy of the state distribution (Shannon, base 2).
pub fn entropy(pop: &[Cell]) -> f64 {
    let (n, z, p) = fractions(pop);
    let mut h = 0.0;
    if n > 0.0 { h -= n * n.log2(); }
    if z > 0.0 { h -= z * z.log2(); }
    if p > 0.0 { h -= p * p.log2(); }
    h
}

/// Health diagnostic from population state.
pub fn health(pop: &[Cell]) -> Health {
    if pop.is_empty() { return Health::Empty; }
    let (_, z, _) = census(pop);
    let frac_zero = z as f64 / pop.len() as f64;
    let rate = flip_rate(pop);

    if frac_zero > 0.95 { Health::Dead }
    else if frac_zero > 0.8 { Health::Critical }
    else if rate < 0.5 { Health::Settled }
    else if rate > 5.0 { Health::Chaotic }
    else { Health::Alive }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Empty,    // No cells
    Dead,     // 95%+ at 0
    Critical, // 80%+ at 0
    Settled,  // Low flip rate
    Alive,    // Normal activity
    Chaotic,  // Very high flip rate
}

/// Tick entire population.
pub fn tick_all(pop: &mut [Cell]) {
    for c in pop.iter_mut() { c.tick(); }
}

/// Apply a rule to entire population. Rule takes current state + index, returns new state.
pub fn apply_rule(pop: &mut [Cell], rule: fn(i8, usize) -> i8) {
    for (i, c) in pop.iter_mut().enumerate() {
        c.set(rule(c.state, i));
    }
}

/// Apply a neighbor rule. Takes own state + left neighbor + right neighbor, returns new state.
pub fn apply_neighbor_rule(pop: &mut [Cell], rule: fn(i8, i8, i8) -> i8) {
    if pop.len() < 3 { return; }
    let n = pop.len();
    let mut next = vec![0i8; n];
    for i in 0..n {
        let left = pop[(i + n - 1) % n].state;
        let right = pop[(i + 1) % n].state;
        next[i] = rule(pop[i].state, left, right);
    }
    for (i, c) in pop.iter_mut().enumerate() {
        c.set(next[i]);
    }
}

/// Initialize a population from a byte slice (any entropy source).
pub fn from_bytes(len: usize, seed: &[u8]) -> Vec<Cell> {
    (0..len).map(|i| Cell::from_byte(seed[i % seed.len()])).collect()
}

/// Simple LCG PRNG for reproducible experiments.
pub struct Rng { state: u64 }
impl Rng {
    pub fn new(seed: u64) -> Self { Self { state: seed } }
    pub fn next_u8(&mut self) -> u8 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 56) as u8
    }
    pub fn next_bool(&mut self) -> bool { self.next_u8() & 1 == 1 }
}

/// Create random population with controlled distribution.
pub fn random_pop(len: usize, frac_neg: f64, frac_zero: f64, rng: &mut Rng) -> Vec<Cell> {
    let n_neg = (len as f64 * frac_neg) as usize;
    let n_zero = (len as f64 * frac_zero) as usize;
    let mut pop = Vec::with_capacity(len);
    for i in 0..len {
        if i < n_neg { pop.push(Cell::neg()); }
        else if i < n_neg + n_zero { pop.push(Cell::zero()); }
        else { pop.push(Cell::pos()); }
    }
    // Shuffle
    for i in (1..pop.len()).rev() {
        let j = (rng.next_u8() as usize) % (i + 1);
        pop.swap(i, j);
    }
    pop
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn cell_new() { assert_eq!(Cell::new(1).state, 1); assert_eq!(Cell::new(-1).state, -1); assert_eq!(Cell::new(0).state, 0); }
    #[test] fn cell_clamp() { assert_eq!(Cell::new(5).state, 1); assert_eq!(Cell::new(-5).state, -1); }
    #[test] fn cell_set_same() { let mut c = Cell::pos(); assert!(!c.set(1)); assert_eq!(c.dwell, 0); }
    #[test] fn cell_set_change() { let mut c = Cell::pos(); assert!(c.set(-1)); assert_eq!(c.flips, 1); }
    #[test] fn cell_tick() { let mut c = Cell::pos(); c.tick(); c.tick(); assert_eq!(c.dwell, 2); }
    #[test] fn cell_flip() { let mut c = Cell::pos(); c.flip(); assert_eq!(c.state, -1); }
    #[test] fn cell_flip_zero() { let mut c = Cell::zero(); c.flip(); assert_eq!(c.state, 0); }
    #[test] fn cell_tunnel() { let mut c = Cell::zero(); assert!(c.tunnel(1)); assert_eq!(c.state, 1); }
    #[test] fn cell_tunnel_fail() { let mut c = Cell::pos(); assert!(!c.tunnel(1)); }
    #[test] fn cell_trap() { let mut c = Cell::pos(); assert!(c.trap()); assert_eq!(c.state, 0); }
    #[test] fn cell_trap_already() { let mut c = Cell::zero(); assert!(!c.trap()); }
    #[test] fn cell_settled() { let mut c = Cell::pos(); for _ in 0..10 { c.tick(); } assert!(c.is_settled(5)); }
    #[test] fn cell_active() { assert!(Cell::pos().is_active()); assert!(!Cell::zero().is_active()); }
    #[test] fn cell_spindle() { assert!(Cell::zero().is_spindle()); assert!(!Cell::pos().is_spindle()); }
    #[test] fn cell_oscillating() { let mut c = Cell::pos(); for _ in 0..10 { c.flip(); } assert!(c.is_oscillating(5)); }
    #[test] fn cell_from_byte() { assert_eq!(Cell::from_byte(0).state, -1); assert_eq!(Cell::from_byte(1).state, 0); assert_eq!(Cell::from_byte(2).state, 1); }
    #[test] fn cell_charge() { assert_eq!(Cell::pos().charge(), 1); assert_eq!(Cell::neg().charge(), -1); assert_eq!(Cell::zero().charge(), 0); }
    #[test] fn cell_abs_charge() { assert_eq!(Cell::neg().abs_charge(), 1); }
    #[test] fn cell_size() { assert!(std::mem::size_of::<Cell>() <= 3); }

    #[test] fn census_test() { let pop = vec![Cell::pos(), Cell::zero(), Cell::neg(), Cell::pos()]; let (n,z,p) = census(&pop); assert_eq!((n,z,p), (1,1,2)); }
    #[test] fn gamma_test() { let pop = vec![Cell::pos(), Cell::pos(), Cell::neg()]; assert_eq!(gamma(&pop), 1); }
    #[test] fn abs_gamma_test() { let pop = vec![Cell::pos(), Cell::neg(), Cell::zero()]; assert_eq!(abs_gamma(&pop), 2); }
    #[test] fn fractions_test() { let pop = vec![Cell::pos(), Cell::neg(), Cell::zero()]; let (f1,f0,fp) = fractions(&pop); assert!((f1 - 1.0/3.0).abs() < 0.01); }
    #[test] fn entropy_max() { let pop = vec![Cell::pos(), Cell::neg(), Cell::zero()]; let h = entropy(&pop); assert!(h > 1.5); }
    #[test] fn entropy_min() { let pop = vec![Cell::pos(), Cell::pos(), Cell::pos()]; let h = entropy(&pop); assert!(h < 0.01); }
    #[test] fn flip_rate_test() { let pop = vec![Cell::pos(), Cell::neg()]; let r = flip_rate(&pop); assert!(r >= 0.0); }
    #[test] fn health_alive() { let mut pop = vec![Cell::pos(), Cell::neg(), Cell::pos(), Cell::neg()]; for c in &mut pop { for _ in 0..5 { c.flip(); } } assert_eq!(health(&pop), Health::Alive); }
    #[test] fn health_dead() { let mut pop = vec![Cell::zero(); 100]; for c in &mut pop { for _ in 0..20 { c.tick(); } } assert_eq!(health(&pop), Health::Dead); }
    #[test] fn health_empty() { let pop: Vec<Cell> = vec![]; assert_eq!(health(&pop), Health::Empty); }

    #[test] fn tick_all_test() { let mut pop = vec![Cell::pos(), Cell::neg()]; tick_all(&mut pop); assert_eq!(pop[0].dwell, 1); }
    #[test] fn apply_rule_test() { let mut pop = vec![Cell::pos(), Cell::neg()]; apply_rule(&mut pop, |_s, _i| 0); assert!(pop.iter().all(|c| c.state == 0)); }
    #[test] fn apply_neighbor_majority() { let mut pop = vec![Cell::pos(), Cell::pos(), Cell::neg(), Cell::pos(), Cell::pos()]; apply_neighbor_rule(&mut pop, |me, l, r| { let sum = me as i32 + l as i32 + r as i32; if sum > 0 { 1 } else if sum < 0 { -1 } else { 0 } }); assert!(pop.iter().all(|c| c.state >= 0)); }
    #[test] fn rng_deterministic() { let mut r1 = Rng::new(42); let mut r2 = Rng::new(42); assert_eq!(r1.next_u8(), r2.next_u8()); assert_eq!(r1.next_u8(), r2.next_u8()); }
    #[test] fn random_pop_distribution() { let mut rng = Rng::new(42); let pop = random_pop(300, 0.33, 0.34, &mut rng); let (n,z,p) = census(&pop); assert!(n > 50 && z > 50 && p > 50); }
    #[test] fn from_bytes_test() { let pop = from_bytes(10, &[0,1,2,0,1,2,0,1,2,0]); assert_eq!(pop.len(), 10); }
}
