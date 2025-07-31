#!/bin/bash

# Run the test for a short time and capture output
timeout 3 ./examples/building_comfort/drasi_server_internal/run_test.sh 2>&1 | tee test_output.log

echo "========== Checking for key events =========="
echo "Initial inserts sent:"
grep -c "Sending initial insert" test_output.log || echo "0"

echo "Events dispatched to ApplicationSourceHandle:"
grep -c "Dispatching event.*to source.*via ApplicationSourceHandle" test_output.log || echo "0"

echo "Events successfully dispatched:"
grep -c "Successfully dispatched.*events to source.*via ApplicationSourceHandle" test_output.log || echo "0"

echo "Reaction outputs:"
grep -c "ConsoleLogger" test_output.log || echo "0"

echo "========== Sample of dispatched events =========="
grep "Dispatching event" test_output.log | head -5

echo "========== ApplicationHandle status =========="
grep "ApplicationHandle" test_output.log | head -5

echo "========== Reaction output files =========="
find examples/building_comfort/drasi_server_internal/test_data_cache -name "*.jsonl" -type f -exec sh -c 'echo "File: $1, Lines: $(wc -l < "$1")"' _ {} \; 2>/dev/null

rm -f test_output.log