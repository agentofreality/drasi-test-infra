# Work Plan: Implement Output Loggers and Stop Triggers for Reactions

## Overview
Implement output_loggers and stop_triggers for reactions in the test-run-host, following the existing patterns established for queries. This will enable reactions to log their outputs and define conditions for stopping execution.

## Tasks

### 1. Create Output Logger Infrastructure for Reactions
- **Description**: Create the base trait and module structure for reaction output loggers
- **Actions**: 
  - Create `reactions/output_loggers/mod.rs` with OutputLogger trait and error types
  - Define OutputLoggerConfig enum with Console and JsonlFile variants
  - Implement factory functions for creating output loggers
  - Update HandlerRecord type to support reaction-specific data if needed
- **Success Criteria**: Base infrastructure compiles with proper trait definitions
- **Dependencies**: None

### 2. Implement Console Output Logger
- **Description**: Implement console logger for reactions following the query console logger pattern
- **Actions**: 
  - Create `reactions/output_loggers/console_logger.rs`
  - Implement ConsoleOutputLogger struct with configuration
  - Implement OutputLogger trait methods (log_handler_record, end_test_run)
  - Format reaction outputs appropriately for console display
- **Success Criteria**: Console logger outputs reaction events to stdout
- **Dependencies**: Task 1

### 3. Implement JSONL File Output Logger
- **Description**: Implement JSONL file logger for reactions following the query JSONL logger pattern
- **Actions**: 
  - Create `reactions/output_loggers/jsonl_file_logger.rs`
  - Implement JsonlFileOutputLogger with file writing capabilities
  - Integrate with TestRunReactionStorage for output file management
  - Handle buffering and flushing for performance
- **Success Criteria**: JSONL logger writes reaction events to files in proper format
- **Dependencies**: Task 1

### 4. Create Stop Trigger Infrastructure for Reactions
- **Description**: Create the base trait and module structure for reaction stop triggers
- **Actions**: 
  - Create `reactions/stop_triggers/mod.rs` with StopTrigger trait
  - Define error types and factory function
  - Update ReactionHandlerStatus if needed for trigger evaluation
  - Create ReactionObserverMetrics type for trigger conditions
- **Success Criteria**: Stop trigger infrastructure compiles with trait definitions
- **Dependencies**: None

### 5. Implement Record Count Stop Trigger
- **Description**: Implement record count stop trigger for reactions
- **Actions**: 
  - Create `reactions/stop_triggers/record_count.rs`
  - Implement RecordCountStopTrigger with configuration
  - Implement is_true method to check record count conditions
  - Handle various count conditions (equals, greater than, etc.)
- **Success Criteria**: Stop trigger correctly evaluates record count conditions
- **Dependencies**: Task 4

### 6. Integrate Output Loggers with Reaction Observer
- **Description**: Wire output loggers into the reaction execution flow
- **Actions**: 
  - Update reaction_observer.rs to accept output logger configurations
  - Call output logger methods during reaction processing
  - Handle multiple output loggers per reaction
  - Ensure proper error handling and cleanup
- **Success Criteria**: Output loggers receive and process all reaction events
- **Dependencies**: Tasks 1, 2, 3

### 7. Integrate Stop Triggers with Reaction Observer
- **Description**: Wire stop triggers into the reaction execution flow
- **Actions**: 
  - Update reaction_observer.rs to accept stop trigger configurations
  - Evaluate stop triggers after each reaction event
  - Implement proper shutdown when triggers fire
  - Handle multiple stop triggers with OR logic
- **Success Criteria**: Reactions stop when trigger conditions are met
- **Dependencies**: Tasks 4, 5

### 8. Update Configuration Models
- **Description**: Update test data store models to include reaction output loggers and stop triggers
- **Actions**: 
  - Add OutputLoggerDefinition enum to test data store models
  - Add stop trigger support to ReactionDefinition
  - Update serialization/deserialization for new fields
  - Ensure backward compatibility with existing configs
- **Success Criteria**: Configuration models support new features
- **Dependencies**: None

### 9. Add Unit Tests
- **Description**: Create comprehensive unit tests for all new components
- **Actions**: 
  - Test console logger output formatting
  - Test JSONL file logger file operations
  - Test record count trigger evaluation logic
  - Test integration points in reaction observer
- **Success Criteria**: All tests pass with good coverage
- **Dependencies**: Tasks 1-7

### 10. Update Examples and Documentation
- **Description**: Update example configurations to demonstrate new features
- **Actions**: 
  - Add output logger configurations to example JSON files
  - Add stop trigger configurations to examples
  - Update code comments with usage examples
  - Document configuration options
- **Success Criteria**: Examples demonstrate proper usage of new features
- **Dependencies**: Tasks 1-8

## Execution Order
1. Task 1 (Output Logger Infrastructure)
2. Task 4 (Stop Trigger Infrastructure) - can run in parallel with Task 1
3. Task 2 (Console Logger) - depends on Task 1
4. Task 3 (JSONL Logger) - depends on Task 1
5. Task 5 (Record Count Trigger) - depends on Task 4
6. Task 8 (Configuration Models) - can start anytime
7. Task 6 (Integrate Output Loggers) - depends on Tasks 1, 2, 3
8. Task 7 (Integrate Stop Triggers) - depends on Tasks 4, 5
9. Task 9 (Unit Tests) - depends on Tasks 1-7
10. Task 10 (Examples/Docs) - depends on Tasks 1-8

## Notes
- Follow the exact patterns from query result_stream_loggers and stop_triggers
- Ensure consistent error handling and async patterns
- Consider performance implications for high-volume reaction processing
- Maintain backward compatibility with existing reaction configurations
- Use the same storage abstraction patterns as queries for output files
- Consider whether HandlerRecord needs reaction-specific fields or can be reused as-is