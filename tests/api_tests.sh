#!/bin/bash
# API Integration Tests
# These tests require a running worker instance on localhost:8787

set -e

BASE_URL="${1:-http://localhost:8787}"
PASSED=0
FAILED=0

test_endpoint() {
    local name="$1"
    local method="$2"
    local url="$3"
    local data="$4"
    local expected_status="${5:-200}"
    
    echo -n "Testing $name... "
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$url" \
            -H "Content-Type: application/json" \
            -d "$data")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$url")
    fi
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" -eq "$expected_status" ]; then
        echo "✓ PASSED"
        ((PASSED++))
        return 0
    else
        echo "✗ FAILED (expected $expected_status, got $http_code)"
        echo "  Response: $body"
        ((FAILED++))
        return 1
    fi
}

echo "Running API tests against $BASE_URL"
echo "=================================="
echo ""

# Test health endpoint
test_endpoint "Health check" "GET" "$BASE_URL/api/health"

# Test POST /api/generate with SVG
test_endpoint "POST generate SVG" "POST" "$BASE_URL/api/generate" \
    '{"data":"https://example.com","format":"svg"}'

# Test POST /api/generate with PNG
test_endpoint "POST generate PNG" "POST" "$BASE_URL/api/generate" \
    '{"data":"https://example.com","format":"png"}'

# Test GET /api/qr with SVG
test_endpoint "GET QR SVG" "GET" "$BASE_URL/api/qr?data=test123&format=svg"

# Test GET /api/qr with PNG
test_endpoint "GET QR PNG" "GET" "$BASE_URL/api/qr?data=test123&format=png"

# Test error handling - missing data
test_endpoint "Missing data field" "POST" "$BASE_URL/api/generate" \
    '{"format":"svg}"' 400

# Test error handling - invalid format
test_endpoint "Invalid format" "POST" "$BASE_URL/api/generate" \
    '{"data":"test","format":"invalid"}' 400

# Test error handling - missing query param
test_endpoint "Missing query param" "GET" "$BASE_URL/api/qr" 400

echo ""
echo "=================================="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"

if [ $FAILED -eq 0 ]; then
    echo "All tests passed! ✓"
    exit 0
else
    echo "Some tests failed ✗"
    exit 1
fi

