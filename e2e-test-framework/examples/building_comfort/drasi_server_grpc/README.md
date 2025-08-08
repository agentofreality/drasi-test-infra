# Building Comfort Test with gRPC

This example demonstrates using gRPC for communication between the E2E Test Framework and a Drasi Server for the building comfort scenario.

## Overview

This test simulates a building management system that monitors comfort levels across rooms based on temperature, CO2, and humidity sensors. Unlike the HTTP version, this example uses gRPC for:

- **Source Change Dispatching**: Sends sensor data updates via gRPC protocol
- **Reaction Handling**: Receives comfort level calculations via gRPC invocations

## Key Differences from HTTP Version

| Feature | HTTP Version | gRPC Version |
|---------|--------------|--------------|
| Source Dispatcher | HTTP POST to port 9000 | gRPC to port 50051 |
| Reaction Handler | HTTP webhook on port 9001 | gRPC server on port 50052 |
| Data Format | JSON | Protocol Buffers |
| Protocol | HTTP/1.1 | HTTP/2 (gRPC) |
| Correlation | HTTP headers | gRPC metadata |

## Architecture

```
┌─────────────────────┐         gRPC          ┌──────────────────┐
│   Test Framework    │──────────────────────>│  gRPC Receiver   │
│  (Source Dispatcher)│      Port 50051       │  (Your Service)  │
└─────────────────────┘                       └──────────────────┘
                                                       │
                                                       ▼
┌─────────────────────┐                       ┌──────────────────┐
│   Test Framework    │                       │   Drasi Server   │
│ (Reaction Handler)  │<──────────────────────│   (Processes     │
│   gRPC Server       │      Reactions        │    Query Data)   │
└─────────────────────┘                       └──────────────────┘
```

## Prerequisites

### For Running the Test
- Rust toolchain installed
- Cargo and the test framework built

### For Python gRPC Tools (Optional)
```bash
pip install grpcio grpcio-tools
```

## Running the Test

### Quick Start (Recommended)

Use the all-in-one startup script that handles everything:
```bash
chmod +x start_grpc_test.sh
./start_grpc_test.sh
```

This script will:
- Check dependencies
- Generate gRPC stubs if needed  
- Start the gRPC receiver
- Start the test service
- Clean up when you press Ctrl+C

### Manual Setup (Alternative)

If you prefer to run components separately:

#### ⚠️ IMPORTANT: Start the gRPC Receiver First!

The gRPC event receiver MUST be running before starting the test service, otherwise events will fail to dispatch.

### 1. Start the gRPC Event Receiver (REQUIRED)

First, generate the Python gRPC stubs (one-time setup):
```bash
chmod +x generate_grpc_stubs.sh
./generate_grpc_stubs.sh
```

Then start the receiver:
```bash
python3 grpc_event_receiver.py
```

This starts a gRPC server on port 50051 that will receive and log all source change events. Keep this running!

### 2. Start the Test Service

In a new terminal (while the receiver is still running):
```bash
chmod +x run_test.sh
./run_test.sh
```

Or for debug mode with detailed gRPC logging:
```bash
chmod +x run_test_debug.sh
./run_test_debug.sh
```

The test service will:
- Start a gRPC reaction handler on port 50052
- Generate building sensor data
- Dispatch events to your gRPC receiver on port 50051
- Wait for reaction invocations on port 50052

### 3. Test the Reaction Handler (Optional)

To manually test the reaction handler, use the provided client:
```bash
python3 grpc_reaction_client.py --count 5 --delay 1
```

This sends 5 test reaction invocations to verify the handler is working.

## Configuration Details

### gRPC Source Dispatcher
```json
{
  "kind": "Grpc",
  "host": "localhost",
  "port": 50051,
  "timeout_seconds": 60,
  "batch_events": false,
  "tls": false
}
```

### gRPC Reaction Handler
```json
{
  "kind": "Grpc",
  "host": "0.0.0.0",
  "port": 50052,
  "correlation_metadata_key": "x-query-sequence"
}
```

## Data Flow

1. **Source Generation**: BuildingHierarchy model generates sensor data
2. **gRPC Dispatch**: Events sent via gRPC to port 50051
3. **Processing**: Your service/Drasi Server processes the data
4. **Reaction Invocation**: Results sent back via gRPC to port 50052
5. **Logging**: Results logged to JSONL files

## Monitoring

### Check Test Status
Use the web API files to monitor test progress:
```bash
# Check source status
curl http://localhost:8080/api/test_runs/test_run_001/sources

# Check reaction status
curl http://localhost:8080/api/test_runs/test_run_001/reactions
```

### View Logs
Reaction outputs are saved in:
```
test_data_cache/test_run_001/reactions/building-comfort/output_logs/
```

## Protocol Buffer Definitions

### Source Dispatcher Service
- `DispatchBatch`: Handles multiple events in one call
- `DispatchSingle`: Handles individual events

### Reaction Handler Service  
- `HandleInvocation`: Processes single reaction invocation
- `StreamInvocations`: (Future) Streaming support

## Troubleshooting

### "Unimplemented" Error
If you see errors like:
```
Failed to dispatch event via gRPC: status: Unimplemented
```

This means the gRPC server is running but doesn't implement the correct RPC methods. 

**Solution**: Use the proper receiver implementation:
```bash
# Stop any existing receiver
pkill -f grpc_event_receiver

# Use the correct implementation
python3 grpc_event_receiver_proper.py
```

### Testing the gRPC Connection
To verify your gRPC server is working correctly:
```bash
# First, generate the stubs if needed
./generate_grpc_stubs.sh

# Then test the connection
python3 test_grpc_connection.py
```

This will send test events and verify both DispatchSingle and DispatchBatch methods work.

### Port Already in Use
```bash
# Check what's using the ports
lsof -i :50051
lsof -i :50052

# Kill the process if needed
kill -9 <PID>
```

### gRPC Connection Issues
- Ensure firewall allows connections on ports 50051 and 50052
- Check that the gRPC receiver is running BEFORE starting the test
- Enable debug logging to see detailed gRPC communication:
  ```bash
  ./run_test_debug.sh
  ```

### Python Import Errors
If the Python scripts can't import gRPC stubs:
```bash
# Install dependencies
pip install grpcio grpcio-tools

# Regenerate the stubs
./generate_grpc_stubs.sh
```

### Debugging Event Flow
1. Start the receiver with the proper implementation:
   ```bash
   python3 grpc_event_receiver_proper.py
   ```

2. In another terminal, watch for events being sent:
   ```bash
   # Run with debug logging
   RUST_LOG=debug ./run_test_debug.sh 2>&1 | grep -i grpc
   ```

3. You should see:
   - "Attempting to connect to gRPC endpoint"
   - "Successfully connected to gRPC endpoint"  
   - "gRPC dispatcher sending individual event" (or batch)
   - "Successfully dispatched N events via gRPC"

## Performance Benefits

Using gRPC provides several advantages:
- **Binary Protocol**: More efficient than JSON serialization
- **HTTP/2**: Multiplexing and better connection management
- **Type Safety**: Protocol Buffers ensure consistent message structure
- **Streaming**: Future support for high-volume streaming

## Customization

To modify the test:
1. Edit `config.json` to change sensor parameters
2. Adjust `change_count` to control test duration
3. Modify `batch_events` to test batch vs individual dispatch
4. Update correlation key for different tracking needs

## Integration with Drasi Server

When integrating with an actual Drasi Server:
1. Configure the server to connect to the gRPC reaction handler on port 50052
2. Implement a gRPC source adapter that receives from port 50051
3. The test framework handles all lifecycle management automatically