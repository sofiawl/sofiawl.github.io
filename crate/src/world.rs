//! The body: Jeff Jones' (2010) agent model of Physarum, steered by the solver.
//!
//! Agents sense a field three times (front, left, right), turn toward the
//! strongest reading, step forward and deposit. The field they read is their
//! own trail plus a "chemoattractant" painted along the tubes the solver is
//! currently thickening, so the veins you see are the brain's decisions.

use crate::rng::Rng;
use crate::solver::{Solver, Terminal};

pub struct Params {
    pub sensor_angle: f32,
    pub sensor_dist: f32,
    pub turn: f32,
    pub deposit: f32,
    pub decay: f32,
    pub chemo_gain: f32,
}

impl Default for Params {
    fn default() -> Self {
        Params { sensor_angle: 0.6, sensor_dist: 6.0, turn: 0.4, deposit: 0.9, decay: 0.9, chemo_gain: 6.0 }
    }
}

pub struct World {
    pub w: usize,
    pub h: usize,
    pub solver: Solver,
    pub params: Params,
    ax: Vec<f32>,
    ay: Vec<f32>,
    aa: Vec<f32>,
    pub active: usize,
    trail: Vec<f32>,
    next: Vec<f32>,
    chemo: Vec<f32>,
    body: Vec<f32>,
    speck: Vec<f32>,
    lut: [u8; 768],
    pub pixels: Vec<u8>,
    rng: Rng,
    pub tick: u64,
}

/// Polycephalum palette: olive agar → ochre sheet → yellow veins → pale highlight.
pub const POLYCEPHALUM: [[u8; 3]; 4] = [[38, 42, 22], [122, 106, 12], [232, 190, 14], [255, 244, 168]];

impl World {
    pub fn new(w: usize, h: usize, agents: usize, terminals: Vec<Terminal>, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let n = w * h;
        let solver = Solver::new(w as f32, h as f32, terminals, &mut rng);
        let (mut ax, mut ay, mut aa) = (Vec::with_capacity(agents), Vec::with_capacity(agents), Vec::with_capacity(agents));
        let tn = solver.terminals.len();
        for i in 0..agents {
            let t = &solver.terminals[i % tn];
            let (r, q) = (rng.f32() * w as f32 * 0.05, rng.f32() * std::f32::consts::TAU);
            ax.push((t.x + q.cos() * r).clamp(0.0, w as f32 - 1.0));
            ay.push((t.y + q.sin() * r).clamp(0.0, h as f32 - 1.0));
            aa.push(q);
        }
        let speck = (0..n).map(|_| if rng.f32() < 0.12 { 0.55 } else { 0.88 + rng.f32() * 0.24 }).collect();
        let mut world = World {
            w,
            h,
            solver,
            params: Params::default(),
            ax,
            ay,
            aa,
            active: agents,
            trail: vec![0.0; n],
            next: vec![0.0; n],
            chemo: vec![0.0; n],
            body: vec![0.0; n],
            speck,
            lut: [0; 768],
            pixels: vec![255; n * 4],
            rng,
            tick: 0,
        };
        // Keep the look independent of resolution and agent count: the reference
        // tuning is 5000 agents on a 320 px wide field.
        let density = agents as f32 / n as f32;
        world.params.deposit = 0.9 * (5000.0 / (320.0 * 215.0)) / density;
        world.params.sensor_dist = 6.0 * w as f32 / 320.0;
        world.set_palette(POLYCEPHALUM);
        world
    }

    pub fn set_palette(&mut self, stops: [[u8; 3]; 4]) {
        let at = [0.0f32, 0.28, 0.62, 1.0];
        for i in 0..256 {
            let v = i as f32 / 255.0;
            let mut k = 0;
            while k < 2 && v > at[k + 1] {
                k += 1;
            }
            let f = ((v - at[k]) / (at[k + 1] - at[k])).clamp(0.0, 1.0);
            for c in 0..3 {
                let (a, b) = (stops[k][c] as f32, stops[k + 1][c] as f32);
                self.lut[i * 3 + c] = (a + (b - a) * f) as u8;
            }
        }
    }

    pub fn agent_capacity(&self) -> usize {
        self.ax.len()
    }

    #[inline]
    fn sense(&self, x: f32, y: f32) -> f32 {
        if x < 0.0 || y < 0.0 || x >= self.w as f32 || y >= self.h as f32 {
            return -4.0;
        }
        let i = y as usize * self.w + x as usize;
        // Saturate the agents' own trail so the solver's tubes keep a say.
        self.trail[i].min(12.0) + self.chemo[i] * self.params.chemo_gain
    }

