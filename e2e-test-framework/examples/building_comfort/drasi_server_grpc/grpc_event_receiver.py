#!/usr/bin/env python3
"""
gRPC server to receive and log events from the gRPC dispatcher.
Listens on port 50051 and logs all received events.

To install dependencies:
    pip install grpcio grpcio-tools

To generate the Python gRPC code:
    python -m grpc_tools.protoc -I../../test-run-host/proto --python_out=. --grpc_python_out=. ../../test-run-host/proto/source_dispatcher.proto
"""

import grpc
from concurrent import futures
import json
import sys
from datetime import datetime
import logging
import signal
import time

# You'll need to generate these from the proto files
# For now, we'll create a simple stub implementation
try:
    import source_dispatcher_pb2
    import source_dispatcher_pb2_grpc
except ImportError:
    print("Warning: gRPC stubs not generated. Run the protoc command shown in the docstring.")
    print("Creating minimal stub implementation...")
    
    # Minimal stub implementation for demonstration
    class SourceDispatcherServicer:
        def DispatchBatch(self, request, context):
            print(f"\n[{datetime.now().isoformat()}] Received batch dispatch")
            print(f"Source ID: {request.source_id if hasattr(request, 'source_id') else 'unknown'}")
            print(f"Number of events: {len(request.events) if hasattr(request, 'events') else 0}")
            
            # Create response
            response = type('DispatchResponse', (), {
                'success': True,
                'message': 'Batch received',
                'events_processed': len(request.events) if hasattr(request, 'events') else 0
            })()
            return response
        
        def DispatchSingle(self, request, context):
            print(f"\n[{datetime.now().isoformat()}] Received single event dispatch")
            if hasattr(request, 'op'):
                print(f"Operation: {request.op}")
            if hasattr(request, 'id'):
                print(f"ID: {request.id}")
            if hasattr(request, 'timestamp'):
                print(f"Timestamp: {request.timestamp}")
            if hasattr(request, 'properties'):
                print(f"Properties: {dict(request.properties)}")
                
            # Create response
            response = type('DispatchResponse', (), {
                'success': True,
                'message': 'Event received',
                'events_processed': 1
            })()
            return response
    
    source_dispatcher_pb2_grpc = type('Module', (), {
        'SourceDispatcherServicer': SourceDispatcherServicer,
        'add_SourceDispatcherServicer_to_server': lambda servicer, server: print("Added servicer to server")
    })()


class SourceDispatcherServicer(source_dispatcher_pb2_grpc.SourceDispatcherServicer):
    """Implements the SourceDispatcher gRPC service"""
    
    def __init__(self):
        self.event_count = 0
    
    def DispatchBatch(self, request, context):
        """Handle batch of events"""
        self.event_count += len(request.events)
        
        print(f"\n[{datetime.now().isoformat()}] Received DispatchBatch")
        print(f"Source ID: {request.source_id}")
        print(f"Metadata: {dict(request.metadata)}")
        print(f"Number of events: {len(request.events)}")
        
        for i, event in enumerate(request.events):
            print(f"\nEvent {i + 1}:")
            print(f"  Sequence: {event.sequence_number}")
            print(f"  Timestamp: {event.timestamp}")
            print(f"  Operation: {event.op}")
            print(f"  ID: {event.id}")
            print(f"  Labels: {list(event.labels)}")
            print(f"  Properties: {dict(event.properties)}")
            
            if event.HasField('relationship'):
                rel = event.relationship
                print(f"  Relationship:")
                print(f"    Start ID: {rel.start_id}")
                print(f"    End ID: {rel.end_id}")
                print(f"    Start Labels: {list(rel.start_labels)}")
                print(f"    End Labels: {list(rel.end_labels)}")
        
        print(f"\nTotal events received so far: {self.event_count}")
        
        # Create response
        return source_dispatcher_pb2.DispatchResponse(
            success=True,
            message=f"Successfully processed {len(request.events)} events",
            events_processed=len(request.events)
        )
    
    def DispatchSingle(self, request, context):
        """Handle single event"""
        self.event_count += 1
        
        print(f"\n[{datetime.now().isoformat()}] Received DispatchSingle")
        print(f"  Sequence: {request.sequence_number}")
        print(f"  Timestamp: {request.timestamp}")
        print(f"  Operation: {request.op}")
        print(f"  ID: {request.id}")
        print(f"  Labels: {list(request.labels)}")
        print(f"  Properties: {dict(request.properties)}")
        
        if request.HasField('relationship'):
            rel = request.relationship
            print(f"  Relationship:")
            print(f"    Start ID: {rel.start_id}")
            print(f"    End ID: {rel.end_id}")
            print(f"    Start Labels: {list(rel.start_labels)}")
            print(f"    End Labels: {list(rel.end_labels)}")
        
        print(f"\nTotal events received so far: {self.event_count}")
        
        # Create response
        return source_dispatcher_pb2.DispatchResponse(
            success=True,
            message="Successfully processed event",
            events_processed=1
        )


def serve():
    """Start the gRPC server"""
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    source_dispatcher_pb2_grpc.add_SourceDispatcherServicer_to_server(
        SourceDispatcherServicer(), server
    )
    
    port = 50051
    server.add_insecure_port(f'[::]:{port}')
    
    print(f"Starting gRPC event receiver on port {port}...")
    print("Press Ctrl+C to stop\n")
    
    server.start()
    
    # Keep the server running
    try:
        while True:
            time.sleep(86400)  # Sleep for a day
    except KeyboardInterrupt:
        print("\nShutting down...")
        server.stop(0)


if __name__ == '__main__':
    logging.basicConfig()
    serve()