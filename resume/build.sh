#!/usr/bin/env bash
# Compile resume.tex and publish the PDF + source to the site's download folder.
set -euo pipefail
cd "$(dirname "$0")"
latexmk -pdf -interaction=nonstopmode -quiet resume.tex
mkdir -p ../web/files
cp resume.pdf ../web/files/sofia-lima-resume.pdf
cp resume.tex ../web/files/resume.tex
# Page previews for the site (shown instantly, on every device; the PDF is the download).
rm -f ../web/files/resume-page-*.png
pdftoppm -png -r 160 resume.pdf ../web/files/resume-page
pages=$(pdfinfo resume.pdf 2>/dev/null | awk '/^Pages:/ {print $2}' || true)
sha=$(git rev-parse --short HEAD 2>/dev/null || echo local)
kb=$(( $(wc -c < resume.pdf) / 1024 ))
cat > ../web/files/build.json <<JSON
{"built": "$(date -u +%Y-%m-%d)", "sha": "$sha", "pages": ${pages:-1}, "kb": $kb, "engine": "pdfTeX via latexmk"}
JSON
latexmk -c -quiet resume.tex
echo "résumé published to web/files/"