    /// Paint the solver's live tubes into the chemoattractant field.
    fn paint_tubes(&mut self) {
        self.chemo.iter_mut().for_each(|v| *v = 0.0);
        let s = &self.solver;
        for k in 0..s.eu.len() {
            let d = s.d[k] as f32;
            if d < 0.04 {
                continue;
            }
            let (i, j) = (s.eu[k] as usize, s.ev[k] as usize);
            let (x0, y0, x1, y1) = (s.nx[i], s.ny[i], s.nx[j], s.ny[j]);
            let steps = s.len[k].ceil() as usize;
            for t in 0..=steps {
                let f = t as f32 / steps as f32;
                let x = (x0 + (x1 - x0) * f) as usize;
                let y = (y0 + (y1 - y0) * f) as usize;
                // A soft 3 px wide stroke.
                for (ox, oy, wgt) in [(0i32, 0i32, 1.0f32), (1, 0, 0.5), (-1, 0, 0.5), (0, 1, 0.5), (0, -1, 0.5)] {
                    let (px, py) = (x as i32 + ox, y as i32 + oy);
                    if px >= 0 && py >= 0 && (px as usize) < self.w && (py as usize) < self.h {
                        self.chemo[py as usize * self.w + px as usize] += d * 1.4 * wgt;
                    }
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;
        let pick = self.rng.f64();
        self.solver.step(pick);
        self.paint_tubes();

        let (w, h) = (self.w as f32, self.h as f32);
        let p = &self.params;
        let (sa, sd, turn, dep) = (p.sensor_angle, p.sensor_dist, p.turn, p.deposit);
        for i in 0..self.active {
            let (x, y, a) = (self.ax[i], self.ay[i], self.aa[i]);
            let f = self.sense(x + a.cos() * sd, y + a.sin() * sd);
            let l = self.sense(x + (a - sa).cos() * sd, y + (a - sa).sin() * sd);
            let r = self.sense(x + (a + sa).cos() * sd, y + (a + sa).sin() * sd);
            let mut na = a;
            if f < l && f < r {
                na += if self.rng.f32() < 0.5 { turn } else { -turn };
            } else if l > r {
                na -= turn;
            } else if r > l {
                na += turn;
            }
            na += (self.rng.f32() - 0.5) * 0.25;
            let (mut nx, mut ny) = (x + na.cos(), y + na.sin());
            if nx < 0.0 || ny < 0.0 || nx >= w || ny >= h {
                na += std::f32::consts::PI + (self.rng.f32() - 0.5);
                nx = nx.clamp(0.0, w - 1.0);
                ny = ny.clamp(0.0, h - 1.0);
            }
            // Rarely, an agent is re-born at a page. Strays starve, the network stays fed.
            if self.rng.f32() < 0.003 {
                let tn = self.solver.terminals.len();
                let t = &self.solver.terminals[(self.rng.f32() * tn as f32) as usize % tn];
                nx = t.x;
                ny = t.y;
                na = self.rng.f32() * std::f32::consts::TAU;
            }
            self.ax[i] = nx;
            self.ay[i] = ny;
            self.aa[i] = na;
            self.trail[ny as usize * self.w + nx as usize] += dep;
        }

        // Food at every terminal (scaled by its live weight).
        let rad = (self.w as f32 / 107.0).max(2.0) as i32;
        for t in &self.solver.terminals {
            let amt = 2.2 * t.weight as f32;
            for dy in -rad..=rad {
                for dx in -rad..=rad {
                    let (x, y) = (t.x as i32 + dx, t.y as i32 + dy);
                    if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
                        self.trail[y as usize * self.w + x as usize] += amt;
                    }
                }
            }
        }

        // 3x3 box blur + decay, then the slow "plasmodium sheet" memory.
        let k = self.params.decay;
        let (wu, hu) = (self.w, self.h);
        for y in 0..hu {
            let ym = if y == 0 { 0 } else { y - 1 } * wu;
            let yp = if y + 1 == hu { y } else { y + 1 } * wu;
            let y0 = y * wu;
            for x in 0..wu {
                let xm = if x == 0 { 0 } else { x - 1 };
                let xp = if x + 1 == wu { x } else { x + 1 };
                let t = &self.trail;
                let blur = (t[ym + xm] + t[ym + x] + t[ym + xp] + t[y0 + xm] + t[y0 + x] + t[y0 + xp] + t[yp + xm] + t[yp + x] + t[yp + xp]) / 9.0;
                // Partial diffusion keeps veins thin.
                self.next[y0 + x] = (blur * 0.55 + t[y0 + x] * 0.45) * k;
            }
        }
        std::mem::swap(&mut self.trail, &mut self.next);
        self.render();
    }

    fn render(&mut self) {
        for i in 0..self.w * self.h {
            let v = self.trail[i].max(0.0);
            self.body[i] = self.body[i] * 0.985 + v * 0.015;
            let vv = (v / 14.0).sqrt();
            let bb = (self.body[i] / 5.0).sqrt() * 0.6;
            let mut c = vv.max(bb);
            if vv < 0.55 {
                c *= self.speck[i];
            }
            let li = ((c * 255.0) as usize).min(255) * 3;
            let o = i * 4;
            self.pixels[o] = self.lut[li];
            self.pixels[o + 1] = self.lut[li + 1];
            self.pixels[o + 2] = self.lut[li + 2];
        }
    }
}
