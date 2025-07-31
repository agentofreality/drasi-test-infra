# Work Plan: Investigate TestRunHost DrasiServerCore Startup Management

## Overview
Investigate how TestRunHost manages DrasiServerCore startup to determine if the "no queries subscribed" issue is caused by improper startup sequencing or configuration passing in the test framework rather than in DrasiServerCore itself.

## Tasks

### 1. Analyze TestRunHost Configuration Passing
- **Description**: Verify that TestRunHost correctly passes the full DrasiServerCore configuration including sources, queries, and reactions
- **Actions**: 
  - Review how TestRunDrasiServer is created in `test-run-host/src/drasi_servers/mod.rs`
  - Check if all configuration fields are properly mapped from TestDrasiServerConfig to DrasiServerCore config
  - Verify that auto_start flags are preserved for all components
  - Examine if query-to-source subscriptions are properly configured
- **Success Criteria**: Confirm all config fields are passed correctly to DrasiServerCore
- **Dependencies**: None

### 2. Examine Startup Synchronization
- **Description**: Determine if TestRunHost properly waits for DrasiServerCore to complete startup before proceeding
- **Actions**: 
  - Review the start_immediately logic in TestRunDrasiServer
  - Check if server.start().await properly blocks until all components are started
  - Verify if ApplicationHandles are stored only after successful startup
  - Look for any async/await issues that might cause premature continuation
- **Success Criteria**: Confirm TestRunHost waits for complete DrasiServerCore initialization
- **Dependencies**: Task 1

### 3. Trace Component Startup Order
- **Description**: Map the exact sequence of events from TestRunHost starting a server to test data dispatch
- **Actions**: 
  - Document when DrasiServerCore.start() is called
  - Identify when test sources begin dispatching data
  - Check if there's a timing gap between server startup and data dispatch
  - Review logs for the actual startup sequence vs expected sequence
- **Success Criteria**: Complete timeline of startup events documented
- **Dependencies**: Task 2

### 4. Investigate TestSource Timing
- **Description**: Analyze if test sources are dispatching data too early before queries are ready
- **Actions**: 
  - Review when TestRunSource starts sending data via DrasiServerChannel
  - Check if start_immediately for sources considers query readiness
  - Examine the DrasiServerChannelDispatcher initialization timing
  - Verify if there's proper coordination between server startup and source activation
- **Success Criteria**: Understand exact timing of test data dispatch
- **Dependencies**: Task 3

### 5. Review ApplicationHandle Management
- **Description**: Verify that ApplicationHandles for sources and reactions are properly obtained and stored
- **Actions**: 
  - Check how ApplicationSourceHandle and ApplicationReactionHandle are obtained
  - Verify these handles are valid and connected to the running DrasiServerCore
  - Examine if the handles are used correctly for data dispatch
  - Look for any handle lifecycle issues
- **Success Criteria**: Confirm handles are properly managed throughout the test lifecycle
- **Dependencies**: Task 2

### 6. Test Startup Synchronization Fix
- **Description**: Implement and test potential fixes for any timing issues found
- **Actions**: 
  - Add explicit waits or synchronization if needed
  - Ensure queries are fully started before source data dispatch
  - Add logging to confirm proper startup sequence
  - Run the building_comfort example to verify the fix
- **Success Criteria**: "No queries subscribed" error no longer appears in logs
- **Dependencies**: Tasks 3, 4, 5

## Execution Order
1. Task 1 - Analyze TestRunHost Configuration Passing
2. Task 2 - Examine Startup Synchronization  
3. Task 3 - Trace Component Startup Order (parallel with Task 4)
4. Task 4 - Investigate TestSource Timing (parallel with Task 3)
5. Task 5 - Review ApplicationHandle Management
6. Task 6 - Test Startup Synchronization Fix

## Notes
- Focus on the building_comfort/drasi_server_internal example as the test case
- The issue appears to be in the test framework, not DrasiServerCore itself
- Pay special attention to async/await patterns that might cause race conditions
- Consider adding debug logging to trace the exact sequence of events
- The fix likely involves ensuring proper synchronization between server startup and test data dispatch