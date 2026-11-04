#!/usr/bin/env python3
"""Release spores: turn spores/*.md into the blog.

Write a spore as spores/2026-11-20-my-title.md:

    ---
    title: How a brainless cell routes this website
    tag: engineering          # engineering | research | life (or anything)
    summary: One line shown in lists.
    ---
    Your text in Markdown...

Files whose name starts with "_" (like _template.md) are drafts and are skipped.
The date comes from the file name; the slug is the rest of the name.

Outputs (all inside web/, commit them):
    web/spores/<slug>/index.html   one page per spore
    web/spores/index.json          list the site reads (panel + slime food)
    web/spores/index.html          copy of web/index.html so /spores/ opens the panel

No dependencies: Python 3.8+ only.
"""
import datetime as dt
import html
import json
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "spores"
WEB = ROOT / "web"

AUTHOR = "Sofia Lima"

WORDS_PER_MIN = 220


# ---------------------------------------------------------------- markdown ---

def inline(text):
    """Escape, then apply inline Markdown: code, images, links, bold, italic."""
    parts = re.split(r"(`[^`]+`)", text)
    out = []
    for part in parts:
        if part.startswith("`") and part.endswith("`") and len(part) > 1:
            out.append(f"<code>{html.escape(part[1:-1])}</code>")
            continue
        s = html.escape(part, quote=False)
        s = re.sub(r"!\[([^\]]*)\]\(([^)\s]+)\)", r'<img src="\2" alt="\1" loading="lazy">', s)
        s = re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", r'<a href="\2">\1</a>', s)
        s = re.sub(r"\*\*(.+?)\*\*", r"<strong>\1</strong>", s)
        s = re.sub(r"(?<![\w*])\*(?!\s)(.+?)(?<!\s)\*(?![\w*])", r"<em>\1</em>", s)
        s = re.sub(r"(?<![\w_])_(?!\s)(.+?)(?<!\s)_(?![\w_])", r"<em>\1</em>", s)
        out.append(s)
    return "".join(out)


def markdown(src):
    """A small, predictable Markdown: headings, paragraphs, lists, quotes,
    fenced code, images, links, emphasis, inline code, horizontal rules."""
    lines = src.replace("\r\n", "\n").split("\n")
    out, para, i = [], [], 0

    def flush():
        if para:
            out.append(f"<p>{inline(' '.join(para))}</p>")
            para.clear()

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        if stripped.startswith("```"):
            flush()
            lang = stripped[3:].strip()
            code = []
            i += 1
            while i < len(lines) and not lines[i].strip().startswith("```"):
                code.append(lines[i])
                i += 1
            cls = f' class="lang-{html.escape(lang)}"' if lang else ""
            out.append(f"<pre><code{cls}>{html.escape(chr(10).join(code))}</code></pre>")
        elif not stripped:
            flush()
        elif m := re.match(r"(#{1,4})\s+(.*)", stripped):
            flush()
            level = min(len(m.group(1)) + 1, 5)  # the post title is the <h1>
            out.append(f"<h{level}>{inline(m.group(2))}</h{level}>")
        elif re.match(r"^(-{3,}|\*{3,})$", stripped):
            flush()
            out.append("<hr>")
        elif stripped.startswith(">"):
            flush()
            quote = []
            while i < len(lines) and lines[i].strip().startswith(">"):
                quote.append(lines[i].strip()[1:].strip())
                i += 1
            out.append(f"<blockquote><p>{inline(' '.join(quote))}</p></blockquote>")
            continue
        elif re.match(r"^([-*]|\d+\.)\s+", stripped):
            flush()
            ordered = bool(re.match(r"^\d+\.", stripped))
            items = []
            while i < len(lines) and re.match(r"^\s*([-*]|\d+\.)\s+", lines[i]):
                items.append(re.sub(r"^\s*([-*]|\d+\.)\s+", "", lines[i]))
                i += 1
            tag = "ol" if ordered else "ul"
            out.append(f"<{tag}>" + "".join(f"<li>{inline(t)}</li>" for t in items) + f"</{tag}>")
            continue
        else:
            para.append(stripped)
        i += 1
    flush()
    return "\n".join(out)


