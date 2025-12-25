#!/bin/bash
# API Integration Tests
# These tests require a running worker instance on localhost:8787
# Usage: bash tests/api_tests.sh [base_url] [--junit]

set +e  # Don't exit on first error, we want to count failures

BASE_URL="${1:-http://localhost:8787}"
if [ "$1" = "--junit" ]; then
    BASE_URL="http://localhost:8787"
    JUNIT_OUTPUT=true
elif [ "$2" = "--junit" ]; then
    JUNIT_OUTPUT=true
else
    JUNIT_OUTPUT=false
fi

PASSED=0
FAILED=0
TEST_COUNT=0
START_TIME=$(date +%s)
TEST_RESULTS=()

test_endpoint() {
    local name="$1"
    local method="$2"
    local url="$3"
    local data="$4"
    local expected_status="${5:-200}"
    
    ((TEST_COUNT++))
    local test_start=$(date +%s.%N)
    
    if [ "$JUNIT_OUTPUT" = true ]; then
        echo -n ""
    else
        echo -n "Testing $name... "
    fi
    
    local http_code
    local body
    local error_msg=""
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$url" \
            -H "Content-Type: application/json" \
            -d "$data" 2>&1) || error_msg="$response"
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$url" 2>&1) || error_msg="$response"
    fi
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    local test_end=$(date +%s.%N)
    # Calculate duration (fallback if bc not available)
    if command -v bc &> /dev/null; then
        local duration=$(echo "$test_end - $test_start" | bc)
    else
        local duration="0"
    fi
    
    if [ -n "$error_msg" ] || [ "$http_code" != "$expected_status" ]; then
        if [ "$JUNIT_OUTPUT" = false ]; then
            echo "✗ FAILED (expected $expected_status, got ${http_code:-error})"
            echo "  Response: ${body:-$error_msg}"
        fi
        ((FAILED++))
        TEST_RESULTS+=("FAILED|$name|$duration|Expected $expected_status, got ${http_code:-error}")
        return 1
    else
        if [ "$JUNIT_OUTPUT" = false ]; then
            echo "✓ PASSED"
        fi
        ((PASSED++))
        TEST_RESULTS+=("PASSED|$name|$duration")
        return 0
    fi
}

if [ "$JUNIT_OUTPUT" = false ]; then
    echo "Running API tests against $BASE_URL"
    echo "=================================="
    echo ""
fi

# Test health endpoint
test_endpoint "Health check" "GET" "$BASE_URL/api/health"

# Test POST /api/generate with PNG
test_endpoint "POST generate PNG" "POST" "$BASE_URL/api/generate" \
    '{"data":"https://example.com"}'

# Test GET /api/qr with PNG
test_endpoint "GET QR PNG" "GET" "$BASE_URL/api/qr?data=test123"

# Test error handling - missing data
test_endpoint "Missing data field" "POST" "$BASE_URL/api/generate" \
    '{}' 400

# Test error handling - missing query param
test_endpoint "Missing query param" "GET" "$BASE_URL/api/qr" 400

END_TIME=$(date +%s)
TOTAL_DURATION=$((END_TIME - START_TIME))

if [ "$JUNIT_OUTPUT" = true ]; then
    # Generate JUnit XML
    mkdir -p test-reports
    cat > test-reports/api-tests-junit.xml <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="API Integration Tests" tests="$TEST_COUNT" failures="$FAILED" time="$TOTAL_DURATION">
  <testsuite name="API Tests" tests="$TEST_COUNT" failures="$FAILED" time="$TOTAL_DURATION" timestamp="$(date -u +%Y-%m-%dT%H:%M:%S)">
EOF
    
    for result in "${TEST_RESULTS[@]}"; do
        IFS='|' read -r status name duration message <<< "$result"
        if [ "$status" = "PASSED" ]; then
            cat >> test-reports/api-tests-junit.xml <<EOF
    <testcase classname="API" name="$name" time="$duration"/>
EOF
        else
            cat >> test-reports/api-tests-junit.xml <<EOF
    <testcase classname="API" name="$name" time="$duration">
      <failure message="$message">$message</failure>
    </testcase>
EOF
        fi
    done
    
    cat >> test-reports/api-tests-junit.xml <<EOF
  </testsuite>
</testsuites>
EOF
    echo "JUnit XML report generated: test-reports/api-tests-junit.xml"
else
    echo ""
    echo "=================================="
    echo "Tests passed: $PASSED"
    echo "Tests failed: $FAILED"
    echo "Total duration: ${TOTAL_DURATION}s"
    
    if [ $FAILED -eq 0 ]; then
        echo "All tests passed! ✓"
        exit 0
    else
        echo "Some tests failed ✗"
        exit 1
    fi
fi
