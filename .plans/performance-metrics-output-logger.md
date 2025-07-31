# Work Plan: Performance Metrics OutputLogger

## Overview
Create a new OutputLogger implementation that tracks timing metrics (in nanoseconds) for received records, counts the total records, and writes a performance summary file when the TestRunReaction StopTrigger fires. This logger will help measure reaction performance and throughput.

## Tasks

### 1. Create PerformanceMetricsOutputLogger Implementation
- **Description**: Create the main logger struct and implementation
- **Actions**: 
  - Create new file `performance_metrics_logger.rs` in the output_loggers directory
  - Define `PerformanceMetricsOutputLoggerConfig` struct for configuration
  - Define `PerformanceMetricsOutputLogger` struct with fields:
    - `start_time_ns: Option<u64>` - tracks first record timestamp
    - `end_time_ns: u64` - tracks when stop trigger fires
    - `record_count: u64` - counts total records
    - `test_run_reaction_id: TestRunReactionId` - for identification
    - `output_storage: TestRunReactionStorage` - for file output
    - `output_path: PathBuf` - where to write the metrics file
  - Implement the new() constructor method
- **Success Criteria**: Struct compiles with all necessary fields
- **Dependencies**: None

### 2. Implement OutputLogger Trait Methods
- **Description**: Implement the required trait methods for OutputLogger
- **Actions**: 
  - Implement `log_handler_record`:
    - If `start_time_ns` is None, set it to `std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64`
    - Increment `record_count`
    - Optionally log debug info about received records
  - Implement `end_test_run`:
    - Set `end_time_ns` to current time in nanoseconds
    - Calculate duration: `end_time_ns - start_time_ns.unwrap_or(end_time_ns)`
    - Calculate records per second: `record_count as f64 / (duration as f64 / 1_000_000_000.0)`
    - Create metrics JSON structure
    - Write metrics to file using output_storage
    - Return OutputLoggerResult with `has_output: true` and output folder path
- **Success Criteria**: All trait methods implemented correctly
- **Dependencies**: Task 1

### 3. Create Metrics Data Structure
- **Description**: Define the structure for the performance metrics output
- **Actions**: 
  - Create a `PerformanceMetrics` struct with Serialize derive:
    - `start_time_ns: u64`
    - `end_time_ns: u64` 
    - `duration_ns: u64`
    - `record_count: u64`
    - `records_per_second: f64`
    - `test_run_reaction_id: String`
    - `timestamp: chrono::DateTime<Utc>` - when metrics were written
  - Implement Display trait for human-readable output
- **Success Criteria**: Metrics structure serializes to clean JSON
- **Dependencies**: None

### 4. Add to OutputLoggerConfig Enum
- **Description**: Integrate the new logger into the existing configuration system
- **Actions**: 
  - Add `PerformanceMetrics(PerformanceMetricsOutputLoggerConfig)` variant to OutputLoggerConfig enum in mod.rs
  - Add corresponding match arm in `create_output_logger` function
  - Add public exports for the new types in mod.rs
- **Success Criteria**: New logger can be configured via JSON config
- **Dependencies**: Tasks 1, 2

### 5. Add File Writing Logic
- **Description**: Implement the file writing using TestRunReactionStorage
- **Actions**: 
  - Use the JsonlFileWriter pattern from existing loggers
  - Create output file path like `performance_metrics_{timestamp}.json`
  - Write the metrics JSON to the file
  - Handle any IO errors appropriately
  - Ensure the output folder exists via storage abstraction
- **Success Criteria**: Metrics file is written to correct location
- **Dependencies**: Tasks 2, 3

### 6. Create Unit Tests
- **Description**: Add comprehensive unit tests for the new logger
- **Actions**: 
  - Add test module in performance_metrics_logger.rs
  - Test logger creation
  - Test first record sets start time
  - Test multiple records increment count
  - Test end_test_run produces correct metrics
  - Test edge cases (no records, single record)
  - Add integration test in output_loggers/tests.rs
- **Success Criteria**: All tests pass with good coverage
- **Dependencies**: Tasks 1-5

### 7. Update Documentation
- **Description**: Document the new logger and its usage
- **Actions**: 
  - Add module-level documentation to performance_metrics_logger.rs
  - Add configuration example to E2E Test Framework documentation
  - Add example output file format
  - Update CLAUDE.md if needed
- **Success Criteria**: Clear documentation for users
- **Dependencies**: All previous tasks

## Execution Order
1. Task 1 - Create PerformanceMetricsOutputLogger Implementation
2. Task 3 - Create Metrics Data Structure (can run parallel with Task 1)
3. Task 2 - Implement OutputLogger Trait Methods 
4. Task 4 - Add to OutputLoggerConfig Enum
5. Task 5 - Add File Writing Logic
6. Task 6 - Create Unit Tests
7. Task 7 - Update Documentation

## Notes
- Use nanosecond precision for timing to capture high-frequency events accurately
- The logger should handle the case where no records are received (start_time_ns remains None)
- Consider using atomic operations if thread safety becomes a concern
- The metrics file should be human-readable JSON format
- Follow the existing pattern from JsonlFileOutputLogger for file operations
- Ensure compatibility with the existing stop trigger mechanism in reaction_observer.rs
- Consider adding additional metrics in the future (p50/p95/p99 latencies, etc.)