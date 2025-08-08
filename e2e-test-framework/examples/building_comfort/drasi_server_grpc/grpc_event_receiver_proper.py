#!/usr/bin/env python3
"""
Proper gRPC server implementation that matches the proto file exactly.
This version correctly implements the SourceDispatcher service.

To generate the required Python files from proto:
    python3 -m grpc_tools.protoc -I../../../test-run-host/proto \
        --python_out=. --grpc_python_out=. \
        ../../../test-run-host/proto/source_dispatcher.proto
"""

import sys
import os
import grpc
from concurrent import futures
import json
from datetime import datetime
import logging
import time

# First, try to generate the proto files if they don't exist
try:
    import source_dispatcher_pb2
    import source_dispatcher_pb2_grpc
except ImportError:
    print("Proto files not found. Attempting to generate them...")
    import subprocess
    
    proto_path = "../../../test-run-host/proto"
    proto_file = "source_dispatcher.proto"
    
    try:
        # Try to install grpcio-tools if not present
        subprocess.run([sys.executable, "-m", "pip", "install", "grpcio-tools"], 
                      capture_output=True, check=False)
        
        # Generate the proto files
        result = subprocess.run([
            sys.executable, "-m", "grpc_tools.protoc",
            f"-I{proto_path}",
            "--python_out=.",
            "--grpc_python_out=.",
            f"{proto_path}/{proto_file}"
        ], capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Failed to generate proto files: {result.stderr}")
            print("\nPlease run the following command manually:")
            print(f"python3 -m grpc_tools.protoc -I{proto_path} --python_out=. --grpc_python_out=. {proto_path}/{proto_file}")
            sys.exit(1)
            
        print("Proto files generated successfully!")
        import source_dispatcher_pb2
        import source_dispatcher_pb2_grpc
        
    except Exception as e:
        print(f"Error generating proto files: {e}")
        print("\nPlease install grpcio-tools and run:")
        print(f"python3 -m grpc_tools.protoc -I{proto_path} --python_out=. --grpc_python_out=. {proto_path}/{proto_file}")
        sys.exit(1)


class SourceDispatcherServicer(source_dispatcher_pb2_grpc.SourceDispatcherServicer):
    """
    Implements the SourceDispatcher gRPC service exactly as defined in the proto file.
    """
    
    def __init__(self):
        self.event_count = 0
        self.batch_count = 0
        
    def DispatchBatch(self, request, context):
        """Handle a batch of source change events"""
        self.batch_count += 1
        batch_size = len(request.events)
        self.event_count += batch_size
        
        print(f"\n{'='*60}")
        print(f"[{datetime.now().isoformat()}] Received DispatchBatch #{self.batch_count}")
        print(f"{'='*60}")
        print(f"Source ID: {request.source_id}")
        print(f"Batch size: {batch_size} events")
        
        if request.metadata:
            print(f"Metadata: {dict(request.metadata)}")
        
        # Process each event in the batch
        for i, event in enumerate(request.events, 1):
            print(f"\n--- Event {i}/{batch_size} ---")
            self._print_event(event)
        
        print(f"\n✓ Total events processed so far: {self.event_count}")
        print(f"✓ Total batches received: {self.batch_count}")
        
        # Return success response
        response = source_dispatcher_pb2.DispatchResponse(
            success=True,
            message=f"Successfully processed batch of {batch_size} events",
            events_processed=batch_size
        )
        return response
    
    def DispatchSingle(self, request, context):
        """Handle a single source change event"""
        self.event_count += 1
        
        print(f"\n{'='*60}")
        print(f"[{datetime.now().isoformat()}] Received DispatchSingle #{self.event_count}")
        print(f"{'='*60}")
        
        self._print_event(request)
        
        print(f"\n✓ Total events processed: {self.event_count}")
        
        # Return success response
        response = source_dispatcher_pb2.DispatchResponse(
            success=True,
            message="Successfully processed event",
            events_processed=1
        )
        return response
    
    def _print_event(self, event):
        """Helper to print event details"""
        print(f"  Sequence Number: {event.sequence_number}")
        print(f"  Timestamp: {event.timestamp} ({datetime.fromtimestamp(event.timestamp / 1e9).isoformat()})")
        print(f"  Operation: {event.op}")
        print(f"  ID: {event.id}")
        
        if event.labels:
            print(f"  Labels: {list(event.labels)}")
        
        if event.properties:
            print(f"  Properties:")
            for key, value in event.properties.items():
                print(f"    {key}: {value}")
        
        # Check if relationship data exists
        if event.HasField('relationship'):
            rel = event.relationship
            print(f"  Relationship:")
            print(f"    Start ID: {rel.start_id}")
            print(f"    End ID: {rel.end_id}")
            if rel.start_labels:
                print(f"    Start Labels: {list(rel.start_labels)}")
            if rel.end_labels:
                print(f"    End Labels: {list(rel.end_labels)}")


def serve():
    """Start the gRPC server"""
    # Configure logging
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s'
    )
    
    # Create server with thread pool
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    
    # Add the service
    servicer = SourceDispatcherServicer()
    source_dispatcher_pb2_grpc.add_SourceDispatcherServicer_to_server(servicer, server)
    
    # Bind to port
    port = 50051
    address = f'[::]:{port}'
    server.add_insecure_port(address)
    
    # Start server
    print(f"\n{'='*60}")
    print(f"gRPC Source Dispatcher Server")
    print(f"{'='*60}")
    print(f"Server starting on port {port}")
    print(f"Listening for source change events...")
    print(f"\nService: SourceDispatcher")
    print(f"Methods:")
    print(f"  - DispatchBatch: Handles batches of events")
    print(f"  - DispatchSingle: Handles individual events")
    print(f"\nPress Ctrl+C to stop")
    print(f"{'='*60}\n")
    
    server.start()
    
    try:
        # Keep server running
        while True:
            time.sleep(86400)  # Sleep for a day
    except KeyboardInterrupt:
        print(f"\n{'='*60}")
        print(f"Shutting down server...")
        print(f"Final statistics:")
        print(f"  Total events processed: {servicer.event_count}")
        print(f"  Total batches received: {servicer.batch_count}")
        print(f"{'='*60}")
        server.stop(grace=5)
        print("Server stopped.")


if __name__ == '__main__':
    serve()