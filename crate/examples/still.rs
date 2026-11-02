//! Render one grown frame of the organism: `cargo run --release --example still -- out.ppm`
//! Used for design mockups and as the reduced-motion / social preview image.

use physarum::Engine;
use std::io::Write;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "still.ppm".into());
    let (w, h) = (416usize, 280usize);
    // work, side projects, notes, résumé, say hi (same fractions as web/config.js)
    let pages = [(0.72, 0.19, 1.2), (0.92, 0.56, 0.9), (0.67, 0.88, 1.0), (0.40, 0.84, 0.8), (0.50, 0.49, 0.6)];
    let terms: Vec<f32> = pages.iter().flat_map(|&(x, y, wt)| [x * w as f32, y * h as f32, wt]).collect();
    let mut e = Engine::new(w, h, 20_000, &terms, "sofia");
    e.advance(900);
    let px = unsafe { std::slice::from_raw_parts(e.pixels_ptr(), w * h * 4) };
    let mut f = std::fs::File::create(&out).unwrap();
    write!(f, "P6 {w} {h} 255\n").unwrap();
    for p in px.chunks_exact(4) {
        f.write_all(&p[..3]).unwrap();
    }
}
