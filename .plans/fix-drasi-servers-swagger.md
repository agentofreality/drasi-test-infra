# Work Plan: Fix Drasi Servers Swagger Documentation

## Overview
The drasi-servers section in the Swagger documentation is not displaying any endpoints despite the code having proper OpenAPI annotations. This plan addresses the issue to ensure all drasi-server endpoints are properly documented and visible in the Swagger UI.

## Tasks

### 1. Investigate Router Registration Issue
- **Description**: Verify that drasi-servers routes are properly registered in the main router
- **Actions**: 
  - Check the route nesting in web_api/mod.rs
  - Verify the path structure matches OpenAPI path definitions
  - Ensure no route conflicts or overlapping paths
  - Validate that the router is correctly merged into the main app
- **Success Criteria**: 
  - Identify why drasi-servers endpoints are not appearing
  - Document the root cause of the issue
- **Dependencies**: None

### 2. Fix OpenAPI Tag Consistency
- **Description**: Ensure OpenAPI tag names match between endpoint annotations and tag definitions
- **Actions**: 
  - Review tag names in drasi_servers.rs (uses "drasi_servers" with underscore)
  - Check tag definition in openapi.rs (uses "drasi-servers" with hyphen)
  - Standardize on one naming convention (prefer hyphens for REST URLs)
  - Update all #[utoipa::path] annotations to use consistent tag names
- **Success Criteria**: 
  - All drasi-servers endpoints use the same tag name
  - Tag name matches the tag definition in OpenApi derive
- **Dependencies**: Task 1

### 3. Correct Path Definitions
- **Description**: Fix path definitions to match actual route structure
- **Actions**: 
  - Current paths in annotations: `/test_run_host/drasi_servers`
  - Actual route nesting may differ due to router setup
  - Update all path annotations to reflect correct full paths
  - Consider if routes should be nested differently
- **Success Criteria**: 
  - OpenAPI paths match actual HTTP endpoint paths
  - Swagger UI shows correct URLs for testing
- **Dependencies**: Task 1

### 4. Add Missing Response Type Schemas
- **Description**: Ensure all response types are properly registered in OpenAPI components
- **Actions**: 
  - Verify TestRunDrasiServerConfig schema is imported correctly
  - Check if all response body types have ToSchema derive
  - Add any missing schemas to the components section
  - Validate JSON examples in schema definitions
- **Success Criteria**: 
  - All response types have complete schema definitions
  - No missing schema warnings in Swagger UI
- **Dependencies**: None

### 5. Test OpenAPI Generation
- **Description**: Verify the OpenAPI JSON is correctly generated
- **Actions**: 
  - Access `/api-docs/openapi.json` endpoint
  - Validate JSON structure includes drasi-servers paths
  - Check for any generation errors or warnings
  - Use OpenAPI validator tools if needed
- **Success Criteria**: 
  - OpenAPI JSON contains all drasi-servers endpoints
  - JSON validates against OpenAPI 3.0 specification
- **Dependencies**: Tasks 2, 3, 4

### 6. Update Swagger UI Integration
- **Description**: Ensure Swagger UI properly displays the corrected documentation
- **Actions**: 
  - Clear browser cache and reload Swagger UI
  - Verify drasi-servers section appears with all endpoints
  - Test "Try it out" functionality for each endpoint
  - Check request/response examples render correctly
- **Success Criteria**: 
  - Drasi-servers section visible in Swagger UI
  - All 5 endpoints (create, list, get, delete, status) displayed
  - Interactive testing works correctly
- **Dependencies**: Task 5

## Execution Order
1. Task 1 (Investigate Router Registration)
2. Task 2 (Fix Tag Consistency) - Primary suspected issue
3. Task 3 (Correct Path Definitions)
4. Task 4 (Add Missing Schemas)
5. Task 5 (Test OpenAPI Generation)
6. Task 6 (Update Swagger UI)

## Notes
- The most likely issue is the tag name mismatch: "drasi_servers" in code vs "drasi-servers" in OpenAPI definition
- Router nesting structure needs careful review to ensure paths match
- Consider adding integration tests for OpenAPI generation to prevent future issues
- Document the fix in comments to help future developers
- May need to restart the service after fixes for changes to take effect