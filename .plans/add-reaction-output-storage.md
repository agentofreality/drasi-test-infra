# Work Plan: Add Reaction Output Storage to TestDataStore

## Overview
Extend the TestDataStore API and implementation to support writing and reading reaction output data, following the same pattern used for sources and queries. Reaction invocations will be written to a "reactions" subfolder within test_runs, utilizing the existing unified logging infrastructure.

## Tasks

### 1. Extend TestDataStore API
- **Description**: Add methods to TestDataStore trait for managing reaction storage
- **Actions**: 
  - Add `get_test_run_reaction_storage()` method to TestDataStore trait
  - Create `TestRunReactionId` struct following the pattern of TestRunSourceId/TestRunQueryId
  - Create `TestRunReactionStorage` struct to manage reaction output paths
  - Add reaction-specific path construction to the storage layer
- **Success Criteria**: TestDataStore provides consistent API for reaction storage alongside sources and queries
- **Dependencies**: None

### 2. Implement Reaction Storage Layer
- **Description**: Create the storage implementation for reactions
- **Actions**: 
  - Implement TestRunReactionStorage with paths for:
    - Base reaction path: `test_runs/{run_id}/reactions/{reaction_id}/`
    - Invocation log path: `test_runs/{run_id}/reactions/{reaction_id}/output_log/`
  - Add support for reaction summaries (similar to source/query summaries)
  - Ensure proper directory creation and error handling
- **Success Criteria**: Reaction data can be written to and read from the test_data_cache
- **Dependencies**: Task 1

### 3. Add Reaction Logger Configuration
- **Description**: Extend reaction definitions to support logger configuration
- **Actions**: 
  - Add `loggers` field to TestReactionDefinition (Vec<ResultStreamLoggerDefinition>)
  - Update JSON deserialization to parse logger configurations
  - Set default loggers if none specified (e.g., JSONL file logger)
  - Update example configurations to show logger usage
- **Success Criteria**: Reactions can be configured with multiple output loggers
- **Dependencies**: Task 1

### 4. Integrate Loggers into ReactionObserver
- **Description**: Modify ReactionObserver to use the unified logging infrastructure
- **Actions**: 
  - Add logger collection to ReactionObserver struct
  - Initialize loggers during reaction setup using TestRunReactionStorage paths
  - Convert ReactionInvocation to HandlerRecord format
  - Call logger.log() for each invocation in handle_reaction_invocation()
  - Implement logger cleanup on reaction stop
- **Success Criteria**: Reaction invocations are logged to configured outputs
- **Dependencies**: Tasks 2, 3

### 5. Update Test Service API
- **Description**: Expose reaction storage through the REST API
- **Actions**: 
  - Add endpoints to retrieve reaction output files
  - Add reaction storage info to test run status responses
  - Update OpenAPI documentation
  - Add support for downloading reaction logs
- **Success Criteria**: Users can access reaction output via the API
- **Dependencies**: Tasks 1, 2

### 6. Add Tests and Examples
- **Description**: Ensure the implementation is properly tested and documented
- **Actions**: 
  - Add unit tests for TestRunReactionStorage
  - Add integration tests for reaction logging
  - Update building_comfort example to demonstrate reaction output
  - Create documentation showing how to access reaction data
- **Success Criteria**: Feature is well-tested and examples demonstrate usage
- **Dependencies**: Tasks 1-5

## Execution Order
1. Task 1 - Extend TestDataStore API
2. Task 2 - Implement Reaction Storage Layer (depends on Task 1)
3. Task 3 - Add Reaction Logger Configuration (can run parallel with Task 2)
4. Task 4 - Integrate Loggers into ReactionObserver (depends on Tasks 2, 3)
5. Task 5 - Update Test Service API (depends on Tasks 1, 2)
6. Task 6 - Add Tests and Examples (depends on all previous tasks)

## Notes
- Follow existing patterns from source and query storage implementations
- Reuse the unified HandlerRecord and ResultStreamLogger infrastructure
- Maintain backwards compatibility with existing test configurations
- Consider performance implications for high-frequency reaction invocations
- The JSONL file logger should be the default for reactions, matching query behavior
- Ensure proper error handling for storage backend failures
- Document the new API methods and configuration options