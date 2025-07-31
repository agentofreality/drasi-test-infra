# Work Plan: Validate Drasi Server Integration

## Overview
Validate that the hosted drasi-server integration is working correctly with the test infrastructure, specifically testing the DrasiServerCallbackHandler, DrasiServerChannelHandler, DrasiServerChannelSourceChangeDispatcher, and DrasiServerApiSourceChangeDispatcher components using the building_comfort example.

## Tasks

### 1. Analyze Current Implementation
- **Description**: Review the existing implementation of all four handlers/dispatchers
- **Actions**: 
  - Examine DrasiServerCallbackHandler implementation and integration points
  - Examine DrasiServerChannelHandler implementation and channel communication
  - Review DrasiServerChannelSourceChangeDispatcher for source data flow
  - Review DrasiServerApiSourceChangeDispatcher for API-based data flow
  - Check how these components integrate with DrasiServerCore
- **Success Criteria**: Clear understanding of expected behavior and integration points
- **Dependencies**: None

### 2. Run Existing Building Comfort Test
- **Description**: Execute the building_comfort test in drasi_server_internal folder
- **Actions**: 
  - Run the test using run_test.sh script
  - Monitor console output for any errors or warnings
  - Check if data flows properly through the configured dispatchers/handlers
  - Verify test completion with expected output count (10 records)
- **Success Criteria**: Test runs without errors and produces expected outputs
- **Dependencies**: Task 1

### 3. Validate DrasiServerChannelSourceChangeDispatcher
- **Description**: Verify source data is properly dispatched to internal Drasi Server
- **Actions**: 
  - Check that facilities-db source receives change events
  - Verify channel buffer size is respected (2048 as configured)
  - Monitor for any dropped events or channel overflow
  - Examine debug logs for dispatcher behavior
- **Success Criteria**: All change events reach the Drasi Server source without loss
- **Dependencies**: Task 2

### 4. Validate DrasiServerChannelHandler
- **Description**: Verify reaction outputs are properly received from Drasi Server
- **Actions**: 
  - Check that building-comfort-alerts reaction sends outputs
  - Verify channel buffer size is respected (1024 as configured)
  - Monitor reaction output count matches stop trigger (10 records)
  - Examine console and JSONL file outputs for correctness
- **Success Criteria**: All reaction outputs are received and logged correctly
- **Dependencies**: Task 2

### 5. Test DrasiServerApiSourceChangeDispatcher
- **Description**: Create a test configuration using API-based dispatcher
- **Actions**: 
  - Modify config.json to use DrasiServerApi dispatcher instead of Channel
  - Run the test and verify API endpoints are called correctly
  - Check for proper batching behavior if enabled
  - Monitor timeout handling (default 30 seconds)
- **Success Criteria**: API dispatcher successfully sends events to Drasi Server
- **Dependencies**: Task 2

### 6. Test DrasiServerCallbackHandler
- **Description**: Create a test configuration using callback-based handler
- **Actions**: 
  - Modify config.json to use DrasiServerCallback handler
  - Implement or verify callback mechanism for reaction outputs
  - Run the test and verify callbacks are invoked
  - Check callback type handling
- **Success Criteria**: Callback handler receives and processes reaction outputs
- **Dependencies**: Task 2

### 7. Stress Test Channel Communication
- **Description**: Test channel-based components under load
- **Actions**: 
  - Increase change_count in model generator to 100+ events
  - Reduce buffer sizes to test overflow handling
  - Monitor for backpressure and dropped events
  - Verify error handling and recovery
- **Success Criteria**: System handles load gracefully with proper error reporting
- **Dependencies**: Tasks 3, 4

### 8. Integration Test with External Drasi Server
- **Description**: Test with an actual external Drasi Server instance
- **Actions**: 
  - Deploy a real Drasi Server instance
  - Configure test to use external server endpoints
  - Test both API and Channel dispatchers/handlers
  - Verify connectivity and data flow
- **Success Criteria**: All components work with external Drasi Server
- **Dependencies**: Tasks 5, 6

## Execution Order
1. Task 1 - Analyze Current Implementation
2. Task 2 - Run Existing Building Comfort Test
3. Task 3 - Validate DrasiServerChannelSourceChangeDispatcher (in parallel with 4)
4. Task 4 - Validate DrasiServerChannelHandler (in parallel with 3)
5. Task 5 - Test DrasiServerApiSourceChangeDispatcher
6. Task 6 - Test DrasiServerCallbackHandler
7. Task 7 - Stress Test Channel Communication
8. Task 8 - Integration Test with External Drasi Server (optional)

## Notes
- The building_comfort example uses an internal Drasi Server with memory storage
- Debug logging is enabled for key components in run_test.sh
- Test data includes building hierarchy with room sensors (temperature, CO2, humidity)
- Stop trigger is configured for 10 reaction output records
- Both Console and JSONL file loggers are configured for output observation