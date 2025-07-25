# Work Plan: Restore Stop Triggers to Test Definitions

## Overview
Move stop_triggers configuration from test run configurations back to test definitions. This change recognizes that stop triggers are intrinsic test properties that define when a test should complete, rather than runtime configuration options. Unlike loggers (which are about how to observe/record test execution), stop triggers define the test's completion criteria and should be part of the test definition.

## Tasks

### 1. Update Test Definition Models
- **Description**: Add stop_trigger fields back to TestQueryDefinition and TestReactionDefinition
- **Actions**: 
  - Add `stop_trigger: Option<StopTriggerDefinition>` field to `TestQueryDefinition` in `test-data-store/src/test_repo_storage/models.rs`
  - Add `stop_triggers: Option<Vec<StopTriggerDefinition>>` field to `TestReactionDefinition` in the same file
  - Ensure proper serde attributes for optional fields
- **Success Criteria**: Test definitions can store stop trigger configurations
- **Dependencies**: None

### 2. Remove Stop Triggers from Test Run Configurations
- **Description**: Remove stop_trigger fields from runtime configurations
- **Actions**: 
  - Remove `stop_trigger` field from `TestRunQueryConfig` in `test-run-host/src/queries/mod.rs`
  - Remove `stop_triggers` field from `TestRunReactionConfig` in `test-run-host/src/reactions/mod.rs`
  - Keep the fields in TestRunQueryOverrides and TestRunReactionOverrides for runtime overrides
- **Success Criteria**: Runtime configurations no longer have stop trigger fields
- **Dependencies**: Task 1

### 3. Update TestRunHost Logic
- **Description**: Modify how TestRunHost retrieves stop triggers from test definitions
- **Actions**: 
  - Update `add_test_query` method in `test-run-host/src/lib.rs` to get stop_trigger from test definition
  - Update `add_test_reaction` method to get stop_triggers from test definition
  - Ensure overrides are still properly applied
- **Success Criteria**: TestRunHost correctly uses stop triggers from definitions
- **Dependencies**: Tasks 1, 2

### 4. Update Test Creation Logic
- **Description**: Modify how stop triggers are passed to observers
- **Actions**: 
  - Update `TestRunQueryDefinition` to get stop_trigger from test definition
  - Update `TestRunReactionDefinition` to get stop_triggers from test definition
  - Update `QueryResultObserverSettings::new` to handle stop_trigger from definition
  - Ensure proper handling of overrides
- **Success Criteria**: Observers receive stop triggers from the correct source
- **Dependencies**: Task 3

### 5. Update Tests
- **Description**: Update all tests to put stop triggers in test definitions
- **Actions**: 
  - Update unit tests in test-run-host that use stop triggers
  - Update integration tests in test-service
  - Move stop trigger configurations from TestRunQueryConfig/TestRunReactionConfig to test definitions
- **Success Criteria**: All tests pass with stop triggers in definitions
- **Dependencies**: Tasks 3, 4

### 6. Update Examples and Documentation
- **Description**: Update example configurations and documentation
- **Actions**: 
  - Update example JSON files to show stop triggers in test definitions
  - Update any example configs that incorrectly show stop triggers in runtime configs
  - Update CLAUDE.md to clarify that only loggers are runtime configuration
  - Add note about stop triggers being part of test definitions
- **Success Criteria**: Documentation and examples accurately reflect the configuration approach
- **Dependencies**: All previous tasks

## Execution Order
1. Task 1 - Update Test Definition Models
2. Task 2 - Remove Stop Triggers from Test Run Configurations
3. Task 3 - Update TestRunHost Logic  
4. Task 4 - Update Test Creation Logic
5. Task 5 - Update Tests
6. Task 6 - Update Examples and Documentation

## Notes
- This partially reverts the previous change, keeping loggers as runtime configuration but moving stop triggers back to test definitions
- Stop triggers define test completion criteria and are not just observability concerns
- The override mechanism should still allow runtime modification of stop triggers if needed
- This change better aligns with the semantic difference between loggers (how to observe) and stop triggers (when to stop)
- Need to ensure backward compatibility or provide clear migration guidance