# ------------------------------------------------------------------ spores ---

def read_spore(path):
    m = re.match(r"(\d{4}-\d{2}-\d{2})-(.+)\.md$", path.name)
    if not m:
        sys.exit(f"spores: {path.name} should be named YYYY-MM-DD-slug.md")
    date, slug = m.group(1), m.group(2)
    text = path.read_text(encoding="utf-8")
    meta, body = {}, text
    if text.startswith("---"):
        _, front, body = text.split("---", 2)
        for line in front.strip().splitlines():
            if ":" in line:
                key, value = line.split(":", 1)
                key = key.strip().lower()
                # "# comments" are allowed after short fields like tag, not inside titles.
                meta[key] = value.strip() if key in ("title", "summary") else re.sub(r"\s+#.*$", "", value).strip()
    if "title" not in meta:
        sys.exit(f"spores: {path.name} needs a 'title:' line in its header")
    words = len(re.findall(r"\w+", body))
    return {
        "slug": slug,
        "date": date,
        "title": meta["title"],
        "tag": meta.get("tag", "notes").lower(),
        "summary": meta.get("summary", ""),
        "minutes": max(1, round(words / WORDS_PER_MIN)),
        "html": markdown(body.strip()),
    }


def pretty(date):
    d = dt.date.fromisoformat(date)
    return f"{d:%b} {d.day}, {d.year}".lower()


HEAD = """<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <meta name="description" content="{description}">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;700&family=Syne:wght@800&family=Newsreader:ital,opsz@0,6..72;1,6..72&display=swap" rel="stylesheet">
  <link rel="stylesheet" href="/reading.css">
</head>
"""


def post_page(s, number, prev, nxt):
    nav = []
    if prev:
        nav.append(f'<a href="/spores/{prev["slug"]}/">{html.escape(prev["title"])}</a>')
    if nxt:
        nav.append(f'<a class="next" href="/spores/{nxt["slug"]}/">{html.escape(nxt["title"])}</a>')
    return HEAD.format(title=f'{html.escape(s["title"])} · spores', description=html.escape(s["summary"]), author=AUTHOR) + f"""<body class="reading">
  <header class="bar">
    <a href="/"><span class="dim">sofia lima /</span></a> <a href="/spores/">spores</a>
  </header>
  <article>
    <p class="meta"><b>{number:03d}</b> · {pretty(s["date"])} · {html.escape(s["tag"])} · {s["minutes"]} min</p>
    <h1>{html.escape(s["title"])}</h1>
    {f'<p class="lede">{html.escape(s["summary"])}</p>' if s["summary"] else ''}
    <div class="body">
{s["html"]}
    </div>
  </article>
  <footer class="post-foot">
    <p>Released by {AUTHOR}.</p>
    <nav aria-label="More spores">{''.join(nav)}</nav>
  </footer>
</body>
</html>
"""


def main():
    SRC.mkdir(exist_ok=True)
    spores = sorted(
        (read_spore(p) for p in SRC.glob("*.md") if not p.name.startswith("_")),
        key=lambda s: (s["date"], s["slug"]),
        reverse=True,
    )

    out = WEB / "spores"
    if out.exists():
        # Remove pages of spores that no longer exist (only folders this script made).
        for d in out.iterdir():
            if d.is_dir() and (d / ".spore").exists() and d.name not in {s["slug"] for s in spores}:
                shutil.rmtree(d)
    out.mkdir(exist_ok=True)

    n = len(spores)
    for k, s in enumerate(spores):
        page = out / s["slug"]
        page.mkdir(exist_ok=True)
        (page / ".spore").write_text("generated by tools/spores.py\n")
        older = spores[k + 1] if k + 1 < n else None
        newer = spores[k - 1] if k > 0 else None
        (page / "index.html").write_text(post_page(s, n - k, older, newer), encoding="utf-8")

    index = [{key: s[key] for key in ("slug", "date", "title", "tag", "summary", "minutes")} for s in spores]
    (out / "index.json").write_text(json.dumps(index, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    shutil.copyfile(WEB / "index.html", out / "index.html")  # /spores/ opens the site with the panel

    print(f"spores: {n} released · web/spores/index.json")


if __name__ == "__main__":
    main()
