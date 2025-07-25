# Work Plan: Refactor Logger Configuration from Test Definitions to Test Run Configurations

## Overview
Move query and reaction logger configurations from test definitions (which are stored in the test repository) to test run configurations. This change recognizes that logger choices are runtime decisions rather than intrinsic properties of a test, allowing different logging strategies when running the same test.

## Tasks

### 1. Update Test Definition Models
- **Description**: Remove logger configurations from TestQueryDefinition and TestReactionDefinition in the test repository models
- **Actions**: 
  - Remove `output_handler` and `stop_trigger` fields from `TestQueryDefinition` in `test-data-store/src/test_repo_storage/models.rs`
  - Remove `output_loggers` and `stop_triggers` fields from `TestReactionDefinition` in the same file
  - Update any serialization/deserialization logic that depends on these fields
  - Remove the associated type imports (OutputLoggerDefinition, etc.) if no longer needed
- **Success Criteria**: Test definitions no longer contain logger configurations
- **Dependencies**: None

### 2. Update Test Run Query Configuration
- **Description**: Ensure query logger configuration is properly handled in TestRunQueryConfig
- **Actions**: 
  - Verify `TestRunQueryConfig` in `test-run-host/src/queries/mod.rs` already has `loggers` field (it does)
  - Add `stop_trigger` field to `TestRunQueryConfig` if not present
  - Ensure the configuration properly accepts `ResultStreamLoggerConfig` and `StopTriggerDefinition`
- **Success Criteria**: TestRunQueryConfig can specify all logger and stop trigger configurations
- **Dependencies**: Task 1

### 3. Update Test Run Reaction Configuration  
- **Description**: Move reaction logger configuration to TestRunReactionConfig
- **Actions**: 
  - Change `output_loggers` field type in `TestRunReactionConfig` from `Vec<OutputLoggerDefinition>` to use the reaction-specific logger config type
  - Ensure consistency with how queries handle logger configuration
  - Keep `stop_triggers` field as is (already in the right place)
- **Success Criteria**: TestRunReactionConfig properly defines logger configurations
- **Dependencies**: Task 1

### 4. Update TestRunHost Logic
- **Description**: Modify how TestRunHost creates queries and reactions to use run-time configurations
- **Actions**: 
  - Update `add_test_query` method in `test-run-host/src/lib.rs` to use loggers from TestRunQueryConfig instead of test definition
  - Update `add_test_reaction` method to use loggers from TestRunReactionConfig instead of test definition
  - Remove any code that reads logger configurations from test definitions
  - Ensure backward compatibility or provide migration path
- **Success Criteria**: TestRunHost correctly uses runtime configurations for loggers
- **Dependencies**: Tasks 2, 3

### 5. Update Test Creation Logic
- **Description**: Modify TestRunQueryDefinition and TestRunReactionDefinition creation
- **Actions**: 
  - Update `TestRunQueryDefinition::new` to accept loggers from config rather than test definition
  - Update `TestRunReactionDefinition::new` to accept loggers from config rather than test definition
  - Ensure proper merging of configurations with any overrides
- **Success Criteria**: Test run definitions are created with runtime logger configurations
- **Dependencies**: Task 4

### 6. Update Tests and Examples
- **Description**: Update all tests and examples to use the new configuration approach
- **Actions**: 
  - Update unit tests in test-run-host that create TestRunQueryConfig and TestRunReactionConfig
  - Update integration tests in test-service
  - Update any example configurations to show logger configuration in the right place
  - Remove logger configurations from test JSON files
- **Success Criteria**: All tests pass with new configuration structure
- **Dependencies**: Tasks 4, 5

### 7. Update API and Documentation
- **Description**: Ensure REST API and documentation reflect the configuration changes
- **Actions**: 
  - Update OpenAPI schemas if needed
  - Update any API handlers that create test run configurations
  - Update CLAUDE.md or other documentation to reflect where loggers should be configured
  - Add migration notes for users
- **Success Criteria**: API and docs accurately reflect new configuration approach
- **Dependencies**: All previous tasks

## Execution Order
1. Task 1 - Update Test Definition Models
2. Task 2 & 3 (in parallel) - Update Test Run Configurations
3. Task 4 - Update TestRunHost Logic
4. Task 5 - Update Test Creation Logic
5. Task 6 - Update Tests and Examples
6. Task 7 - Update API and Documentation

## Notes
- This is a breaking change that affects how tests are configured
- Consider providing a migration tool or clear migration instructions
- The change improves flexibility by allowing different logging strategies for the same test
- May need to handle backward compatibility for existing test definitions that have logger configurations
- Consider whether default loggers should be provided if none are specified in the run configuration