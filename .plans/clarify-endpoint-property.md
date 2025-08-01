# Work Plan: Clarify or Remove Misleading Endpoint Property

## Overview
The `endpoint` property appears in DrasiServerCore details but always returns `null` because DrasiServerCore doesn't expose any web API endpoints. This creates confusion for users who might expect an endpoint to be available. The plan addresses whether to remove this property or clarify its purpose.

## Tasks

### 1. Analyze Current Endpoint Usage
- **Description**: Understand why the endpoint property exists and where it's used
- **Actions**: 
  - Review all references to `get_api_endpoint()` method
  - Check if any code depends on this property
  - Investigate historical context - was this for a planned feature?
  - Examine if it's used in any client code or examples
- **Success Criteria**: 
  - Complete understanding of endpoint property usage
  - List of all code that references or depends on it
- **Dependencies**: None

### 2. Document the Current Architecture
- **Description**: Add clear documentation explaining why endpoint is always null
- **Actions**: 
  - Update method documentation for `get_api_endpoint()`
  - Add comments explaining DrasiServerCore vs DrasiServer differences
  - Update API response schemas to clarify the null value
  - Consider adding a note in the Swagger documentation
- **Success Criteria**: 
  - Clear inline documentation explaining the null endpoint
  - No user confusion about missing endpoints
- **Dependencies**: Task 1

### 3. Consider Removing Endpoint from API Responses
- **Description**: Evaluate if the endpoint field should be removed entirely
- **Actions**: 
  - Identify all API responses that include the endpoint field
  - Check for backward compatibility concerns
  - Consider using `#[serde(skip_serializing_if = "Option::is_none")]`
  - Evaluate impact on existing clients
- **Success Criteria**: 
  - Decision made on whether to remove or keep the field
  - If keeping, clear justification documented
  - If removing, migration path identified
- **Dependencies**: Task 1

### 4. Implement Chosen Solution
- **Description**: Either remove the endpoint field or improve its documentation
- **Actions**: 
  - Option A: Remove endpoint from API responses
    - Update response DTOs to exclude endpoint
    - Remove from Swagger schemas
    - Update any client code
  - Option B: Keep but clarify
    - Add detailed documentation
    - Consider renaming to `web_api_endpoint` for clarity
    - Add explanation in response examples
- **Success Criteria**: 
  - No confusion about the endpoint property
  - API responses are clear and accurate
- **Dependencies**: Task 3

### 5. Update Related Documentation
- **Description**: Ensure all documentation accurately reflects the architecture
- **Actions**: 
  - Update CLAUDE.md with clearer explanation
  - Add notes to API documentation
  - Update any example responses in tests
  - Consider adding an architecture diagram
- **Success Criteria**: 
  - Documentation clearly explains DrasiServerCore architecture
  - No misleading information about endpoints
- **Dependencies**: Task 4

### 6. Add Validation Tests
- **Description**: Ensure the endpoint behavior is properly tested
- **Actions**: 
  - Add tests verifying endpoint is always None/null
  - Test API responses include proper endpoint value
  - Add documentation tests for API examples
  - Consider adding integration tests
- **Success Criteria**: 
  - Tests prevent regression
  - Behavior is locked in and documented
- **Dependencies**: Task 4

## Execution Order
1. Task 1 (Analyze Usage) - Understand current state
2. Task 2 (Document Architecture) - Quick win for clarity
3. Task 3 (Evaluate Removal) - Make architectural decision
4. Task 4 (Implement Solution) - Execute the decision
5. Task 5 (Update Documentation) - Ensure consistency
6. Task 6 (Add Tests) - Prevent future confusion

## Notes
- The root issue is that DrasiServerCore is an embedded library, not a standalone server
- The test infrastructure provides its own REST API that wraps DrasiServerCore
- This architectural choice should be made more explicit
- Consider if other properties might be similarly misleading (e.g., port)
- Backward compatibility is important - many tests may expect this field
- The null endpoint might be intentional to maintain API consistency with full DrasiServer