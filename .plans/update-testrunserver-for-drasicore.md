# Work Plan: Update TestRunDrasiServer for DrasiServerCore Changes

## Overview
DrasiServerCore has undergone changes in how it's started and managed. The `start_legacy()` method is now being used, which consumes the DrasiServerCore instance and returns a ServerHandle. TestRunDrasiServer needs to be updated to properly handle these lifecycle changes and clarify its role as a wrapper around an embedded library.

## Tasks

### 1. Analyze Current DrasiServerCore Usage Pattern
- **Description**: Understand the new lifecycle model where DrasiServerCore is consumed by start_legacy()
- **Actions**: 
  - Review how ServerHandle is obtained from start_legacy()
  - Understand what ServerHandle provides (access to core, application handles)
  - Document the ownership transfer pattern (Arc unwrapping)
  - Identify all places where DrasiServerCore is accessed
- **Success Criteria**: 
  - Clear understanding of the new lifecycle
  - Documentation of the consumption pattern
- **Dependencies**: None

### 2. Rename and Clarify TestRunDrasiServer
- **Description**: Consider renaming TestRunDrasiServer to better reflect it wraps an embedded library
- **Actions**: 
  - Evaluate name options: TestRunDrasiServerCore, TestRunEmbeddedDrasi, TestRunDrasiInstance
  - Update class documentation to emphasize embedded nature
  - Remove any remaining server-like terminology
  - Update field names if needed (e.g., server_handle might need clarification)
- **Success Criteria**: 
  - Class name and documentation clearly indicate embedded library usage
  - No confusion with standalone server concepts
- **Dependencies**: Task 1

### 3. Fix ServerHandle Import and Type Issues
- **Description**: Resolve the missing ServerHandle type import and ensure proper typing
- **Actions**: 
  - Add proper import for ServerHandle from drasi_server
  - Verify ServerHandle is the correct type being returned by start_legacy()
  - Update type annotations throughout the module
  - Fix any compilation errors related to type changes
- **Success Criteria**: 
  - Code compiles without errors
  - All types are properly imported and used
- **Dependencies**: Task 1

### 4. Update Start Method Logic
- **Description**: Refactor the start() method to handle the new consumption pattern
- **Actions**: 
  - Document why Arc::try_unwrap is needed (ownership transfer)
  - Add better error handling for Arc unwrapping failures
  - Consider if the Arc wrapper is still needed given the consumption pattern
  - Update logging to reflect embedded library startup
- **Success Criteria**: 
  - Start method handles ownership transfer correctly
  - Clear error messages if unwrapping fails
  - Appropriate logging for embedded library lifecycle
- **Dependencies**: Task 3

### 5. Simplify Application Handle Management
- **Description**: Review if separate ApplicationHandle storage is still needed
- **Actions**: 
  - Check if ServerHandle provides access to application handles
  - Remove redundant storage if handles are accessible via ServerHandle
  - Update getter methods to use ServerHandle's access patterns
  - Simplify the overall handle management strategy
- **Success Criteria**: 
  - No duplicate storage of handles
  - Simplified access patterns
  - Reduced complexity in handle management
- **Dependencies**: Task 3

### 6. Update Method Documentation
- **Description**: Update all method documentation to reflect embedded library semantics
- **Actions**: 
  - Remove references to "server" in method docs
  - Clarify that this manages an embedded Drasi instance
  - Document the consumption pattern in start()
  - Add examples showing proper usage patterns
- **Success Criteria**: 
  - All documentation reflects embedded library nature
  - Usage patterns are clear to developers
  - No misleading server terminology
- **Dependencies**: Tasks 2, 4

### 7. Review and Update Tests
- **Description**: Ensure tests reflect the new lifecycle and embedded nature
- **Actions**: 
  - Update test names to avoid "server" terminology where inappropriate
  - Add tests for Arc unwrapping edge cases
  - Verify tests cover the consumption pattern
  - Add tests for embedded library lifecycle events
- **Success Criteria**: 
  - Tests accurately reflect the new implementation
  - Edge cases are covered
  - Test names are clear and accurate
- **Dependencies**: Tasks 4, 5

### 8. Consider Lifecycle Improvements
- **Description**: Evaluate if the current lifecycle model can be improved
- **Actions**: 
  - Consider if consuming DrasiServerCore is the best approach
  - Evaluate alternatives that don't require Arc unwrapping
  - Document why the current pattern is necessary
  - Propose improvements if applicable
- **Success Criteria**: 
  - Clear documentation of design decisions
  - Identified improvements (if any)
  - Plan for future enhancements
- **Dependencies**: All previous tasks

## Execution Order
1. Task 1 (Analyze Current Usage) - Understand the changes
2. Task 3 (Fix Imports) - Get code compiling
3. Task 4 (Update Start Logic) - Core functionality fix
4. Task 5 (Simplify Handles) - Clean up redundancy
5. Task 2 (Rename/Clarify) - Improve naming once we understand the pattern
6. Task 6 (Update Documentation) - Document the changes
7. Task 7 (Update Tests) - Ensure test coverage
8. Task 8 (Consider Improvements) - Future planning

## Notes
- The consumption pattern (Arc::try_unwrap) suggests DrasiServerCore might not be designed for the current usage pattern
- Consider if a different initialization pattern would be cleaner
- The embedded library nature should be emphasized throughout
- ServerHandle appears to be a legacy name that might need updating in drasi_server itself
- The mixing of "server" terminology with embedded library concepts creates confusion