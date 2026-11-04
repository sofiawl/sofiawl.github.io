# sofiawl.github.io

My personal site. The menu is not designed: a simulated slime mold
(*Physarum polycephalum*) finds the paths between my pages, live, and the
paths you see are its decisions. The simulation is written in Rust, compiled
to WebAssembly, and runs in a background thread on the visitor's CPU. No GPU,
no server, no framework.

Live at https://sofiawl.github.io

## Layout

```
web/              the static site, this is what gets deployed
  index.html      the content of every page, as plain HTML
  config.js       page positions, seed, palette, GitHub settings
  pages.js        opening a node: lens, slide, URLs, keyboard, phone sheet
  main.js         starts the worker, places the links, draws "show the math"
  feeds.js        turns GitHub activity and spores into food for the organism
  engine.worker.js  runs the Rust engine off the main thread
  pkg/            the compiled engine (committed, about 94 KB of wasm)
  files/          resume PDF, .tex and page previews (made by resume/build.sh)
  spores/         generated blog pages
resume/           my resume in LaTeX, plus a build script
spores/           blog posts, one Markdown file each
tools/spores.py   turns spores/ into pages
crate/            the Rust source
  src/solver.rs   the "brain": Tero's adaptive network
  src/world.rs    the "body": Jones agents and the renderer
  src/lib.rs      the wasm-bindgen API the browser calls
build.sh          rebuilds web/pkg from the Rust source
```

## Run it locally

The engine is already compiled, so you only need Python:

```bash
python3 serve.py
```

Then open http://localhost:8080. `serve.py` also sends `/work`, `/resume` and
the other page URLs to `index.html`, so deep links work. Opening `index.html`
straight from disk will not work, because browsers only load WebAssembly and
module workers over http.

## The pages

Clicking a node opens its page at its own URL: `/work`, `/side-projects`,
`/spores`, `/resume` and `/hi`. The back button works, Esc closes the panel,
and on phones the panel is a bottom sheet. With "reduce motion" turned on,
everything just fades. The text of each page lives in `web/index.html`, inside
`<section data-page="...">`. Positions and labels are in `web/config.js`.

After editing `index.html`, copy it to `web/404.html`. GitHub Pages serves
that file for URLs that do not exist, which is what makes `/work` and friends
open the right panel. `./build.sh` and the spores workflow do the copy too.

## Resume

Edit `resume/resume.tex`, then run:

```bash
resume/build.sh
```

It needs `latexmk` and `pdftoppm` (from poppler). The script builds the PDF
and copies it, the `.tex` and PNG previews into `web/files/`, which is what
the resume page shows. The `resume` workflow does the same on every push that
touches `resume/`.

## Spores (the blog)

To write a post, copy `spores/_template.md` to
`spores/YYYY-MM-DD-your-title.md`, write in Markdown, then run:

```bash
python3 tools/spores.py
```

That creates a reading page for each post under `web/spores/` and a
`web/spores/index.json` that the spores page reads. Files that start with `_`
are drafts and are skipped. The `spores` workflow runs the script for me
whenever something in `spores/` changes.

## Changing the Rust

You need [rustup](https://rustup.rs) once. Then:

```bash
./build.sh                              # builds the wasm into web/pkg
cd crate && cargo test --release        # solver tests
cargo run --release --example render    # preview frames and ms per step
```

## Deploy

The site is deployed by `.github/workflows/pages.yml`, which publishes the
`web/` folder to GitHub Pages. It runs on pushes to `main` that touch `web/`,
and after the resume and spores workflows finish. There is no build step,
because `web/pkg` and `web/files` are committed. In the repo settings, Pages
has to use "GitHub Actions" as its source.

The repo is named `sofiawl.github.io` on purpose, so the site is served from
the root and absolute paths like `/styles.css` work. A project site under a
sub-path would break them.

`web/_redirects` (Netlify) and `web/vercel.json` (Vercel) are there in case I
ever move the site, since both need a fallback to `index.html`.

## How it works

**The brain.** About 200 random points joined by tubes. Five of them are my
pages. Each frame, one page is the food source (picked in proportion to its
food weight) and the others are sinks. The solver finds the pressure at every
point (Kirchhoff's law, conjugate gradient on the graph Laplacian), reads how
much flows through each tube, and lets busy tubes grow thicker while idle ones
fade:

```
sum_j (D_ij / L_ij)(p_i - p_j) = b_i
Q_ij = (D_ij / L_ij)(p_i - p_j)
dD_ij/dt = |Q|^1.8 / (1 + |Q|^1.8) - D_ij
```

**The body.** Thousands of agents follow the Jones model: sense ahead, left
and right, turn toward the strongest signal, step, leave a trail. The signal
is their own trail plus a chemical painted along the brain's tubes, so the
veins on screen are the solver's decisions.

**Food from real life.** `feeds.js` changes how hungry each page is based on
what I have been doing. Work grows with my recent pushes to the Kinebot
organization, side projects with pushes to HER and running, and spores with
how recent my newest post is. Which repos count is set in `web/config.js`.
GitHub's public events API only shows public repos, so private ones count as
nothing.

**Being nice to the browser.** It runs in a Web Worker, pauses when the tab is
hidden, drops agents if a frame takes longer than about 12 ms, and draws one
still frame for visitors with "reduce motion" on. The same seed gives every
visitor the same organism. Links and text are real HTML, so the site still
works for screen readers and search engines if the animation never loads.

The "show the math" button shows the tubes, the live solver stats and the food
weights.

## References

- Tero, Kobayashi, Nakagaki (2007). A mathematical model for adaptive
  transport network in path finding by true slime mold. *J. Theor. Biol.* 244.
- Tero et al. (2010). Rules for biologically inspired adaptive network design.
  *Science* 327.
- Bonifaci, Mehlhorn, Varma (2012). Physarum can compute shortest paths.
  *J. Theor. Biol.* 309.
- Jones (2010). Characteristics of pattern formation and evolution in
  approximations of Physarum transport networks. *Artificial Life* 16.
