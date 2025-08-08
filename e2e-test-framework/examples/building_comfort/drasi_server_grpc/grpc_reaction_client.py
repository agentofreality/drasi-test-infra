#!/usr/bin/env python3
"""
gRPC client to send reaction invocations to the gRPC reaction handler.
Connects to port 50052 and sends test invocations.

To install dependencies:
    pip install grpcio grpcio-tools

To generate the Python gRPC code:
    python -m grpc_tools.protoc -I../../test-run-host/proto --python_out=. --grpc_python_out=. ../../test-run-host/proto/reaction_handler.proto
"""

import grpc
import json
import sys
from datetime import datetime
import time

# You'll need to generate these from the proto files
# For now, we'll create a simple stub implementation
try:
    import reaction_handler_pb2
    import reaction_handler_pb2_grpc
except ImportError:
    print("Warning: gRPC stubs not generated. Run the protoc command shown in the docstring.")
    print("Creating minimal stub implementation...")
    
    # Minimal stub implementation for demonstration
    class ReactionInvocation:
        def __init__(self):
            self.id = f"inv_{int(time.time())}"
            self.timestamp = int(time.time() * 1e9)
            self.query_id = "building-comfort"
            self.metadata = {"x-query-sequence": "1"}
            self.payload = type('Payload', (), {
                'sequence': 1,
                'added': [],
                'updated': [],
                'deleted': []
            })()
    
    class Stub:
        def __init__(self, channel):
            self.channel = channel
        
        def HandleInvocation(self, request):
            print(f"Would send invocation: {request.id}")
            return type('Response', (), {'success': True, 'message': 'OK', 'invocation_id': request.id})()
    
    reaction_handler_pb2 = type('Module', (), {
        'ReactionInvocation': ReactionInvocation,
        'ReactionPayload': type('ReactionPayload', (), {}),
        'ResultRecord': type('ResultRecord', (), {}),
        'Value': type('Value', (), {})
    })()
    
    reaction_handler_pb2_grpc = type('Module', (), {
        'ReactionHandlerStub': Stub
    })()


def create_test_invocation(sequence_num=1):
    """Create a test reaction invocation"""
    
    # Create some test result records
    added_records = []
    for i in range(3):
        record = reaction_handler_pb2.ResultRecord()
        record.fields["id"] = reaction_handler_pb2.Value(string_value=f"room_{i}")
        record.fields["temperature"] = reaction_handler_pb2.Value(double_value=72.5 + i)
        record.fields["co2"] = reaction_handler_pb2.Value(double_value=450.0 + i * 10)
        record.fields["humidity"] = reaction_handler_pb2.Value(double_value=45.0 + i)
        record.fields["comfort_level"] = reaction_handler_pb2.Value(string_value="comfortable")
        added_records.append(record)
    
    # Create the payload
    payload = reaction_handler_pb2.ReactionPayload()
    payload.sequence = sequence_num
    payload.added.extend(added_records)
    
    # Create the invocation
    invocation = reaction_handler_pb2.ReactionInvocation()
    invocation.id = f"inv_{int(time.time() * 1000)}_{sequence_num}"
    invocation.timestamp = int(time.time() * 1e9)
    invocation.query_id = "building-comfort"
    invocation.metadata["x-query-sequence"] = str(sequence_num)
    invocation.metadata["content-type"] = "application/grpc"
    invocation.payload.CopyFrom(payload)
    
    return invocation


def send_invocations(count=5, delay=1.0):
    """Send multiple test invocations to the reaction handler"""
    
    # Create gRPC channel
    channel = grpc.insecure_channel('localhost:50052')
    stub = reaction_handler_pb2_grpc.ReactionHandlerStub(channel)
    
    print(f"Connecting to gRPC reaction handler at localhost:50052...")
    print(f"Will send {count} invocations with {delay}s delay between each\n")
    
    for i in range(count):
        try:
            # Create invocation
            invocation = create_test_invocation(i + 1)
            
            print(f"[{datetime.now().isoformat()}] Sending invocation {i + 1}...")
            print(f"  ID: {invocation.id}")
            print(f"  Query ID: {invocation.query_id}")
            print(f"  Sequence: {invocation.payload.sequence}")
            print(f"  Records: {len(invocation.payload.added)} added, "
                  f"{len(invocation.payload.updated)} updated, "
                  f"{len(invocation.payload.deleted)} deleted")
            
            # Send the invocation
            response = stub.HandleInvocation(invocation)
            
            print(f"  Response: success={response.success}, "
                  f"message='{response.message}', "
                  f"invocation_id='{response.invocation_id}'")
            
            if i < count - 1:
                time.sleep(delay)
                
        except grpc.RpcError as e:
            print(f"  Error: {e.code()} - {e.details()}")
        except Exception as e:
            print(f"  Error: {e}")
    
    print(f"\nFinished sending {count} invocations")
    channel.close()


def main():
    """Main entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Send test reaction invocations via gRPC')
    parser.add_argument('--count', type=int, default=5,
                        help='Number of invocations to send (default: 5)')
    parser.add_argument('--delay', type=float, default=1.0,
                        help='Delay in seconds between invocations (default: 1.0)')
    
    args = parser.parse_args()
    
    send_invocations(args.count, args.delay)


if __name__ == '__main__':
    main()