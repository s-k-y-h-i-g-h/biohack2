"""SPA-compatible HTTP server for biohack2 web frontend."""
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import os


class SPAHandler(SimpleHTTPRequestHandler):
    """Serves files from dist/, falling back to index.html for unknown routes."""

    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory="dist", **kwargs)

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def do_GET(self):
        # Serve actual files first
        path = self.path.split("?")[0].lstrip("/")
        full_path = os.path.join(self.directory, path)
        if os.path.isfile(full_path):
            super().do_GET()
            return

        # SPA fallback: serve index.html for all other routes
        index_path = os.path.join(self.directory, "index.html")
        if os.path.isfile(index_path):
            self.path = "/"
            super().do_GET()
        else:
            super().do_GET()


if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    with ThreadingHTTPServer(("0.0.0.0", 8082), SPAHandler) as httpd:
        print("Serving biohack2 at http://localhost:8082")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nShutting down server.")
            httpd.server_close()
