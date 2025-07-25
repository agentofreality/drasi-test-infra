# Work Plan: Extend REST API for Reactions Support

## Overview
Extend the E2E Test Framework's REST API to include support for managing reactions, expanding both the web API layer in test-service and the underlying TestRunHost functionality.

## Tasks

### 1. Create Reactions Router Module
- **Description**: Create a new web API module for handling reaction endpoints
- **Actions**: 
  - Create new file `e2e-test-framework/test-service/src/web_api/reactions.rs`
  - Define route handlers for CRUD operations on reactions
  - Implement OpenAPI documentation annotations
  - Create response types matching the pattern used in queries.rs
- **Success Criteria**: New reactions.rs module exists with all necessary route handlers
- **Dependencies**: None

### 2. Update Web API Module Registration
- **Description**: Register the reactions routes in the main web API module
- **Actions**: 
  - Import reactions module in `web_api/mod.rs`
  - Add `pub mod reactions;` declaration
  - Update the API router to include reactions routes
  - Ensure reactions routes are nested under `/test_run_host` path
- **Success Criteria**: Reactions routes are accessible via the API
- **Dependencies**: Task 1

### 3. Update TestRunHostStateResponse
- **Description**: Add reaction tracking to the service state response
- **Actions**: 
  - Add `test_run_reaction_ids` field to TestRunHostStateResponse struct
  - Update the get_service_info_handler to populate reaction IDs
  - Add appropriate OpenAPI schema examples
- **Success Criteria**: Service info endpoint returns reaction IDs
- **Dependencies**: None

### 4. Implement Reaction List Endpoint
- **Description**: Create GET /test_run_host/reactions endpoint
- **Actions**: 
  - Implement get_reaction_list_handler in reactions.rs
  - Return list of reaction IDs from TestRunHost
  - Add OpenAPI documentation
  - Handle error states appropriately
- **Success Criteria**: Can retrieve list of all reaction IDs
- **Dependencies**: Task 1

### 5. Implement Get Reaction State Endpoint
- **Description**: Create GET /test_run_host/reactions/{id} endpoint
- **Actions**: 
  - Implement get_reaction_handler
  - Return detailed reaction state information
  - Add proper path parameter validation
  - Include OpenAPI documentation
- **Success Criteria**: Can retrieve individual reaction state
- **Dependencies**: Task 1

### 6. Implement Reaction Control Endpoints
- **Description**: Create reaction control endpoints (start, stop, pause, reset)
- **Actions**: 
  - Implement POST /test_run_host/reactions/{id}/start
  - Implement POST /test_run_host/reactions/{id}/stop
  - Implement POST /test_run_host/reactions/{id}/pause
  - Implement POST /test_run_host/reactions/{id}/reset
  - Add OpenAPI documentation for each endpoint
- **Success Criteria**: All reaction control operations work via API
- **Dependencies**: Task 1

### 7. Implement Create Reaction Endpoint
- **Description**: Create POST /test_run_host/reactions endpoint
- **Actions**: 
  - Implement post_reaction_handler
  - Accept TestRunReactionConfig in request body
  - Call TestRunHost::add_test_reaction
  - Return created reaction state
  - Add request body validation and OpenAPI docs
- **Success Criteria**: Can create new reactions via API
- **Dependencies**: Task 1

### 8. Update OpenAPI Documentation
- **Description**: Ensure all reaction endpoints are properly documented
- **Actions**: 
  - Add reaction-related schema types to OpenAPI spec
  - Ensure all endpoints have proper tags
  - Add example requests and responses
  - Update the main API documentation
- **Success Criteria**: Swagger UI shows all reaction endpoints with documentation
- **Dependencies**: Tasks 4-7

### 9. Add Integration Tests
- **Description**: Create integration tests for the new reaction endpoints
- **Actions**: 
  - Create test file for reaction API endpoints
  - Test CRUD operations
  - Test state transitions (start, stop, pause, reset)
  - Test error handling
- **Success Criteria**: All reaction endpoints have test coverage
- **Dependencies**: Tasks 1-7

### 10. Update Existing Components
- **Description**: Update existing components to be aware of reactions
- **Actions**: 
  - Ensure TestRunHost properly exposes reaction counts
  - Update any shared types or utilities
  - Verify no breaking changes to existing functionality
- **Success Criteria**: Existing functionality continues to work with reactions added
- **Dependencies**: Tasks 1-8

## Execution Order
1. Task 1 - Create Reactions Router Module
2. Task 2 - Update Web API Module Registration
3. Task 3 - Update TestRunHostStateResponse
4. Task 4 - Implement Reaction List Endpoint
5. Task 5 - Implement Get Reaction State Endpoint
6. Task 6 - Implement Reaction Control Endpoints
7. Task 7 - Implement Create Reaction Endpoint
8. Task 8 - Update OpenAPI Documentation
9. Task 9 - Add Integration Tests
10. Task 10 - Update Existing Components

## Notes
- Follow the same patterns established in queries.rs and sources.rs for consistency
- Ensure all endpoints check TestRunHost status before processing
- Use proper error handling with TestServiceWebApiError types
- Maintain OpenAPI documentation standards for all new endpoints
- The reaction endpoints should mirror the query endpoints structure where applicable