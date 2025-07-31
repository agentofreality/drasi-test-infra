# Work Plan: Fix drasi_core Logging Not Being Suppressed

## Overview
Investigate and fix the issue where drasi_core logging messages continue to appear in the console output despite setting `RUST_LOG=info,drasi_core::query::continuous_query=off` in the test script. The logs are appearing from drasi_core modules (continuous_query and path_solver) when running the building_comfort internal Drasi server test.

## Tasks

### 1. Analyze Logging Initialization Order
- **Description**: Investigate how and when logging is initialized across the test framework components
- **Actions**: 
  - Trace the initialization sequence in test-service main.rs
  - Check if DrasiServerCore initializes its own logger
  - Verify if multiple env_logger::init() calls are happening
  - Determine which component initializes logging first
- **Success Criteria**: Understand the exact order of logging initialization
- **Dependencies**: None

### 2. Test Different RUST_LOG Filter Syntaxes
- **Description**: Verify the correct syntax for filtering drasi_core logs
- **Actions**: 
  - Test with `drasi_core=off` to disable all drasi_core logs
  - Test with `drasi_core::query=off` to disable query-related logs
  - Test with `drasi_core::query::continuous_query=off,drasi_core::path_solver=off`
  - Try using `error` level instead of `off`: `drasi_core=error`
  - Test with module target syntax: `target[drasi_core]=off`
- **Success Criteria**: Find a working filter syntax that suppresses the logs
- **Dependencies**: None

### 3. Investigate drasi_core Logging Mechanism
- **Description**: Check if drasi_core uses a different logging mechanism or hardcoded log statements
- **Actions**: 
  - Search drasi_core source for log macros usage
  - Check if drasi_core uses tracing instead of log crate
  - Verify if there are any hardcoded println! statements
  - Look for any custom logging initialization in drasi_core
- **Success Criteria**: Understand how drasi_core produces its log output
- **Dependencies**: Access to drasi-server/drasi-core source

### 4. Modify Logging Initialization in test-service
- **Description**: Ensure test-service controls the logging configuration
- **Actions**: 
  - Move env_logger::init() to the very beginning of main()
  - Set RUST_LOG before any other imports or initializations
  - Consider using env_logger::Builder for more control
  - Add logging configuration validation after init
- **Success Criteria**: test-service successfully controls all logging
- **Dependencies**: Task 1 completion

### 5. Create Custom Logger Configuration
- **Description**: Implement a more sophisticated logging setup if simple filters don't work
- **Actions**: 
  - Use env_logger::Builder to create custom filter
  - Implement regex-based filtering for drasi_core modules
  - Add runtime log level adjustment capability
  - Consider using tracing-subscriber if env_logger limitations exist
- **Success Criteria**: Complete control over drasi_core log output
- **Dependencies**: Tasks 2 and 3 completion

### 6. Update Test Scripts and Documentation
- **Description**: Apply the fix to all affected test scripts and document the solution
- **Actions**: 
  - Update run_test.sh with the working RUST_LOG configuration
  - Update run_test_debug.sh similarly
  - Add comments explaining the logging configuration
  - Update CLAUDE.md with logging configuration notes
  - Create a troubleshooting section for common logging issues
- **Success Criteria**: All test scripts work correctly and documentation is clear
- **Dependencies**: Tasks 4 or 5 completion

## Execution Order
1. Task 1 - Analyze Logging Initialization Order
2. Task 2 - Test Different RUST_LOG Filter Syntaxes (can run in parallel with Task 1)
3. Task 3 - Investigate drasi_core Logging Mechanism
4. Task 4 - Modify Logging Initialization in test-service (based on findings from Tasks 1-3)
5. Task 5 - Create Custom Logger Configuration (only if Task 4 doesn't solve the issue)
6. Task 6 - Update Test Scripts and Documentation

## Notes
- The issue appears to be related to env_logger only being initialized once per process
- Multiple components (test-service, drasi-server) may be competing for logger initialization
- The RUST_LOG filter syntax needs to match the actual module paths in drasi_core
- Consider that drasi_core might be using the tracing crate instead of log crate
- The solution should work for both regular and debug test runs
- Performance impact of logging should be considered for production tests