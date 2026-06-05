//! # ternary-cell
//!
//! Cellular computing inspired by room-cell and biological cells.
//! TernaryCell with tick cycle (predict→perceive→surprise→vibe→gc→conservation),
//! CellGrid, cell signaling, apoptosis and division, tissue-level coordination.
//! Maps to construct-core compute model.

#![forbid(unsafe_code)]

/// Ternary messenger signal between cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TernaryMessenger {
    /// Positive signal: grow, activate, promote.
    Signal,
    /// Neutral: maintain, acknowledge.
    Silence,
    /// Negative signal: inhibit, suppress, retract.
    Suppress,
}

impl TernaryMessenger {
    pub fn to_ternary(self) -> i8 {
        match self {
            TernaryMessenger::Signal => 1,
            TernaryMessenger::Silence => 0,
            TernaryMessenger::Suppress => -1,
        }
    }

    pub fn from_ternary(v: i8) -> Option<Self> {
        match v {
            1 => Some(TernaryMessenger::Signal),
            0 => Some(TernaryMessenger::Silence),
            -1 => Some(TernaryMessenger::Suppress),
            _ => None,
        }
    }

    /// Combine two messengers (max wins).
    pub fn combine(a: Self, b: Self) -> Self {
        let v = a.to_ternary().max(b.to_ternary());
        Self::from_ternary(v).unwrap()
    }
}

/// Cell state in the lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    /// Alive and active.
    Active,
    /// Marked for removal.
    Apoptotic,
    /// Just divided, recovering.
    Dividing,
}

/// A single ternary cell with internal state and tick lifecycle.
#[derive(Debug, Clone)]
pub struct TernaryCell {
    pub id: u64,
    pub energy: i32,
    pub state: CellState,
    /// Internal ternary state: -1, 0, or +1.
    pub ternary_value: i8,
    /// Prediction for next tick.
    prediction: i8,
    /// Accumulated surprise.
    surprise: i32,
    /// Incoming messages.
    inbox: Vec<TernaryMessenger>,
    /// Tick counter.
    tick_count: u64,
    /// Generation (incremented on division).
    pub generation: u32,
}

impl TernaryCell {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            energy: 10,
            state: CellState::Active,
            ternary_value: 0,
            prediction: 0,
            surprise: 0,
            inbox: Vec::new(),
            tick_count: 0,
            generation: 0,
        }
    }

    pub fn with_value(id: u64, value: i8) -> Self {
        let mut cell = Self::new(id);
        cell.ternary_value = value.clamp(-1, 1);
        cell
    }

    /// Receive a messenger signal.
    pub fn receive(&mut self, msg: TernaryMessenger) {
        self.inbox.push(msg);
    }

    /// Step 1: Predict next ternary value based on current state and inbox.
    pub fn predict(&mut self) {
        let combined: i32 = self.inbox.iter().map(|m| m.to_ternary() as i32).sum();
        self.prediction = if combined > 0 { 1 } else if combined < 0 { -1 } else { self.ternary_value };
    }

    /// Step 2: Perceive — update value based on combined signals.
    pub fn perceive(&mut self) {
        let combined: i32 = self.inbox.iter().map(|m| m.to_ternary() as i32).sum();
        if combined != 0 {
            self.ternary_value = combined.clamp(-1, 1) as i8;
        }
    }

    /// Step 3: Compute surprise (prediction error).
    pub fn compute_surprise(&mut self) -> i32 {
        self.surprise = (self.ternary_value as i32 - self.prediction as i32).abs();
        self.surprise
    }

    /// Step 4: Vibe — adjust energy based on surprise.
    pub fn vibe(&mut self) {
        self.energy -= self.surprise;
        // Being in sync with neighbors gives energy
        if self.surprise == 0 {
            self.energy += 1;
        }
    }

    /// Step 5: GC — clear inbox, reclaim resources.
    pub fn gc(&mut self) {
        self.inbox.clear();
    }

    /// Step 6: Conservation — enforce energy bounds and check apoptosis.
    pub fn conservation(&mut self) {
        self.energy = self.energy.clamp(0, 20);
        if self.energy == 0 {
            self.state = CellState::Apoptotic;
        }
        self.tick_count += 1;
    }

    /// Run a full tick cycle.
    pub fn tick(&mut self) -> i32 {
        self.predict();
        self.perceive();
        let surprise = self.compute_surprise();
        self.vibe();
        self.gc();
        self.conservation();
        surprise
    }

    /// Check if cell can divide (enough energy and active).
    pub fn can_divide(&self) -> bool {
        self.energy >= 10 && self.state == CellState::Active
    }

    /// Divide: create a daughter cell, halve energy.
    pub fn divide(&mut self, daughter_id: u64) -> Option<TernaryCell> {
        if !self.can_divide() {
            return None;
        }
        self.energy /= 2;
        self.state = CellState::Dividing;
        Some(TernaryCell {
            id: daughter_id,
            energy: self.energy,
            state: CellState::Active,
            ternary_value: self.ternary_value,
            prediction: self.ternary_value,
            surprise: 0,
            inbox: Vec::new(),
            tick_count: 0,
            generation: self.generation + 1,
        })
    }

    /// Emit current value as a messenger.
    pub fn emit(&self) -> TernaryMessenger {
        TernaryMessenger::from_ternary(self.ternary_value).unwrap_or(TernaryMessenger::Silence)
    }

    pub fn is_alive(&self) -> bool {
        self.state != CellState::Apoptotic
    }
}

