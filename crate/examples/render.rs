//! Native preview + benchmark: `cargo run --release --example render`
//! Writes frames as PPM to ./frames and prints the time per step.

use physarum::Engine;
use std::io::Write;

fn main() {
    let (w, h) = (400usize, 269usize);
    let terms: Vec<f32> = [(0.72, 0.19, 1.2), (0.92, 0.56, 0.9), (0.67, 0.88, 1.0), (0.41, 0.86, 0.8), (0.50, 0.49, 0.6)]
        .iter()
        .flat_map(|&(x, y, wt)| [x * w as f32, y * h as f32, wt])
        .collect();
    let mut e = Engine::new(w, h, 20_000, &terms, "sofia");
    std::fs::create_dir_all("frames").unwrap();
    let t0 = std::time::Instant::now();
    for i in 1..=900 {
        e.step();
        if i % 300 == 0 {
            let px = unsafe { std::slice::from_raw_parts(e.pixels_ptr(), w * h * 4) };
            let mut f = std::fs::File::create(format!("frames/f{i:04}.ppm")).unwrap();
            write!(f, "P6 {w} {h} 255\n").unwrap();
            for p in px.chunks_exact(4) {
                f.write_all(&p[..3]).unwrap();
            }
        }
    }
    let tubes = e.tubes();
    let mut f = std::fs::File::create("frames/tubes.txt").unwrap();
    for c in tubes.chunks_exact(5) { writeln!(f, "{} {} {} {} {}", c[0], c[1], c[2], c[3], c[4]).unwrap(); }
    let ms = t0.elapsed().as_secs_f64() * 1000.0 / 900.0;
    println!("{ms:.2} ms/step (native, 20k agents, {w}x{h})");
    println!("stats {:?}", e.stats());
}
