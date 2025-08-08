#!/usr/bin/env python3
"""
Test script to verify gRPC connection and send a test event.
Use this to verify your gRPC server is working correctly.
"""

import sys
import grpc
import time
from datetime import datetime

# Try to import the generated files
try:
    import source_dispatcher_pb2
    import source_dispatcher_pb2_grpc
except ImportError:
    print("Error: Proto files not found!")
    print("Please run: ./generate_grpc_stubs.sh")
    sys.exit(1)


def test_single_event():
    """Send a single test event"""
    print("Testing DispatchSingle RPC...")
    
    # Create channel and stub
    channel = grpc.insecure_channel('localhost:50051')
    stub = source_dispatcher_pb2_grpc.SourceDispatcherStub(channel)
    
    # Create a test event
    event = source_dispatcher_pb2.SourceChangeEvent()
    event.sequence_number = 1
    event.timestamp = int(time.time() * 1e9)
    event.op = "c"  # create
    event.id = "test_item_1"
    event.labels.extend(["test", "grpc"])
    event.properties["name"] = "Test Item"
    event.properties["temperature"] = "72.5"
    event.properties["co2"] = "450"
    event.properties["humidity"] = "45"
    
    try:
        # Send the event
        response = stub.DispatchSingle(event)
        print(f"✓ Success: {response.success}")
        print(f"  Message: {response.message}")
        print(f"  Events processed: {response.events_processed}")
        return True
    except grpc.RpcError as e:
        print(f"✗ RPC Error: {e.code()}")
        print(f"  Details: {e.details()}")
        return False


def test_batch_events():
    """Send a batch of test events"""
    print("\nTesting DispatchBatch RPC...")
    
    # Create channel and stub
    channel = grpc.insecure_channel('localhost:50051')
    stub = source_dispatcher_pb2_grpc.SourceDispatcherStub(channel)
    
    # Create a batch of events
    batch = source_dispatcher_pb2.SourceChangeEventBatch()
    batch.source_id = "test_source"
    batch.metadata["test_run"] = "manual_test"
    
    for i in range(3):
        event = source_dispatcher_pb2.SourceChangeEvent()
        event.sequence_number = i + 10
        event.timestamp = int(time.time() * 1e9) + i * 1000000000
        event.op = ["c", "u", "d"][i % 3]  # create, update, delete
        event.id = f"test_item_{i + 10}"
        event.labels.extend(["test", "batch"])
        event.properties["index"] = str(i)
        batch.events.append(event)
    
    try:
        # Send the batch
        response = stub.DispatchBatch(batch)
        print(f"✓ Success: {response.success}")
        print(f"  Message: {response.message}")
        print(f"  Events processed: {response.events_processed}")
        return True
    except grpc.RpcError as e:
        print(f"✗ RPC Error: {e.code()}")
        print(f"  Details: {e.details()}")
        return False


def main():
    print(f"{'='*60}")
    print("gRPC Connection Test")
    print(f"{'='*60}")
    print(f"Testing connection to localhost:50051")
    print(f"Time: {datetime.now().isoformat()}\n")
    
    # Test connection
    channel = grpc.insecure_channel('localhost:50051')
    try:
        # Try to check if channel is ready
        grpc.channel_ready_future(channel).result(timeout=5)
        print("✓ Connection established\n")
    except grpc.FutureTimeoutError:
        print("✗ Failed to connect to gRPC server on localhost:50051")
        print("  Make sure the server is running: python3 grpc_event_receiver_proper.py")
        sys.exit(1)
    
    # Run tests
    success = True
    
    if not test_single_event():
        success = False
    
    if not test_batch_events():
        success = False
    
    # Summary
    print(f"\n{'='*60}")
    if success:
        print("✓ All tests passed!")
    else:
        print("✗ Some tests failed")
        print("  Check that the server implements both DispatchSingle and DispatchBatch")
    print(f"{'='*60}")
    
    channel.close()
    return 0 if success else 1


if __name__ == '__main__':
    sys.exit(main())