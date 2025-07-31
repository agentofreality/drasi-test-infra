# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the Drasi E2E Test Framework - a Rust-based testing infrastructure for validating Drasi's reactive graph intelligence platform. The framework simulates data sources, dispatches change events, and monitors query results.

## Key Commands

### Building
```bash
# Build all Docker images
make

# Build with specific options
make DOCKERX_OPTS="--platform linux/amd64,linux/arm64"

# Push to local Kind cluster
make kind-load

# Deploy to Drasi as SourceProvider
make drasi-apply
```

### Development
```bash
# Run the test service locally
cargo run -p test-service

# Run all tests
cargo test

# Run tests for a specific package
cargo test -p test-service

# Run a specific test
cargo test test_name

# Lint check (format + clippy)
make lint-check

# Auto-fix lint issues
cargo fmt
cargo clippy --fix
```

### Running Examples
```bash
# Local population example
./examples/population/run_local

# Drasi population example (requires Kind cluster)
./examples/population/run_kind_drasi
```

## Architecture

The project is a Rust workspace with these main components:

- **test-service**: REST API for managing test runs (port 8080, `/docs` for OpenAPI)
- **test-run-host**: Core test execution engine that orchestrates sources and queries
- **data-collector**: Records test data from external systems
- **proxy**: Routes test traffic between components
- **reactivator**: Reactivates test scenarios for replay
- **test-data-store**: Storage layer supporting Local, Azure Blob, and GitHub backends
- **infrastructure**: Dapr-based messaging abstractions

## Key Concepts

### Test Repository
- Contains bootstrap data and change scripts
- Supports multiple storage backends (Local filesystem, Azure Blob, GitHub)
- JSON-based configuration files

### Test Sources
- Simulate data sources by replaying recorded changes
- Support multiple timing modes: recorded, rebased, live
- Can generate change events with configurable spacing

### Test Queries
- Monitor query results through Redis streams
- Built-in profiling and performance metrics
- Support for various output formats

### Change Dispatchers
- Console: Logs to stdout
- Dapr: Publishes via Dapr pubsub
- Redis: Publishes to Redis streams
- File: Writes to local files

## API Endpoints

Main REST API endpoints (test-service):
- `/api/repos` - Manage test repositories
- `/api/sources` - Configure test sources
- `/api/queries` - Define test queries
- `/api/runs` - Execute and monitor test runs
- `/docs` - Interactive API documentation

## Configuration

Test configurations use JSON format with these key sections:
- `repo`: Repository location and credentials
- `sources`: Data source definitions
- `queries`: Query definitions with profiling options
- `run`: Execution parameters (timing, dispatch, etc.)

## Integration with Drasi

The framework deploys as a Drasi SourceProvider:
1. Build Docker images with `make`
2. Load to Kind cluster with `make kind-load`
3. Deploy provider with `make drasi-apply`
4. Create sources using the E2ETestService provider type

## Important Notes

- Always run `make lint-check` before committing
- The service includes OpenTelemetry for distributed tracing
- Test data can include bootstrap files and change scripts
- Redis is required for query result streaming
- Dapr sidecar is optional but recommended for distributed scenarios

## Configuration Changes (2025-07-25)

**Breaking Change**: Logger configurations have been moved from test definitions to runtime configurations:
- Query loggers should now be specified in `TestRunQueryConfig` instead of `TestQueryDefinition`
- Reaction output loggers should now be specified in `TestRunReactionConfig` instead of `TestReactionDefinition`
- This allows different logging strategies when running the same test multiple times
- Test definitions in repositories should only contain the core test structure, not runtime concerns like logging

**Important**: Stop triggers remain in test definitions:
- Query stop triggers are specified in `TestQueryDefinition.stop_trigger`
- Reaction stop triggers are specified in `TestReactionDefinition.stop_triggers`
- Stop triggers define test completion criteria and are intrinsic to the test itself
- Runtime overrides for stop triggers are available via `TestRunQueryOverrides` and `TestRunReactionOverrides`

## Drasi Server Full Configuration (2025-07-28)

**New Feature**: Drasi Servers can now be fully configured with Sources, Queries, and Reactions:
- Add `sources`, `queries`, and `reactions` arrays to `DrasiServerConfig`
- TestSources can send data to configured sources via `DrasiServerChannel` dispatcher
- TestReactions can receive data from configured reactions via `DrasiServerChannel` handler
- The framework validates that TestSource/TestReaction IDs match configured component names
- See `examples/building_comfort/drasi_server_internal` for a complete example

## DrasiServerCore Integration (2025-07-29)

**Architecture Note**: The test infrastructure uses `DrasiServerCore` instead of `DrasiServer`:
- DrasiServerCore contains only the Source, Query, and Reaction functionality needed for testing
- The Test Service provides its own REST API that wraps DrasiServerCore's programmatic API
- The `api_endpoint` field will always return `None` as DrasiServerCore doesn't expose a Web API
- Port configuration is preserved in test definitions for compatibility but is not used internally
- All component management (sources, queries, reactions) is done through DrasiServerCore's managers
- Starting queries and reactions through the API returns an error as these operations are managed automatically by DrasiServerCore

## Logging Configuration (2025-07-31)

**Suppressing drasi_core Logs**: The drasi_core library uses the `tracing` crate with `#[tracing::instrument]` attributes that generate INFO level logs. To suppress these logs while keeping other logs visible:

```bash
# Set drasi_core modules to error level
RUST_LOG='info,drasi_core::query::continuous_query=error,drasi_core::path_solver=error' cargo run ...
```

**Important Notes**:
- Use `error` level instead of `off` for drasi_core modules (due to tracing/log interop)
- The test-service uses `env_logger` which bridges tracing events to log events
- Apply this pattern to both regular and debug test scripts