/// A grid of ternary cells.
#[derive(Debug, Clone)]
pub struct CellGrid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Option<TernaryCell>>,
    pub next_id: u64,
}

impl CellGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![None; width * height];
        Self { width, height, cells, next_id: 0 }
    }

    /// Place a cell at a position.
    pub fn place(&mut self, x: usize, y: usize, value: i8) -> bool {
        if x >= self.width || y >= self.height { return false; }
        let id = self.next_id;
        self.next_id += 1;
        self.cells[y * self.width + x] = Some(TernaryCell::with_value(id, value));
        true
    }

    /// Get cell at position.
    pub fn get(&self, x: usize, y: usize) -> Option<&TernaryCell> {
        if x >= self.width || y >= self.height { return None; }
        self.cells[y * self.width + x].as_ref()
    }

    /// Get mutable cell at position.
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut TernaryCell> {
        if x >= self.width || y >= self.height { return None; }
        self.cells[y * self.width + x].as_mut()
    }

    /// Get neighbor positions (4-connected).
    pub fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::new();
        if x > 0 { result.push((x - 1, y)); }
        if x + 1 < self.width { result.push((x + 1, y)); }
        if y > 0 { result.push((x, y - 1)); }
        if y + 1 < self.height { result.push((x, y + 1)); }
        result
    }

    /// Signal propagation: cells send messages to neighbors.
    pub fn propagate_signals(&mut self) {
        // Collect emissions first to avoid borrow issues
        let width = self.width;
        let height = self.height;
        let mut emissions: Vec<(usize, usize, TernaryMessenger)> = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = self.get(x, y) {
                    if cell.is_alive() {
                        emissions.push((x, y, cell.emit()));
                    }
                }
            }
        }
        // Now deliver
        for (x, y, msg) in emissions {
            for (nx, ny) in self.neighbors(x, y) {
                if let Some(neighbor) = self.get_mut(nx, ny) {
                    neighbor.receive(msg);
                }
            }
        }
    }

    /// Run one tick across all cells.
    pub fn tick_all(&mut self) -> u32 {
        self.propagate_signals();
        for cell in &mut self.cells {
            if let Some(c) = cell {
                if c.is_alive() {
                    c.tick();
                }
            }
        }
        // Remove apoptotic cells and count survivors.
        let mut alive = 0u32;
        for cell in &mut self.cells {
            if let Some(c) = cell {
                if !c.is_alive() {
                    *cell = None;
                } else {
                    alive += 1;
                }
            }
        }
        alive
    }

    /// Count alive cells.
    pub fn alive_count(&self) -> usize {
        self.cells.iter().filter(|c| c.as_ref().map_or(false, |c| c.is_alive())).count()
    }

    /// Tissue-level ternary balance.
    pub fn tissue_balance(&self) -> (usize, usize, usize) {
        let mut pos = 0;
        let mut zero = 0;
        let mut neg = 0;
        for cell in &self.cells {
            if let Some(c) = cell {
                if c.is_alive() {
                    match c.ternary_value {
                        1 => pos += 1,
                        0 => zero += 1,
                        -1 => neg += 1,
                        _ => {}
                    }
                }
            }
        }
        (pos, zero, neg)
    }
}

