import { SEED, PAGES, SIM_CELLS, PALETTE, agentBudget } from './config.js';
import { readFeeds } from './feeds.js';
import { initPages, refreshKicker } from './pages.js';

const slime = document.getElementById('slime');
const math = document.getElementById('math');
const nav = document.getElementById('pages');
const mathPanel = document.getElementById('mathPanel');
const toggle = document.getElementById('toggle');
const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches;

const weights = PAGES.map((p) => p.weight);
const appetite = PAGES.map(() => 1); // temporary multipliers: hover, open page
const food = (i) => weights[i] * appetite[i];
let worker = null;
let simW = 0;
let simH = 0;
let debug = false;
let lastStats = null;

// Where each page sits on this screen (portrait screens get their own layout).
const spot = (p) => (innerHeight > innerWidth ? [p.px ?? p.x, p.py ?? p.y] : [p.x, p.y]);

// Real links, in the DOM, positioned where the organism will find them.
const links = PAGES.map((page) => {
  const a = document.createElement('a');
  a.href = `/${page.id}`;
  a.textContent = page.label;
  nav.append(a);
  return a;
});
function placeLinks() {
  PAGES.forEach((p, i) => {
    const [x, y] = spot(p);
    links[i].style.left = `${x * 100}%`;
    links[i].style.top = `${y * 100}%`;
  });
}

function start() {
  if (worker) worker.terminate();
  // Keep the cell count constant so phones and ultrawides cost the same.
  const aspect = innerWidth / innerHeight;
  simW = Math.round(Math.sqrt(SIM_CELLS * aspect));
  simH = Math.round(simW / aspect);
  placeLinks();

  // A canvas can only be transferred once, so every restart gets a fresh one.
  const fresh = slime.cloneNode();
  canvas.replaceWith(fresh);
  canvas = fresh;
  canvas.width = simW;
  canvas.height = simH;

  worker = new Worker(new URL('./engine.worker.js', import.meta.url), { type: 'module' });
  const terminals = PAGES.flatMap((p, i) => { const [x, y] = spot(p); return [x * simW, y * simH, food(i)]; });
  const msg = { type: 'init', width: simW, height: simH, agents: agentBudget(), terminals, seed: SEED, palette: PALETTE, reducedMotion };
  const transfer = [];
  if ('transferControlToOffscreen' in canvas) {
    msg.canvas = canvas.transferControlToOffscreen();
    transfer.push(msg.canvas);
  }
  worker.onmessage = onWorkerMessage;
  worker.onerror = (e) => console.warn('physarum worker failed; showing the static page', e.message);
  worker.postMessage(msg, transfer);
  worker.postMessage({ type: 'debug', on: debug });
}

let canvas = slime;
let fallbackCtx = null;

function onWorkerMessage({ data: m }) {
  if (m.type === 'frame') {
    // Browsers without OffscreenCanvas: the worker sends pixels, we paint them.
    fallbackCtx ??= canvas.getContext('2d');
    fallbackCtx.putImageData(new ImageData(m.pixels, m.width, m.height), 0, 0);
  } else if (m.type === 'stats') {
    lastStats = m;
    document.getElementById('agents').textContent = Math.round(m.stats[8]).toLocaleString('en-US');
    if (debug) drawMath(m);
  } else if (m.type === 'error') {
    console.warn('physarum:', m.message);
  }
}

// ---- "show the math" ----

