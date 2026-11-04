// Everything you are likely to edit lives here.

// Same seed => same organism for every visitor. Change it to grow a new one.
export const SEED = 'sofia';

// Your pages. x/y are fractions of the screen (px/py on portrait screens);
// weight is the default food.
// `feed` names a function in feeds.js that turns real activity into food.
export const PAGES = [
  { id: 'work',          label: 'work',          x: 0.72, y: 0.19, px: 0.72, py: 0.36, weight: 1.2, feed: 'github', food: 'Kinebot pushes/14d' },
  { id: 'side-projects', label: 'side projects', x: 0.92, y: 0.56, px: 0.78, py: 0.58, weight: 0.9, feed: 'sideRepos', food: 'pushes/14d' },
  { id: 'spores',        label: 'spores',        x: 0.67, y: 0.88, px: 0.70, py: 0.82, weight: 1.0, feed: 'notes',  food: 'post recency' },
  { id: 'resume',        label: 'résumé',        x: 0.40, y: 0.84, px: 0.30, py: 0.74, weight: 0.8, feed: null,     food: 'last pdf build' },
  { id: 'hi',            label: 'say hi',        x: 0.50, y: 0.49, px: 0.30, py: 0.50, weight: 0.6, feed: null,     food: 'inbox' },
];
// Each page opens at /<id>. Its content lives in index.html, in <section data-page="<id>">.

// Optional live food sources (see feeds.js). Leave null to use the defaults above.
export const GITHUB_USER = 'sofiawl';
export const WORK_ORG = 'Kinebot';
export const SIDE_REPOS = ['sofiawl/HER', 'VRI-UFPR/running'];
export const NOTES_FEED = '/spores/index.json'; // written by tools/spores.py

// Simulation size in cells (about 400 x 270 on a laptop). The shape follows the screen.
export const SIM_CELLS = 108000;

// Agents per device class. The worker also lowers this on its own if a frame is slow.
export function agentBudget() {
  const cores = navigator.hardwareConcurrency || 4;
  const mobile = matchMedia('(pointer: coarse)').matches;
  if (mobile) return 8000;
  return cores >= 8 ? 24000 : 14000;
}

// Polycephalum palette: agar, sheet, vein, highlight.
export const PALETTE = [[38, 42, 22], [122, 106, 12], [232, 190, 14], [255, 244, 168]];
