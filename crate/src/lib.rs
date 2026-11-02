//! physarum: a slime mold that routes a personal website.
//!
//! - `solver`: the brain (Tero's adaptive-network model, solved with CG).
//! - `world`: the body (Jones agents steered by the solver's tubes) + renderer.
//! - `Engine`: the small API the browser talks to, exported with wasm-bindgen.

pub mod rng;
pub mod solver;
pub mod world;

use solver::Terminal;
use wasm_bindgen::prelude::*;
use world::World;

#[wasm_bindgen]
pub struct Engine {
    world: World,
}

#[wasm_bindgen]
impl Engine {
    /// `terminals` is a flat list of (x, y, weight) in simulation pixels.
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize, agents: usize, terminals: &[f32], seed: &str) -> Engine {
        let terms = terminals
            .chunks_exact(3)
            .map(|c| Terminal { x: c[0], y: c[1], weight: c[2] as f64 })
            .collect();
        Engine { world: World::new(width, height, agents, terms, rng::hash_str(seed)) }
    }

    pub fn step(&mut self) {
        self.world.step();
    }

    /// Run `n` steps without anything else (warm-up, reduced-motion still frame).
    pub fn advance(&mut self, n: u32) {
        for _ in 0..n {
            self.world.step();
        }
    }

    /// RGBA pixels live in wasm memory; JS views them without copying.
    pub fn pixels_ptr(&self) -> *const u8 {
        self.world.pixels.as_ptr()
    }

    pub fn width(&self) -> usize {
        self.world.w
    }

    pub fn height(&self) -> usize {
        self.world.h
    }

    /// Live food weight for terminal `i` (wire this to real activity feeds).
    pub fn set_weight(&mut self, i: usize, weight: f64) {
        if let Some(t) = self.world.solver.terminals.get_mut(i) {
            t.weight = weight.max(0.05);
        }
    }

    /// Adaptive quality: lower the number of simulated agents on slow devices.
    pub fn set_active_agents(&mut self, n: usize) {
        self.world.active = n.clamp(500, self.world.agent_capacity());
    }

    pub fn active_agents(&self) -> usize {
        self.world.active
    }

    pub fn set_palette(&mut self, rgb12: &[u8]) {
        if rgb12.len() == 12 {
            let s = |i: usize| [rgb12[i * 3], rgb12[i * 3 + 1], rgb12[i * 3 + 2]];
            self.world.set_palette([s(0), s(1), s(2), s(3)]);
        }
    }

    // ---- debug view ("show the math") ----

    /// Flat (x0, y0, x1, y1, D) per tube.
    pub fn tubes(&self) -> Vec<f32> {
        let s = &self.world.solver;
        let mut out = Vec::with_capacity(s.edges() * 5);
        for k in 0..s.edges() {
            let (i, j) = (s.eu[k] as usize, s.ev[k] as usize);
            out.extend_from_slice(&[s.nx[i], s.ny[i], s.nx[j], s.ny[j], s.d[k] as f32]);
        }
        out
    }

    /// [tick, nodes, tubes, alive tubes, network length, CG residual, CG steps, source index]
    pub fn stats(&self) -> Vec<f64> {
        let s = &self.world.solver;
        let (alive, length) = s.alive(0.04);
        vec![
            self.world.tick as f64,
            s.nodes() as f64,
            s.edges() as f64,
            alive as f64,
            length,
            s.last_residual,
            s.last_cg_steps as f64,
            s.last_source as f64,
        ]
    }
}