function drawMath({ stats, tubes }) {
  const [tick, nodes, edges, alive, length, residual, cgSteps, source] = stats;
  document.getElementById('iter').textContent = `iter ${tick}`;
  document.getElementById('solverStats').innerHTML =
    `graph ${nodes} nodes · ${edges} tubes · ${alive} alive<br>` +
    `CG residual ${residual.toExponential(1)} in ${cgSteps} steps<br>` +
    `network length ${Math.round(length)} px · food source: ${PAGES[source].label}`;
  const max = Math.max(...weights);
  document.getElementById('food').innerHTML =
    '<p class="dim">food = live activity</p>' +
    PAGES.map((p, i) => `<div class="feed"><span>${p.label}</span><span class="bar"><span style="width:${(weights[i] / max) * 100}%"></span></span><span>${weights[i].toFixed(2)}</span></div>`).join('');

  const dpr = devicePixelRatio || 1;
  if (math.width !== Math.round(innerWidth * dpr)) {
    math.width = Math.round(innerWidth * dpr);
    math.height = Math.round(innerHeight * dpr);
  }
  const g = math.getContext('2d');
  const sx = math.width / simW;
  const sy = math.height / simH;
  g.clearRect(0, 0, math.width, math.height);
  if (!tubes) return;
  g.lineCap = 'round';
  for (let k = 0; k < tubes.length; k += 5) {
    const d = tubes[k + 4];
    if (d < 0.01) {
      g.strokeStyle = 'rgba(247,235,176,0.07)';
      g.lineWidth = 0.6 * dpr;
    } else {
      g.strokeStyle = `rgba(120,230,255,${Math.min(0.95, 0.25 + d * 1.4)})`;
      g.lineWidth = (0.6 + d * 7) * dpr;
    }
    g.beginPath();
    g.moveTo(tubes[k] * sx, tubes[k + 1] * sy);
    g.lineTo(tubes[k + 2] * sx, tubes[k + 3] * sy);
    g.stroke();
  }
  PAGES.forEach((p, i) => {
    g.strokeStyle = i === source ? '#78e6ff' : 'rgba(247,235,176,0.8)';
    g.lineWidth = 2 * dpr;
    g.beginPath();
    g.arc(spot(p)[0] * math.width, spot(p)[1] * math.height, (30 + weights[i] * 10) * dpr, 0, Math.PI * 2);
    g.stroke();
  });
}

toggle.addEventListener('click', () => {
  debug = !debug;
  mathPanel.hidden = !debug;
  toggle.textContent = debug ? 'hide the math' : 'show the math';
  toggle.setAttribute('aria-pressed', String(debug));
  worker?.postMessage({ type: 'debug', on: debug });
  if (!debug) math.getContext('2d').clearRect(0, 0, math.width, math.height);
  else if (lastStats) drawMath(lastStats);
});

// ---- lifecycle ----

document.addEventListener('visibilitychange', () => {
  worker?.postMessage({ type: 'visible', visible: !document.hidden });
});

let resizeTimer = 0;
let lastAspect = innerWidth / innerHeight;
addEventListener('resize', () => {
  placeLinks();
  clearTimeout(resizeTimer);
  resizeTimer = setTimeout(() => {
    const aspect = innerWidth / innerHeight;
    if (Math.abs(aspect - lastAspect) / lastAspect > 0.15) {
      lastAspect = aspect;
      start();
    }
  }, 400);
});

// "who? +" toggles a short hello under the sentence.
const who = document.getElementById('who');
const whoCard = document.getElementById('whoCard');
function setWho(open) {
  whoCard.hidden = !open;
  who.setAttribute('aria-expanded', String(open));
  who.lastElementChild.textContent = open ? '−' : '+';
}
who.addEventListener('click', (e) => {
  e.stopPropagation();
  setWho(whoCard.hidden);
});
document.addEventListener('click', (e) => {
  if (!whoCard.hidden && !e.target.closest('#whoCard, #who')) setWho(false);
});
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && !whoCard.hidden) setWho(false);
});
addEventListener('popstate', () => setWho(false));
nav.addEventListener('click', () => setWho(false));

placeLinks();
initPages({
  links,
  spot,
  weightOf: (i) => weights[i],
  setFood(i, mult) {
    appetite[i] = mult;
    worker?.postMessage({ type: 'weight', index: i, weight: food(i) });
  },
});

if (typeof WebAssembly === 'object' && 'Worker' in window) {
  start();
  readFeeds(PAGES, (i, w) => {
    weights[i] = w;
    worker?.postMessage({ type: 'weight', index: i, weight: food(i) });
    refreshKicker();
  });
}
