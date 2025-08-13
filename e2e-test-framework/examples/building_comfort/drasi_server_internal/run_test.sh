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

#!/bin/bash

GREEN="\033[32m"
RESET="\033[0m"

echo -e "${GREEN}\nRunning the E2E Test Service with Internal Drasi Server...${RESET}"
# Set drasi_core modules to error level to suppress INFO logs from tracing instrumentation
RUST_LOG='info,drasi_core::query::continuous_query=error,drasi_core::path_solver=error' cargo run --release --manifest-path ./test-service/Cargo.toml -- --config examples/building_comfort/drasi_server_internal/config.json
