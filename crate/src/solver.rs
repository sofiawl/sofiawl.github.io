//! The brain: Tero's Physarum solver on a random planar-ish graph.
//!
//! Every edge is a tube with conductance `D` and length `L`. Each step one
//! terminal (a page of the site) becomes the food source and the others become
//! sinks, weighted by how "hungry" they are. We solve Kirchhoff's law for the
//! node pressures `p`, read off the flux `Q` through every tube, and let tubes
//! thicken with use and wither without it:
//!
//! ```text
//!   Σ_j (D_ij / L_ij)(p_i − p_j) = b_i          (flow conservation)
//!   Q_ij = (D_ij / L_ij)(p_i − p_j)             (Poiseuille flow)
//!   dD_ij/dt = f(|Q_ij|) − D_ij,  f(q) = q^γ / (1 + q^γ)
//! ```
//!
//! References:
//! - Tero, Kobayashi, Nakagaki (2007), J. Theor. Biol. 244: "A mathematical model
//!   for adaptive transport network in path finding by true slime mold".
//! - Tero et al. (2010), Science 327: "Rules for biologically inspired adaptive
//!   network design" (the Tokyo rail experiment; sigmoidal f with γ > 1).
//! - Bonifaci, Mehlhorn, Varma (2012), J. Theor. Biol.: "Physarum can compute
//!   shortest paths" (convergence proof).

use crate::rng::Rng;

pub struct Terminal {
    pub x: f32,
    pub y: f32,
    /// Food weight. On the live site this comes from real activity feeds.
    pub weight: f64,
}

pub struct Solver {
    pub nx: Vec<f32>,
    pub ny: Vec<f32>,
    pub eu: Vec<u32>,
    pub ev: Vec<u32>,
    pub len: Vec<f64>,
    pub d: Vec<f64>,
    pub q: Vec<f64>,
    pub terminals: Vec<Terminal>,
    pub last_source: usize,
    pub last_residual: f64,
    pub last_cg_steps: u32,
    gamma: f64,
    dt: f64,
    // scratch buffers for conjugate gradient
    c: Vec<f64>,
    p: Vec<f64>,
    b: Vec<f64>,
    r: Vec<f64>,
    s: Vec<f64>,
    ap: Vec<f64>,
}

impl Solver {
    /// Scatter nodes with dart throwing (min spacing), connect each to its
    /// nearest neighbours, and place the terminals as the first nodes.
    pub fn new(w: f32, h: f32, terminals: Vec<Terminal>, rng: &mut Rng) -> Self {
        let spacing = (w.min(h) / 15.0).max(6.0);
        let reach = spacing * 2.4;
        let margin = spacing * 0.4;
        let mut nx: Vec<f32> = terminals.iter().map(|t| t.x).collect();
        let mut ny: Vec<f32> = terminals.iter().map(|t| t.y).collect();
        for _ in 0..6000 {
            let x = margin + rng.f32() * (w - 2.0 * margin);
            let y = margin + rng.f32() * (h - 2.0 * margin);
            let free = nx.iter().zip(&ny).all(|(&a, &b)| {
                let (dx, dy) = (a - x, b - y);
                dx * dx + dy * dy >= spacing * spacing
            });
            if free {
                nx.push(x);
                ny.push(y);
            }
        }
        let n = nx.len();
        let (mut eu, mut ev, mut len) = (Vec::new(), Vec::new(), Vec::new());
        let mut seen = std::collections::HashSet::new();
        for i in 0..n {
            let mut nb: Vec<(f32, usize)> = (0..n)
                .filter(|&j| j != i)
                .map(|j| (((nx[i] - nx[j]).powi(2) + (ny[i] - ny[j]).powi(2)).sqrt(), j))
                .filter(|&(dist, _)| dist < reach)
                .collect();
            nb.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            for &(dist, j) in nb.iter().take(6) {
                let key = (i.min(j), i.max(j));
                if seen.insert(key) {
                    eu.push(key.0 as u32);
                    ev.push(key.1 as u32);
                    len.push(dist as f64);
                }
            }
        }
        let e = eu.len();
        let d = (0..e).map(|_| 0.5 + rng.f64() * 0.1).collect();
        Solver {
            nx,
            ny,
            eu,
            ev,
            len,
            d,
            q: vec![0.0; e],
            terminals,
            last_source: 0,
            last_residual: 0.0,
            last_cg_steps: 0,
            gamma: 1.8,
            dt: 0.12,
            c: vec![0.0; e],
            p: vec![0.0; n],
            b: vec![0.0; n],
            r: vec![0.0; n],
            s: vec![0.0; n],
            ap: vec![0.0; n],
        }
    }

    pub fn nodes(&self) -> usize {
        self.nx.len()
    }

    pub fn edges(&self) -> usize {
        self.eu.len()
    }

    /// y = L x, the weighted graph Laplacian, with one sink tied to ground so the
    /// system is positive definite. Because Σb = 0 the ground carries no current,
    /// so this pins the pressure level without leaking food out of the network.
    fn laplacian(&self, x: &[f64], y: &mut [f64], ground: usize) {
        y.iter_mut().for_each(|v| *v = 0.0);
        y[ground] = x[ground];
        for k in 0..self.eu.len() {
            let (i, j) = (self.eu[k] as usize, self.ev[k] as usize);
            let f = self.c[k] * (x[i] - x[j]);
            y[i] += f;
            y[j] -= f;
        }
    }

