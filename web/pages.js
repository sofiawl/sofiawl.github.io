// Opening a node: the organism slides aside, a lens isolates the node, the panel slides in.
// URLs: /work, /side-projects, /spores, /resume, /hi (back button and deep links work).
import { PAGES } from './config.js';

const $ = (id) => document.getElementById(id);
const body = document.body;
const stage = $('stage');
const sheet = $('sheet');
const lens = $('lens');
const kicker = $('sheetKicker');
const crumbName = $('crumbName');
const sections = PAGES.map((p) => sheet.querySelector(`section[data-page="${p.id}"]`));
const reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;
const narrow = () => matchMedia('(max-width: 720px)').matches;
const SLIDE_MS = reduce ? 0 : 450;
const PULSE_MS = reduce ? 0 : 280;

let current = -1;
let returnFocus = null;
let closeTimer = 0;
let ctx = null; // { links, spot, setFood, weightOf }

const pathIndex = () => {
  const id = decodeURIComponent(location.pathname).replace(/^\/+|\/+$/g, '');
  return PAGES.findIndex((p) => p.id === id);
};

function layout(i) {
  const [fx, fy] = ctx.spot(PAGES[i]);
  const W = innerWidth;
  const H = innerHeight;
  let dx = 0;
  let dy = 0;
  if (narrow()) {
    // Phones: the panel is a bottom sheet, so lift the node into the strip above it.
    dy = H * 0.11 - fy * H;
  } else {
    // Park the node in the strip left of the panel, about 30% across the screen.
    const sheetW = sheet.offsetWidth || W * 0.58;
    const target = Math.min(W * 0.3, (W - sheetW) * 0.62);
    dx = target - fx * W;
  }
  stage.style.transform = `translate(${dx}px, ${dy}px)`;
  lens.style.setProperty('--lx', `${fx * W + dx}px`);
  lens.style.setProperty('--ly', `${fy * H + dy}px`);
  ctx.links.forEach((a, k) => a.classList.toggle('active', k === i));
  // The label sits outside the stage so the lens never dims it.
  const label = $('lensLabel');
  label.textContent = PAGES[i].label;
  label.style.left = `${fx * W + dx}px`;
  label.style.top = `${fy * H + dy + (fy > 0.7 || narrow() ? -104 : 104)}px`;
}

function fillKicker(i) {
  const p = PAGES[i];
  kicker.textContent = `NODE 0${i + 1} · ${p.label.toUpperCase()} · FOOD ${ctx.weightOf(i).toFixed(2)} · ${p.food.toUpperCase()}`;
}

export function open(i, { push = true, instant = false } = {}) {
  if (i === current) return;
  clearTimeout(closeTimer);
  if (current < 0) returnFocus = document.activeElement;
  if (current >= 0) ctx.setFood(current, 1);
  current = i;
  const p = PAGES[i];

  if (instant) body.classList.add('instant');
  sections.forEach((s, k) => (s.hidden = k !== i));
  crumbName.textContent = p.label;
  fillKicker(i);
  sheet.hidden = false;
  body.classList.add('open');
  layout(i);
  ctx.setFood(i, 3); // the opened node keeps eating, so the network leans toward it
  requestAnimationFrame(() => {
    body.classList.add('shown');
    if (instant) requestAnimationFrame(() => body.classList.remove('instant'));
  });

  onOpen[p.id]?.();
  if (push) history.pushState({ page: p.id }, '', `/${p.id}`);
  document.title = `${p.label} · Sofia Lima`;
  setTimeout(() => sections[i].querySelector('h2')?.focus({ preventScroll: true }), instant ? 0 : 60);
}

export function close({ push = true } = {}) {
  if (current < 0) return;
  ctx.setFood(current, 1);
  ctx.links.forEach((a) => a.classList.remove('active'));
  body.classList.remove('shown');
  stage.style.transform = '';
  current = -1;
  closeTimer = setTimeout(() => {
    sheet.hidden = true;
    body.classList.remove('open');
  }, SLIDE_MS);
  if (push) history.pushState({}, '', '/');
  document.title = 'Sofia Lima';
  returnFocus?.focus?.({ preventScroll: true });
}

// ---- per-page behaviour ----