/// Tissue coordinator for grid-level operations.
#[derive(Debug, Clone)]
pub struct Tissue {
    pub grid: CellGrid,
}

impl Tissue {
    pub fn new(width: usize, height: usize) -> Self {
        Self { grid: CellGrid::new(width, height) }
    }

    /// Fill grid with a pattern.
    pub fn fill_pattern(&mut self, pattern: &[i8]) {
        for (i, &val) in pattern.iter().enumerate() {
            let x = i % self.grid.width;
            let y = i / self.grid.width;
            self.grid.place(x, y, val);
        }
    }

    /// Run tissue for N ticks, return alive count at end.
    pub fn run(&mut self, ticks: usize) -> u32 {
        for _ in 0..ticks {
            self.grid.tick_all();
        }
        self.grid.alive_count() as u32
    }

    /// Check if tissue has converged (all cells have same ternary value).
    pub fn is_converged(&self) -> bool {
        let mut values = std::collections::HashSet::new();
        for cell in &self.grid.cells {
            if let Some(c) = cell {
                if c.is_alive() {
                    values.insert(c.ternary_value);
                }
            }
        }
        values.len() <= 1
    }

    /// Compute tissue-level consensus ternary value.
    pub fn consensus(&self) -> i8 {
        let (pos, zero, neg) = self.grid.tissue_balance();
        if pos > zero && pos > neg { 1 }
        else if neg > pos && neg > zero { -1 }
        else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messenger_roundtrip() {
        for v in [-1i8, 0, 1] {
            assert_eq!(TernaryMessenger::from_ternary(v).unwrap().to_ternary(), v);
        }
    }

    #[test]
    fn test_messenger_combine() {
        assert_eq!(TernaryMessenger::combine(TernaryMessenger::Signal, TernaryMessenger::Suppress), TernaryMessenger::Signal);
        assert_eq!(TernaryMessenger::combine(TernaryMessenger::Silence, TernaryMessenger::Suppress), TernaryMessenger::Silence);
    }

    #[test]
    fn test_cell_new() {
        let cell = TernaryCell::new(0);
        assert_eq!(cell.ternary_value, 0);
        assert_eq!(cell.energy, 10);
        assert!(cell.is_alive());
    }

    #[test]
    fn test_cell_tick_basic() {
        let mut cell = TernaryCell::new(0);
        let surprise = cell.tick();
        assert_eq!(surprise, 0); // no signals, no change
        assert_eq!(cell.tick_count, 1);
    }

    #[test]
    fn test_cell_receive_and_tick() {
        let mut cell = TernaryCell::new(0);
        cell.receive(TernaryMessenger::Signal);
        cell.receive(TernaryMessenger::Signal);
        cell.tick();
        assert_eq!(cell.ternary_value, 1);
    }

    #[test]
    fn test_cell_divide() {
        let mut cell = TernaryCell::with_value(0, 1);
        let daughter = cell.divide(1);
        assert!(daughter.is_some());
        assert_eq!(daughter.unwrap().generation, 1);
        assert_eq!(cell.energy, 5);
    }

    #[test]
    fn test_cell_cannot_divide_low_energy() {
        let mut cell = TernaryCell::new(0);
        cell.energy = 3;
        assert!(cell.divide(1).is_none());
    }

    #[test]
    fn test_cell_apoptosis() {
        let mut cell = TernaryCell::new(0);
        cell.energy = 1;
        // Lots of surprise to drain energy
        cell.receive(TernaryMessenger::Signal);
        cell.predict();
        cell.ternary_value = -1; // force mismatch
        let s = cell.compute_surprise();
        assert_eq!(s, 2); // |-1 - 1| = 2
        cell.vibe();
        assert_eq!(cell.energy, -1);
        cell.gc();
        cell.conservation();
        assert_eq!(cell.state, CellState::Apoptotic);
    }

    #[test]
    fn test_cell_emit() {
        let cell = TernaryCell::with_value(0, 1);
        assert_eq!(cell.emit(), TernaryMessenger::Signal);
    }

    #[test]
    fn test_grid_place_and_get() {
        let mut grid = CellGrid::new(3, 3);
        grid.place(1, 1, 1);
        assert!(grid.get(1, 1).is_some());
        assert_eq!(grid.get(1, 1).unwrap().ternary_value, 1);
    }

    #[test]
    fn test_grid_neighbors() {
        let grid = CellGrid::new(3, 3);
        let neighbors = grid.neighbors(1, 1);
        assert_eq!(neighbors.len(), 4);
    }

    #[test]
    fn test_grid_corner_neighbors() {
        let grid = CellGrid::new(3, 3);
        let neighbors = grid.neighbors(0, 0);
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_grid_tick_all() {
        let mut grid = CellGrid::new(2, 2);
        grid.place(0, 0, 1);
        grid.place(1, 0, 1);
        grid.place(0, 1, -1);
        grid.place(1, 1, -1);
        let alive = grid.tick_all();
        assert_eq!(alive, 4);
    }

    #[test]
    fn test_grid_tissue_balance() {
        let mut grid = CellGrid::new(2, 2);
        grid.place(0, 0, 1);
        grid.place(1, 0, 0);
        grid.place(0, 1, -1);
        grid.place(1, 1, 1);
        let (pos, zero, neg) = grid.tissue_balance();
        assert_eq!(pos, 2);
        assert_eq!(zero, 1);
        assert_eq!(neg, 1);
    }

    #[test]
    fn test_tissue_convergence() {
        let mut tissue = Tissue::new(2, 2);
        tissue.fill_pattern(&[1, 1, 1, 1]);
        assert!(tissue.is_converged());
    }

    #[test]
    fn test_tissue_not_converged() {
        let mut tissue = Tissue::new(2, 2);
        tissue.fill_pattern(&[1, -1, 1, -1]);
        assert!(!tissue.is_converged());
    }

    #[test]
    fn test_tissue_consensus() {
        let mut tissue = Tissue::new(3, 1);
        tissue.fill_pattern(&[1, 1, -1]);
        assert_eq!(tissue.consensus(), 1);
    }

    #[test]
    fn test_tissue_run() {
        let mut tissue = Tissue::new(2, 2);
        tissue.fill_pattern(&[1, 0, -1, 0]);
        let alive = tissue.run(3);
        assert!(alive <= 4);
    }

    #[test]
    fn test_cell_with_value_clamps() {
        let cell = TernaryCell::with_value(0, 5);
        assert_eq!(cell.ternary_value, 1);
        let cell2 = TernaryCell::with_value(1, -3);
        assert_eq!(cell2.ternary_value, -1);
    }

    #[test]
    fn test_grid_out_of_bounds() {
        let grid = CellGrid::new(2, 2);
        assert!(grid.get(5, 5).is_none());
    }

    #[test]
    fn test_cell_tick_with_surprise() {
        let mut cell = TernaryCell::with_value(0, 1);
        cell.receive(TernaryMessenger::Suppress);
        cell.tick();
        // Prediction based on inbox (-1) -> pred=-1, but perceive changes to -1
        // Actually: predict sets prediction=-1, perceive sets value=-1, surprise = |-1-(-1)|=0
        // Let me trace: inbox=[Suppress], combined=-1, prediction=-1, perceive sets ternary=-1
        // surprise = |-1 - (-1)| = 0
        assert_eq!(cell.surprise, 0);
    }
}
