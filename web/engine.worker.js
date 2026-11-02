// Runs the Rust/WebAssembly organism off the main thread, so the page never stutters.
import init, { Engine } from './pkg/physarum.js';

let engine = null;
let memory = null;
let ctx = null;        // OffscreenCanvas context, when the browser supports it
let image = null;
let running = false;
let animated = true;
let looping = false;  // guards against two loops after a quick hide/show
let debug = false;
let frame = 0;
let slow = 0;
let width = 0;
let height = 0;

const nextFrame = self.requestAnimationFrame
  ? (fn) => self.requestAnimationFrame(fn)
  : (fn) => setTimeout(() => fn(performance.now()), 16);

function draw() {
  const view = new Uint8ClampedArray(memory.buffer, engine.pixels_ptr(), width * height * 4);
  if (ctx) {
    image.data.set(view);
    ctx.putImageData(image, 0, 0);
  } else {
    const copy = view.slice();
    self.postMessage({ type: 'frame', width, height, pixels: copy }, [copy.buffer]);
  }
}

function report() {
  const stats = Array.from(engine.stats());
  stats.push(engine.active_agents());
  self.postMessage({ type: 'stats', stats, tubes: debug ? engine.tubes() : null });
}

function loop() {
  if (!running) { looping = false; return; }
  looping = true;
  const t0 = performance.now();
  engine.step();
  const ms = performance.now() - t0;
  draw();

  // Adaptive quality: if steps keep taking longer than ~12 ms, shed 25% of agents.
  slow = ms > 12 ? slow + 1 : Math.max(0, slow - 1);
  if (slow > 45) {
    engine.set_active_agents(Math.floor(engine.active_agents() * 0.75));
    slow = 0;
  }
  if (++frame % 12 === 0) report();
  nextFrame(loop);
}

self.onmessage = async ({ data: m }) => {
  if (m.type === 'init') {
    try {
      const wasm = await init();
      memory = wasm.memory;
      width = m.width;
      height = m.height;
      engine = new Engine(width, height, m.agents, new Float32Array(m.terminals), m.seed);
      engine.set_palette(new Uint8Array(m.palette.flat()));
      if (m.canvas) {
        ctx = m.canvas.getContext('2d');
        image = new ImageData(width, height);
      }
      if (m.reducedMotion) {
        // One still frame of a grown network instead of an animation.
        animated = false;
        engine.advance(450);
        draw();
        report();
        self.postMessage({ type: 'ready', animated: false });
        return;
      }
      engine.advance(40);
      self.postMessage({ type: 'ready', animated: true });
      running = true;
      loop();
    } catch (err) {
      self.postMessage({ type: 'error', message: String(err) });
    }
  } else if (m.type === 'visible') {
    running = m.visible && animated && engine !== null;
    if (running && !looping) loop();
  } else if (m.type === 'debug') {
    debug = m.on;
    if (engine) report();
  } else if (m.type === 'weight' && engine) {
    engine.set_weight(m.index, m.weight);
  }
};