let resumeLoaded = false;
const onOpen = {
  resume() {
    if (resumeLoaded) return;
    resumeLoaded = true;
    fetch('/files/build.json')
      .then((r) => (r.ok ? r.json() : null))
      .then((b) => {
        if (!b) return;
        // Extra pages beyond the first preview.
        const pane = $('cvPdf');
        for (let n = 2; n <= b.pages; n++) {
          const a = pane.firstElementChild.cloneNode(true);
          a.querySelector('img').src = `/files/resume-page-${n}.png`;
          a.querySelector('img').alt = `Résumé, page ${n}`;
          pane.append(a);
        }
        $('pdfMeta').textContent = `sofia-lima-resume.pdf · ${b.pages} page${b.pages > 1 ? 's' : ''} · ${b.kb} KB`;
        $('buildInfo').innerHTML = `${b.engine} · built ${b.built}<br>sha <b>${b.sha}</b> · <span class="ok">✓ compiled</span>`;
      })
      .catch(() => {});
  },
};

function wireResume() {
  const tabs = [$('tabPdf'), $('tabTex')];
  const panes = [$('cvPdf'), $('cvTex')];
  let texLoaded = false;
  tabs.forEach((tab, k) =>
    tab.addEventListener('click', () => {
      tabs.forEach((t, j) => t.setAttribute('aria-selected', String(j === k)));
      panes.forEach((p, j) => (p.hidden = j !== k));
      if (k === 1 && !texLoaded) {
        texLoaded = true;
        fetch('/files/resume.tex')
          .then((r) => r.text())
          .then((t) => (panes[1].querySelector('code').textContent = t))
          .catch(() => (panes[1].querySelector('code').textContent = 'Could not load resume.tex'));
      }
    }),
  );
  $('copyLink').addEventListener('click', (e) => copy(`${location.origin}/resume`, e.currentTarget, '✓ link copied'));
}

function wireSpores() {
  const chips = [...sheet.querySelectorAll('.filters .chip')];
  const items = [...sheet.querySelectorAll('section[data-page="spores"] [data-tags]')];
  chips.forEach((chip) =>
    chip.addEventListener('click', () => {
      chips.forEach((c) => {
        c.classList.toggle('on', c === chip);
        c.setAttribute('aria-pressed', String(c === chip));
      });
      const tag = chip.dataset.tag;
      items.forEach((el) => (el.hidden = tag !== 'all' && !el.dataset.tags.split(' ').includes(tag)));
    }),
  );
}

function copy(text, button, done) {
  const label = button.textContent;
  navigator.clipboard?.writeText(text).then(() => {
    button.textContent = done;
    setTimeout(() => (button.textContent = label), 1600);
  });
}

// ---- wiring ----

export function initPages(context) {
  ctx = context;

  ctx.links.forEach((a, i) => {
    a.addEventListener('mouseenter', () => current !== i && ctx.setFood(i, 2.5));
    a.addEventListener('mouseleave', () => current !== i && ctx.setFood(i, 1));
    a.addEventListener('click', (e) => {
      if (e.metaKey || e.ctrlKey || e.shiftKey) return; // let people open in a new tab
      e.preventDefault();
      if (current === i) return close();
      ctx.setFood(i, 4);
      a.classList.add('pulse');
      setTimeout(() => {
        a.classList.remove('pulse');
        open(i);
      }, PULSE_MS);
    });
  });

  $('close').addEventListener('click', () => close());
  $('crumb').addEventListener('click', (e) => {
    e.preventDefault();
    close();
  });
  // A click on the organism (not on a link) closes the panel.
  stage.addEventListener('click', (e) => {
    if (current >= 0 && !e.target.closest('a')) close();
  });
  sheet.addEventListener('click', (e) => {
    const go = e.target.closest('[data-go]');
    if (!go) return;
    e.preventDefault();
    open(PAGES.findIndex((p) => p.id === go.dataset.go));
  });

  document.addEventListener('keydown', (e) => {
    if (current < 0) return;
    if (e.key === 'Escape') return close();
    if (e.key !== 'Tab') return;
    // Keep focus inside the panel while it is open.
    const focusables = [...sheet.querySelectorAll('a[href], button, iframe, [tabindex="-1"]')].filter(
      (el) => !el.closest('[hidden]') && el.offsetParent !== null,
    );
    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  });

  addEventListener('popstate', () => {
    const i = pathIndex();
    if (i >= 0) open(i, { push: false });
    else close({ push: false });
  });
  addEventListener('resize', () => current >= 0 && layout(current));

  $('copyEmail').addEventListener('click', (e) => copy($('email').textContent.trim(), e.currentTarget, '✓ copied'));
  wireResume();
  wireSpores();

  // Deep link: arrive straight into a page.
  const i = pathIndex();
  if (i >= 0) open(i, { push: false, instant: true });
}

export const isOpen = () => current >= 0;
