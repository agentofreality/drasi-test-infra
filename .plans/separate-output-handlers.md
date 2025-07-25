# Work Plan: Separate Unified Handler into Query and Reaction Components

## Overview
Refactor the unified output handler architecture to provide separate, parallel functionality for queries and reactions. This will move QueryResultHandlers into the queries folder and ReactionHandlers into the reactions folder, eliminating the shared unified_handler abstraction.

## Tasks

### 1. Analyze Current Handler Dependencies
- **Description**: Map out all files that currently use the unified handler components
- **Actions**: 
  - Identify all imports of unified_handler, OutputHandler, and related types
  - Document which components are used by queries vs reactions
  - Identify shared functionality that needs to be duplicated or kept common
  - Map out the impact on QueryResultObserver and ReactionObserver
- **Success Criteria**: Complete dependency map showing all usage points
- **Dependencies**: None

### 2. Create Query-Specific Handler Infrastructure
- **Description**: Build dedicated handler types for query result processing
- **Actions**: 
  - Create `queries/query_output_handler.rs` with QueryOutputHandler trait
  - Define QueryHandlerStatus enum specific to query needs
  - Create QueryHandlerMessage type for query-specific messaging
  - Implement query-specific handler creation logic
  - Move ResultStreamHandler implementations under queries module
- **Success Criteria**: Complete query handler infrastructure in queries folder
- **Dependencies**: Task 1

### 3. Create Reaction-Specific Handler Infrastructure
- **Description**: Build dedicated handler types for reaction processing
- **Actions**: 
  - Create `reactions/reaction_output_handler.rs` with ReactionOutputHandler trait
  - Define ReactionHandlerStatus enum specific to reaction needs
  - Create ReactionHandlerMessage type for reaction-specific messaging
  - Implement reaction-specific handler creation logic
  - Ensure HTTP and future EventGrid handlers are properly typed
- **Success Criteria**: Complete reaction handler infrastructure in reactions folder
- **Dependencies**: Task 1

### 4. Update QueryResultObserver
- **Description**: Refactor QueryResultObserver to use query-specific handlers
- **Actions**: 
  - Update imports to use new query handler types
  - Replace OutputHandler references with QueryOutputHandler
  - Update handler creation to use query-specific factory
  - Adjust message handling for QueryHandlerMessage type
  - Update status tracking to use QueryHandlerStatus
- **Success Criteria**: QueryResultObserver works with new query handlers
- **Dependencies**: Task 2

### 5. Update ReactionObserver
- **Description**: Refactor ReactionObserver to use reaction-specific handlers
- **Actions**: 
  - Update imports to use new reaction handler types
  - Replace OutputHandler references with ReactionOutputHandler
  - Update handler creation to use reaction-specific factory
  - Adjust message handling for ReactionHandlerMessage type
  - Update status tracking to use ReactionHandlerStatus
- **Success Criteria**: ReactionObserver works with new reaction handlers
- **Dependencies**: Task 3

### 6. Migrate Redis Result Stream Handler
- **Description**: Move Redis handler to queries module with proper typing
- **Actions**: 
  - Move redis_result_stream_handler to queries/result_stream_handlers/
  - Update to implement QueryOutputHandler trait
  - Adjust message types to QueryHandlerMessage
  - Update status reporting to use QueryHandlerStatus
  - Ensure bootstrap functionality remains intact
- **Success Criteria**: Redis handler works under queries module
- **Dependencies**: Tasks 2, 4

### 7. Migrate HTTP Reaction Handler
- **Description**: Move HTTP handler to reactions module with proper typing
- **Actions**: 
  - Move http_reaction_handler to reactions/reaction_handlers/
  - Update to implement ReactionOutputHandler trait
  - Adjust message types to ReactionHandlerMessage
  - Update status reporting to use ReactionHandlerStatus
  - Ensure webhook receiving functionality remains intact
- **Success Criteria**: HTTP handler works under reactions module
- **Dependencies**: Tasks 3, 5

### 8. Remove Unified Handler Components
- **Description**: Clean up the common module by removing unified handler code
- **Actions**: 
  - Delete common/unified_handler.rs
  - Remove unified handler exports from common/mod.rs
  - Delete or refactor output_handler_message.rs if no longer shared
  - Update any remaining imports in the codebase
  - Ensure no orphaned code remains
- **Success Criteria**: No unified handler code remains in common module
- **Dependencies**: Tasks 4, 5, 6, 7

### 9. Update Handler Factory Functions
- **Description**: Create separate factory functions for queries and reactions
- **Actions**: 
  - Create create_query_handler() in queries module
  - Create create_reaction_handler() in reactions module
  - Update handler definition parsing in each module
  - Remove create_output_handler() from common
  - Update all call sites to use appropriate factory
- **Success Criteria**: Separate factory functions work correctly
- **Dependencies**: Tasks 2, 3, 8

### 10. Test and Validate Separation
- **Description**: Ensure the separated handlers work correctly
- **Actions**: 
  - Run existing tests for query processing
  - Run existing tests for reaction handling
  - Add tests for new handler traits if needed
  - Verify no cross-dependencies between query and reaction handlers
  - Check for any performance regressions
- **Success Criteria**: All tests pass, no shared dependencies remain
- **Dependencies**: Tasks 1-9

## Execution Order
1. Task 1 - Analyze Current Handler Dependencies
2. Task 2 - Create Query-Specific Handler Infrastructure
3. Task 3 - Create Reaction-Specific Handler Infrastructure
4. Task 4 - Update QueryResultObserver
5. Task 5 - Update ReactionObserver
6. Task 6 - Migrate Redis Result Stream Handler
7. Task 7 - Migrate HTTP Reaction Handler
8. Task 8 - Remove Unified Handler Components
9. Task 9 - Update Handler Factory Functions
10. Task 10 - Test and Validate Separation

## Notes
- This refactoring will create clearer separation of concerns between query and reaction handling
- Some code duplication is acceptable to achieve proper separation
- Bootstrap functionality is specific to queries and should not appear in reaction handlers
- Consider keeping truly common types (like basic message structures) in common if needed
- Ensure backward compatibility is maintained for existing configurations
- The separation should make it easier to add new handler types specific to each domain