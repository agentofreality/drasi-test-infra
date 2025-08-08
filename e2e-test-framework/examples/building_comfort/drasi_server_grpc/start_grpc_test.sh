#!/bin/bash

# Copyright 2025 The Drasi Authors.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

GREEN="\033[32m"
RED="\033[31m"
YELLOW="\033[33m"
RESET="\033[0m"

echo -e "${GREEN}Building Comfort gRPC Test Launcher${RESET}"
echo "======================================"

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}Error: Python 3 is not installed${RESET}"
    echo "Please install Python 3 and try again"
    exit 1
fi

# Check if grpcio is installed
if ! python3 -c "import grpc" 2>/dev/null; then
    echo -e "${YELLOW}Warning: grpcio not installed${RESET}"
    echo "Installing required Python packages..."
    pip install grpcio grpcio-tools
fi

# Generate gRPC stubs if needed
if [ ! -f "source_dispatcher_pb2.py" ] || [ ! -f "reaction_handler_pb2.py" ]; then
    echo -e "${YELLOW}gRPC stubs not found. Generating...${RESET}"
    if [ -f "generate_grpc_stubs.sh" ]; then
        chmod +x generate_grpc_stubs.sh
        ./generate_grpc_stubs.sh
    else
        echo -e "${RED}Error: generate_grpc_stubs.sh not found${RESET}"
        exit 1
    fi
fi

# Check if gRPC receiver is already running
if lsof -i :50051 > /dev/null 2>&1; then
    echo -e "${YELLOW}Port 50051 already in use. gRPC receiver may already be running.${RESET}"
    echo "If not, please stop the process using port 50051 and try again."
else
    # Start the gRPC event receiver in the background
    echo -e "${GREEN}Starting gRPC Event Receiver on port 50051...${RESET}"
    python3 grpc_event_receiver.py &
    RECEIVER_PID=$!
    echo "gRPC Receiver PID: $RECEIVER_PID"
    
    # Give it a moment to start
    sleep 2
    
    # Check if it started successfully
    if ! ps -p $RECEIVER_PID > /dev/null; then
        echo -e "${RED}Failed to start gRPC receiver${RESET}"
        exit 1
    fi
fi

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${RESET}"
    if [ ! -z "$RECEIVER_PID" ]; then
        echo "Stopping gRPC receiver (PID: $RECEIVER_PID)"
        kill $RECEIVER_PID 2>/dev/null
    fi
    echo -e "${GREEN}Cleanup complete${RESET}"
}

# Set up trap to cleanup on script exit
trap cleanup EXIT INT TERM

# Now start the test service
echo -e "\n${GREEN}Starting E2E Test Service...${RESET}"
echo "========================================"
echo -e "${YELLOW}Press Ctrl+C to stop both services${RESET}\n"

# Run the test service (this will block)
RUST_LOG='info,drasi_core::query::continuous_query=error,drasi_core::path_solver=error' \
    cargo run --release --manifest-path ./test-service/Cargo.toml -- \
    --config examples/building_comfort/drasi_server_grpc/config.json

echo -e "\n${GREEN}Test completed${RESET}"