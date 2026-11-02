#!/usr/bin/env python3
"""Local dev server: serves web/ and sends page URLs (/work, /resume, ...) to index.html."""
import http.server, os, sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "web")

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *a, **k):
        super().__init__(*a, directory=ROOT, **k)

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def send_head(self):
        path = self.translate_path(self.path)
        if not os.path.exists(path) and "." not in os.path.basename(self.path.split("?")[0]):
            self.path = "/index.html"
        return super().send_head()

port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
print(f"physarum.site on http://localhost:{port}")
http.server.ThreadingHTTPServer(("", port), Handler).serve_forever()
