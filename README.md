# physarum.site

A personal homepage whose menu is routed, live, by a simulated slime mold
(*Physarum polycephalum*). The organism is written in Rust, compiled to
WebAssembly, and runs on the visitor's CPU in a background thread. No GPU,
no server, no framework.

```
web/            the static site (deploy this folder)
  index.html    page markup: real links, and the content of every panel  <- edit me
  pages.js      opening a node: lens, slide, URLs, keyboard, phone sheet
  main.js       boots the worker, places links, draws "show the math"
  engine.worker.js   runs the Rust engine off the main thread (OffscreenCanvas)
  config.js     pages, positions, seed, palette, agent budget   <- edit me
  feeds.js      turns real activity (GitHub, notes) into food
  pkg/          the compiled engine (prebuilt, 94 KB of wasm)
  files/        résumé PDF, .tex source and page previews (made by resume/build.sh)
resume/         your résumé in LaTeX + build.sh
crate/          the Rust source
  src/solver.rs the brain: Tero's adaptive network, solved with conjugate gradient
  src/world.rs  the body: Jones agents steered by the brain's tubes + renderer
  src/lib.rs    the wasm-bindgen API the browser calls
build.sh        rebuilds web/pkg from the Rust source
```

## Run it

The engine is already compiled, so you only need Python:

```bash
python3 serve.py          # http://localhost:8080
```

`serve.py` is a tiny static server that also sends page URLs (`/work`,
`/resume`, ...) to `index.html`, so deep links work locally. Opening
`index.html` from disk will not work: browsers only load WebAssembly and
module workers over http.

## Pages

Click a node and the organism slides aside, a lens isolates the node, and the
page slides in. Every page has its own URL (`/work`, `/side-projects`,
`/spores`, `/resume`, `/hi`), the back button works, Esc or a click on the
organism closes it, focus stays inside the panel while it is open, and on
phones the panel is a bottom sheet. With "reduce motion" on, everything just
fades. The content of each page is plain HTML in `web/index.html`
(`<section data-page="...">`); positions and labels are in `web/config.js`.

## Résumé

Edit `resume/resume.tex`, then:

```bash
resume/build.sh           # needs latexmk + poppler (pdftoppm)
```

It compiles the PDF and copies it, the `.tex` source and PNG page previews to
`web/files/`, which is what the résumé panel shows and offers for download.
If the site lives on GitHub, `.github/workflows/resume.yml` does this for you
on every push that touches `resume/`, so the download is always current.

## Change the Rust and rebuild

```bash
# once: https://rustup.rs
./build.sh            # adds the wasm target, installs wasm-pack, builds into web/pkg
cd crate && cargo test --release                 # solver tests
cargo run --release --example render             # native preview frames + ms/step
```

## Deploy

`web/` is a plain static site; no build step is needed on the host because
`web/pkg` and `web/files` are committed. Page URLs need a fallback to
`index.html`, and the configs for that are already included:

- **Netlify**: drag in `web/` (uses `web/_redirects`).
- **Vercel**: set the root directory to `web/` (uses `web/vercel.json`).
- **GitHub Pages**: publish `web/` from a custom domain or user site
  (`you.github.io`); `web/404.html` is a copy of `index.html` that makes
  `/work` etc. work. `./build.sh` keeps it in sync.

## How it works

**Brain.** About 200 random points connected by tubes, with your pages as five
of the points. Every frame one page becomes the food source (chosen in
proportion to its food weight) and the others become sinks. The solver finds
the pressure at every point (Kirchhoff's law, conjugate gradient on the graph
Laplacian with one sink grounded), reads the flux through each tube, then lets
busy tubes thicken and idle ones wither:

```
Σ_j (D_ij / L_ij)(p_i − p_j) = b_i
Q_ij = (D_ij / L_ij)(p_i − p_j)
dD_ij/dt = |Q|^1.8 / (1 + |Q|^1.8) − D_ij
```

**Body.** Thousands of agents follow the Jones model: sense ahead, left and
right, turn toward the strongest signal, step, deposit. The signal is their own
trail plus a chemoattractant painted along the brain's live tubes, so the
veins you see are the solver's decisions.

**Food = real life.** `feeds.js` can raise or lower each page's appetite from
real activity (pushes on GitHub in the last 14 days, how recent your newest
note is). The network reroutes when your life changes. Set `GITHUB_USER` and
`NOTES_FEED` in `config.js` to switch these on.

**Good citizen.** Runs in a Web Worker; pauses when the tab is hidden; sheds
agents automatically if a frame takes longer than ~12 ms; draws a single
still frame for visitors with "reduce motion" turned on; same seed means every
visitor sees the same organism. Links and text are real HTML, so the site
works for screen readers and search engines even if the animation never loads.

Press **show the math** to see the tubes, live solver stats and food weights.

## References

- Tero, Kobayashi, Nakagaki (2007). A mathematical model for adaptive transport
  network in path finding by true slime mold. *J. Theor. Biol.* 244.
- Tero et al. (2010). Rules for biologically inspired adaptive network design.
  *Science* 327.
- Bonifaci, Mehlhorn, Varma (2012). Physarum can compute shortest paths.
  *J. Theor. Biol.* 309.
- Jones (2010). Characteristics of pattern formation and evolution in
  approximations of Physarum transport networks. *Artificial Life* 16.

## Placeholders to fill

Everything in `[brackets]` in `web/index.html` (roles, projects, spores, the
say-hi message, your email and LinkedIn) and in `resume/resume.tex`. After
editing `index.html`, copy it to `web/404.html` (or run `./build.sh`).
