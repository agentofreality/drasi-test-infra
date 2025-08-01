# Work Plan: Fix test_data_cache Not Being Deleted on Ctrl-C

## Overview
When running the building_comfort test with internal Drasi server and pressing Ctrl-C to exit, the test_data_cache directory is not being deleted even though `delete_on_stop` is set to `true` in the configuration. The root cause is that the signal handler does not explicitly perform cleanup, relying only on Rust's Drop trait which may not execute reliably during signal-based shutdown.

## Tasks

### 1. Add Explicit Cleanup to Signal Handler
- **Description**: Implement explicit TestDataStore cleanup in the shutdown_signal function
- **Actions**: 
  - Pass TestDataStore reference to the shutdown_signal function
  - Add explicit cleanup call before server shutdown
  - Ensure cleanup happens for both SIGINT (Ctrl-C) and SIGTERM
  - Handle cleanup errors gracefully
- **Success Criteria**: TestDataStore cleanup is called explicitly when receiving shutdown signals
- **Dependencies**: None

### 2. Make Cleanup Async-Safe
- **Description**: Convert synchronous filesystem operations to async to prevent blocking during shutdown
- **Actions**: 
  - Replace std::fs::remove_dir_all with tokio::fs::remove_dir_all in cleanup
  - Create an async cleanup method in TestDataStore
  - Ensure Drop trait still works as fallback
- **Success Criteria**: Cleanup operations don't block the async runtime during shutdown
- **Dependencies**: Task 1

### 3. Add Cleanup State Tracking
- **Description**: Prevent double cleanup attempts between signal handler and Drop trait
- **Actions**: 
  - Add a `cleaned_up` flag to TestDataStore
  - Set flag when cleanup is performed
  - Check flag in Drop trait to avoid duplicate cleanup
  - Add appropriate logging for cleanup state
- **Success Criteria**: Cleanup only executes once, either in signal handler or Drop
- **Dependencies**: Tasks 1 and 2

### 4. Update Web API Initialization
- **Description**: Modify the web API startup to support explicit cleanup
- **Actions**: 
  - Clone TestDataStore Arc for the signal handler
  - Update run_web_api function signature if needed
  - Ensure TestDataStore reference is available in shutdown context
- **Success Criteria**: Signal handler has access to TestDataStore for cleanup
- **Dependencies**: Task 1

### 5. Add Integration Tests
- **Description**: Create tests to verify cleanup behavior on signal handling
- **Actions**: 
  - Add test that spawns test-service process
  - Send SIGINT/SIGTERM and verify cleanup
  - Test both delete_on_stop true and false scenarios
  - Verify no double cleanup occurs
- **Success Criteria**: Automated tests confirm cleanup works correctly on signals
- **Dependencies**: Tasks 1-4

### 6. Update Documentation
- **Description**: Document the cleanup behavior and configuration
- **Actions**: 
  - Update comments in shutdown_signal function
  - Document cleanup behavior in TestDataStore
  - Add notes about signal handling in CLAUDE.md if needed
- **Success Criteria**: Clear documentation of cleanup behavior and guarantees
- **Dependencies**: Tasks 1-5

## Execution Order
1. Task 1 - Add Explicit Cleanup to Signal Handler
2. Task 2 - Make Cleanup Async-Safe (depends on Task 1)
3. Task 3 - Add Cleanup State Tracking (depends on Tasks 1 and 2)
4. Task 4 - Update Web API Initialization (depends on Task 1)
5. Task 5 - Add Integration Tests (depends on Tasks 1-4)
6. Task 6 - Update Documentation (depends on Tasks 1-5)

## Notes
- The current implementation relies on Drop trait which is not guaranteed during signal-based shutdown
- Must handle both Unix (SIGTERM) and non-Unix platforms appropriately
- Consider graceful shutdown timeout to ensure cleanup completes
- Cleanup errors should be logged but not prevent shutdown
- The fix should maintain backward compatibility with existing configurations