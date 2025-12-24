#!/bin/bash
# Generate JUnit XML report from Rust test output
# This script runs cargo test and converts the output to JUnit XML format

set -e

mkdir -p test-reports

# Run tests and capture output
TEST_OUTPUT=$(cargo test --lib 2>&1) || TEST_EXIT_CODE=$?

# Extract test results
TOTAL_TESTS=$(echo "$TEST_OUTPUT" | grep -oP 'running \K\d+' | head -1 || echo "0")
PASSED_TESTS=$(echo "$TEST_OUTPUT" | grep -c "test.*\.\.\. ok" || echo "0")
FAILED_TESTS=$(echo "$TEST_OUTPUT" | grep -c "test.*\.\.\. FAILED" || echo "0")

# Get test names and results
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%S)

cat > test-reports/junit.xml <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="Rust Unit Tests" tests="$TOTAL_TESTS" failures="$FAILED_TESTS" time="0">
  <testsuite name="lib" tests="$TOTAL_TESTS" failures="$FAILED_TESTS" time="0" timestamp="$TIMESTAMP">
EOF

# Extract individual test results
while IFS= read -r line; do
    if [[ $line =~ test\ ([a-zA-Z0-9_:]+)\.\.\.\ (ok|FAILED) ]]; then
        TEST_NAME="${BASH_REMATCH[1]}"
        TEST_STATUS="${BASH_REMATCH[2]}"
        
        if [ "$TEST_STATUS" = "ok" ]; then
            cat >> test-reports/junit.xml <<EOF
    <testcase classname="lib" name="$TEST_NAME" time="0"/>
EOF
        else
            cat >> test-reports/junit.xml <<EOF
    <testcase classname="lib" name="$TEST_NAME" time="0">
      <failure message="Test failed">Test failed</failure>
    </testcase>
EOF
        fi
    fi
done <<< "$TEST_OUTPUT"

cat >> test-reports/junit.xml <<EOF
  </testsuite>
</testsuites>
EOF

echo "JUnit XML report generated: test-reports/junit.xml"
echo "Total tests: $TOTAL_TESTS, Passed: $PASSED_TESTS, Failed: $FAILED_TESTS"

# Exit with the test exit code
exit ${TEST_EXIT_CODE:-0}

