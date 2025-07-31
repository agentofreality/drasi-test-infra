#!/usr/bin/env python3
"""
Simple HTTP server to receive and log events from the HTTP dispatcher.
Listens on port 9000 and logs all received events.
"""

from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import sys
from datetime import datetime

class EventHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        # Read the content length
        content_length = int(self.headers.get('Content-Length', 0))
        
        # Read the request body
        body = self.rfile.read(content_length)
        
        # Log the request
        print(f"\n[{datetime.now().isoformat()}] Received POST to {self.path}")
        print(f"Full URL: http://localhost:9000{self.path}")
        print(f"Headers: {dict(self.headers)}")
        
        try:
            # Parse JSON
            data = json.loads(body.decode('utf-8'))
            
            # Check if it's a batch or single event
            if isinstance(data, list):
                print(f"Received batch of {len(data)} events:")
                for i, event in enumerate(data):
                    print(f"\nEvent {i + 1}:")
                    print(json.dumps(event, indent=2))
            else:
                print("Received single event:")
                print(json.dumps(data, indent=2))
                
        except json.JSONDecodeError as e:
            print(f"Failed to parse JSON: {e}")
            print(f"Raw body: {body.decode('utf-8')}")
        
        # Send success response
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(b'{"status": "ok"}')
    
    def log_message(self, format, *args):
        # Suppress default logging
        pass

def main():
    port = 9000
    server_address = ('', port)
    
    print(f"Starting event receiver on port {port}...")
    print("Press Ctrl+C to stop\n")
    
    httpd = HTTPServer(server_address, EventHandler)
    
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        httpd.shutdown()

if __name__ == '__main__':
    main()