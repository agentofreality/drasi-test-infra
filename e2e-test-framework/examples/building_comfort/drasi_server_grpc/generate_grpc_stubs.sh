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
RESET="\033[0m"

echo -e "${GREEN}Generating Python gRPC stubs from proto files...${RESET}"

# Check if grpcio-tools is installed
if ! python3 -c "import grpc_tools.protoc" 2>/dev/null; then
    echo -e "${RED}Error: grpcio-tools not installed${RESET}"
    echo "Please install with: pip install grpcio grpcio-tools"
    exit 1
fi

# Navigate to the proto directory
PROTO_DIR="../../../test-run-host/proto"
OUTPUT_DIR="."

# Generate source_dispatcher stubs
echo "Generating source_dispatcher stubs..."
python3 -m grpc_tools.protoc \
    -I"$PROTO_DIR" \
    --python_out="$OUTPUT_DIR" \
    --grpc_python_out="$OUTPUT_DIR" \
    "$PROTO_DIR/source_dispatcher.proto"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ source_dispatcher stubs generated${RESET}"
else
    echo -e "${RED}✗ Failed to generate source_dispatcher stubs${RESET}"
    exit 1
fi

# Generate reaction_handler stubs
echo "Generating reaction_handler stubs..."
python3 -m grpc_tools.protoc \
    -I"$PROTO_DIR" \
    --python_out="$OUTPUT_DIR" \
    --grpc_python_out="$OUTPUT_DIR" \
    "$PROTO_DIR/reaction_handler.proto"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ reaction_handler stubs generated${RESET}"
else
    echo -e "${RED}✗ Failed to generate reaction_handler stubs${RESET}"
    exit 1
fi

echo -e "${GREEN}\nAll gRPC stubs generated successfully!${RESET}"
echo "Generated files:"
ls -la *_pb2*.py 2>/dev/null || echo "No stub files found"