    /// One adaptation step. `pick` in [0,1) chooses the food source,
    /// proportional to terminal weights.
    pub fn step(&mut self, pick: f64) {
        let total: f64 = self.terminals.iter().map(|t| t.weight).sum();
        let mut u = pick * total;
        let mut src = 0;
        while src + 1 < self.terminals.len() && u > self.terminals[src].weight {
            u -= self.terminals[src].weight;
            src += 1;
        }
        let sink_total = total - self.terminals[src].weight;
        self.b.iter_mut().for_each(|v| *v = 0.0);
        for (k, t) in self.terminals.iter().enumerate() {
            self.b[k] = if k == src { 1.0 } else { -t.weight / sink_total };
        }
        for k in 0..self.eu.len() {
            self.c[k] = self.d[k] / self.len[k];
        }
        let ground = if src == 0 { 1 } else { 0 };

        // Conjugate gradient on L p = b.
        self.p.iter_mut().for_each(|v| *v = 0.0);
        self.r.copy_from_slice(&self.b);
        self.s.copy_from_slice(&self.b);
        let mut rs: f64 = self.r.iter().map(|v| v * v).sum();
        let mut steps = 0;
        let mut ap = std::mem::take(&mut self.ap);
        while steps < 400 && rs > 1e-14 {
            self.laplacian(&self.s, &mut ap, ground);
            let sap: f64 = self.s.iter().zip(&ap).map(|(a, b)| a * b).sum();
            let alpha = rs / sap;
            let mut rn = 0.0;
            for i in 0..self.p.len() {
                self.p[i] += alpha * self.s[i];
                self.r[i] -= alpha * ap[i];
                rn += self.r[i] * self.r[i];
            }
            let beta = rn / rs;
            rs = rn;
            for i in 0..self.s.len() {
                self.s[i] = self.r[i] + beta * self.s[i];
            }
            steps += 1;
        }
        self.ap = ap;

        // Flux and tube adaptation.
        for k in 0..self.eu.len() {
            let (i, j) = (self.eu[k] as usize, self.ev[k] as usize);
            self.q[k] = self.c[k] * (self.p[i] - self.p[j]);
            let g = (self.q[k].abs() * 2.0).powf(self.gamma);
            self.d[k] = (self.d[k] + self.dt * (g / (1.0 + g) - self.d[k])).max(1e-6);
        }
        self.last_source = src;
        self.last_residual = rs.sqrt();
        self.last_cg_steps = steps;
    }

    /// Number of tubes still carrying meaningful flow, and their total length.
    pub fn alive(&self, threshold: f64) -> (usize, f64) {
        self.d
            .iter()
            .zip(&self.len)
            .filter(|(d, _)| **d > threshold)
            .fold((0, 0.0), |(n, l), (_, len)| (n + 1, l + len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_solver() -> Solver {
        let mut rng = Rng::new(7);
        let terms = vec![
            Terminal { x: 20.0, y: 100.0, weight: 1.0 },
            Terminal { x: 280.0, y: 100.0, weight: 1.0 },
        ];
        Solver::new(300.0, 200.0, terms, &mut rng)
    }

    #[test]
    fn flow_is_conserved() {
        let mut s = line_solver();
        s.step(0.1);
        // Net outflow at the source equals the injected unit of food.
        let mut out = vec![0.0; s.nodes()];
        for k in 0..s.edges() {
            out[s.eu[k] as usize] += s.q[k];
            out[s.ev[k] as usize] -= s.q[k];
        }
        assert!((out[s.last_source] - 1.0).abs() < 1e-4, "source outflow {}", out[s.last_source]);
        assert!(s.last_residual < 1e-5);
    }

    #[test]
    fn network_prunes_to_a_short_path() {
        let mut s = line_solver();
        let (alive0, _) = s.alive(0.04);
        for i in 0..800 {
            s.step((i as f64 * 0.618).fract());
        }
        let (alive, length) = s.alive(0.04);
        assert!(alive < alive0 / 4, "expected pruning: {alive0} -> {alive}");
        // The surviving tubes should be close to the straight-line distance (260).
        assert!(length < 260.0 * 1.6, "network too long: {length}");
    }

    #[test]
    fn every_page_stays_connected() {
        let mut rng = Rng::new(42);
        let pts = [(288.0, 51.0, 1.2), (368.0, 151.0, 0.9), (268.0, 237.0, 1.0), (164.0, 231.0, 0.8), (200.0, 132.0, 0.6)];
        let terms = pts.iter().map(|&(x, y, weight)| Terminal { x, y, weight }).collect();
        let mut s = Solver::new(400.0, 269.0, terms, &mut rng);
        for _ in 0..1500 {
            let pick = rng.f64();
            s.step(pick);
        }
        for t in 0..5 {
            let best = (0..s.edges())
                .filter(|&k| s.eu[k] as usize == t || s.ev[k] as usize == t)
                .map(|k| s.d[k])
                .fold(0.0, f64::max);
            assert!(best > 0.04, "terminal {t} lost its tubes (max D {best})");
        }
    }
}
