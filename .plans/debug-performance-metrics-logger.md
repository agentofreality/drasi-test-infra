# Work Plan: Debug PerformanceMetricsOutputLogger Not Writing Results

## Overview
Investigate why the PerformanceMetricsOutputLogger is not writing results when running the building_comfort test with internal Drasi Server. The logger appears to be configured correctly in the config.json but may not be getting created or executed properly.

## Tasks

### 1. Verify Logger Configuration and Creation
- **Description**: Check if the logger is being properly configured and created from the config.json
- **Actions**: 
  - Add debug logging to PerformanceMetricsOutputLogger::new() to confirm it's being instantiated
  - Add debug logging to create_output_logger() in mod.rs to trace logger creation
  - Verify the output_loggers configuration is being parsed correctly from config.json
  - Check if the logger is added to the list of active loggers in reaction_observer
- **Success Criteria**: Confirm logger is being created with correct configuration
- **Dependencies**: None

### 2. Trace Logger Lifecycle in Reaction Observer
- **Description**: Follow the logger through its lifecycle in the reaction observer
- **Actions**: 
  - Add logging to reaction_observer.rs where loggers are initialized
  - Add logging to confirm log_handler_record() is being called
  - Add logging to confirm end_test_run() is being called when stop trigger fires
  - Check if the stop trigger count (89647) is being reached
- **Success Criteria**: Understand the complete lifecycle and identify where it breaks
- **Dependencies**: Task 1

### 3. Verify File Path and Permissions
- **Description**: Ensure the output path is valid and writable
- **Actions**: 
  - Log the full output path being used
  - Check if the performance_metrics directory is being created
  - Verify write permissions on the test_data_cache directory
  - Add error handling and logging around directory creation and file write operations
- **Success Criteria**: Confirm file system operations are working correctly
- **Dependencies**: None

### 4. Check TestRunReactionStorage Integration
- **Description**: Verify the storage abstraction is working correctly
- **Actions**: 
  - Confirm TestRunReactionStorage is properly initialized
  - Check if reaction_output_path is set correctly
  - Verify the path structure matches what the logger expects
  - Compare with JsonlFileOutputLogger to ensure similar patterns
- **Success Criteria**: Storage paths are correctly configured
- **Dependencies**: None

### 5. Investigate DrasiServerChannel Handler Integration
- **Description**: Check if the DrasiServerChannel handler affects logger execution
- **Actions**: 
  - Verify if DrasiServerChannel handler properly forwards records to loggers
  - Check if there's any filtering or transformation that might skip logger calls
  - Confirm the handler receives records from the internal Drasi Server
  - Check if stop triggers are properly propagated when using DrasiServerChannel
- **Success Criteria**: Understand how DrasiServerChannel interacts with loggers
- **Dependencies**: Tasks 1, 2

### 6. Add Comprehensive Error Handling
- **Description**: Ensure all errors are logged and not silently swallowed
- **Actions**: 
  - Add error logging to all Result returns in the logger
  - Add panic handlers or error propagation for critical failures
  - Ensure async errors are properly handled
  - Add logging for successful operations to confirm execution
- **Success Criteria**: All errors are visible and logged
- **Dependencies**: None

### 7. Create Minimal Test Case
- **Description**: Create a simpler test configuration to isolate the issue
- **Actions**: 
  - Create a minimal config with just PerformanceMetrics logger
  - Test without DrasiServerChannel handler
  - Test with a lower record count stop trigger
  - Compare behavior with Console or JsonlFile loggers
- **Success Criteria**: Identify if issue is specific to this configuration
- **Dependencies**: All previous tasks

## Execution Order
1. Task 1 - Verify Logger Configuration and Creation
2. Task 3 - Verify File Path and Permissions (can run parallel with Task 1)
3. Task 4 - Check TestRunReactionStorage Integration (can run parallel with Task 1)
4. Task 2 - Trace Logger Lifecycle in Reaction Observer
5. Task 5 - Investigate DrasiServerChannel Handler Integration
6. Task 6 - Add Comprehensive Error Handling
7. Task 7 - Create Minimal Test Case

## Notes
- The logger has log::error! statements added but they may not be visible if RUST_LOG is filtering them
- The stop trigger is set to 89647 records which is quite high - it may not be reached
- The test uses an internal Drasi Server which may have different behavior than external servers
- The DrasiServerChannel handler is relatively new and may have integration issues
- Check if the test completes successfully or times out before reaching the stop trigger
- Consider adding a time-based stop trigger as a fallback for debugging