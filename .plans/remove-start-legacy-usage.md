# Work Plan: Remove start_legacy Usage from TestRunDrasiServer

## Overview
The `start_legacy()` method has been removed from DrasiServerCore. This breaks the current implementation of TestRunDrasiServer which relies on this method to start the embedded DrasiServerCore and obtain a ServerHandle. We need to find and implement an alternative approach to manage DrasiServerCore lifecycle.

## Tasks

### 1. Investigate New DrasiServerCore API
- **Description**: Understand how DrasiServerCore should now be started and managed without start_legacy
- **Actions**: 
  - Research DrasiServerCore documentation/source for new start mechanism
  - Identify what replaces ServerHandle (if anything)
  - Understand new lifecycle management approach
  - Determine how to access core managers after initialization
- **Success Criteria**: 
  - Clear understanding of new API
  - Documentation of replacement patterns
  - Identified migration path
- **Dependencies**: None

### 2. Remove ServerHandle Dependencies
- **Description**: Remove or replace all ServerHandle usage throughout the codebase
- **Actions**: 
  - Remove ServerHandle field from TestRunDrasiServer struct
  - Update all methods that use server_handle
  - Find alternative ways to access DrasiServerCore
  - Update stop() method which relies on ServerHandle
- **Success Criteria**: 
  - No more references to ServerHandle
  - Alternative access patterns implemented
- **Dependencies**: Task 1

### 3. Redesign DrasiServerCore Lifecycle Management
- **Description**: Implement new approach to manage DrasiServerCore without consuming it
- **Actions**: 
  - Keep DrasiServerCore in an Arc without unwrapping
  - Implement direct initialization instead of start_legacy
  - Handle component startup differently (if needed)
  - Ensure proper cleanup/shutdown without ServerHandle
- **Success Criteria**: 
  - DrasiServerCore properly initialized
  - Components accessible after initialization
  - Clean shutdown mechanism works
- **Dependencies**: Tasks 1, 2

### 4. Update Application Handle Collection
- **Description**: Revise how application handles are collected after core initialization
- **Actions**: 
  - Access managers directly from DrasiServerCore Arc
  - Remove dependency on ServerHandle.core()
  - Ensure handles are collected at the right lifecycle phase
  - Update handle storage mechanism if needed
- **Success Criteria**: 
  - Application handles properly collected
  - Direct access to core managers works
  - No ServerHandle dependencies
- **Dependencies**: Task 3

### 5. Fix Compilation Errors
- **Description**: Address all compilation errors from removed start_legacy and ServerHandle
- **Actions**: 
  - Remove undefined type imports
  - Fix all type mismatches
  - Update method signatures as needed
  - Ensure all modules compile
- **Success Criteria**: 
  - Project compiles without errors
  - All type issues resolved
- **Dependencies**: Tasks 2, 3, 4

### 6. Update Component Status Verification
- **Description**: Revise how component status is verified after initialization
- **Actions**: 
  - Access query/source/reaction managers directly
  - Remove ServerHandle.core() usage
  - Ensure status checks work with new approach
  - Update logging to reflect new patterns
- **Success Criteria**: 
  - Component status properly verified
  - Direct manager access works
  - Appropriate logging in place
- **Dependencies**: Task 3

### 7. Simplify State Management
- **Description**: Consider if the complex state management can be simplified without ServerHandle
- **Actions**: 
  - Review if separate state tracking is still needed
  - Consider using DrasiServerCore's internal state
  - Simplify initialization flow
  - Remove unnecessary Arc wrapping if possible
- **Success Criteria**: 
  - Cleaner state management
  - Reduced complexity
  - Maintainable code
- **Dependencies**: Tasks 3, 4

### 8. Update Tests and Documentation
- **Description**: Ensure tests work with new implementation and docs are updated
- **Actions**: 
  - Fix any broken tests
  - Add tests for new lifecycle management
  - Update method documentation
  - Update CLAUDE.md with new patterns
- **Success Criteria**: 
  - All tests pass
  - Documentation accurate
  - Usage patterns clear
- **Dependencies**: All previous tasks

## Execution Order
1. Task 1 (Investigate New API) - Critical to understand the replacement
2. Task 2 (Remove ServerHandle) - Unblock other changes
3. Task 3 (Redesign Lifecycle) - Core implementation change
4. Task 4 (Update Handle Collection) - Fix handle management
5. Task 5 (Fix Compilation) - Get code building
6. Task 6 (Update Status Verification) - Fix functionality
7. Task 7 (Simplify State) - Clean up implementation
8. Task 8 (Tests and Docs) - Ensure quality

## Notes
- The removal of start_legacy suggests DrasiServerCore now has a simpler initialization model
- We may no longer need the Arc unwrapping pattern
- Consider if TestRunDrasiServer can be significantly simplified
- The embedded library nature should guide the new design
- Without ServerHandle, we likely need direct access to DrasiServerCore throughout its lifecycle