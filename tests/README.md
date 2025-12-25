# Test Suite

This directory contains automated tests for the QR Code Generator worker.

## Test Types

### 1. Unit Tests (Rust)

Located in `src/lib.rs`, these test the core QR code generation logic:

- **Basic QR generation**: Tests that QR codes can be generated from various inputs
- **Empty string handling**: Verifies empty strings are handled correctly
- **Long string handling**: Tests with very long input strings
- **PNG rendering**: Validates PNG image generation
- **Special characters**: Tests Unicode, emojis, URLs with query params, and newlines

**Run unit tests:**
```bash
cargo test --lib
# or
npm run test:unit
```

### 2. Integration Tests (API)

Located in `tests/api_tests.sh`, these test the HTTP API endpoints:

- Health check endpoint
- POST `/api/generate` with PNG format
- GET `/api/qr` with query parameters
- Error handling (missing data, invalid format, etc.)

**Run API tests:**
```bash
# First, start the worker in another terminal:
# wrangler dev --port 8787

# Then run the tests:
bash tests/api_tests.sh
# or
npm run test:api
```

**Run with custom URL:**
```bash
bash tests/api_tests.sh http://localhost:8787
```

### 3. Integration Tests (Rust)

Located in `tests/integration.rs` - placeholder for future Rust-based integration tests using worker test utilities.

## Running All Tests

```bash
npm run test:all
```

This will:
1. Run all unit tests
2. Run API integration tests (requires running worker)

## Test Reports

### Generate Coverage Reports

**HTML Coverage Report:**
```bash
npm run test:coverage
```
Opens `test-reports/tarpaulin-report.html` in your browser showing:
- Line-by-line code coverage
- File coverage percentages
- Overall project coverage statistics

**XML Coverage Report (for CI/CD):**
```bash
npm run test:coverage:xml
```
Generates `test-reports/cobertura.xml` for CI/CD integration.

### Generate JUnit XML Reports

**Rust Unit Tests:**
```bash
npm run test:junit
```
Generates `test-reports/junit.xml` with test results.

**API Integration Tests:**
```bash
npm run test:api:junit
```
Generates `test-reports/api-tests-junit.xml` with API test results.

**All Reports:**
```bash
npm run test:reports
```
Generates both coverage and JUnit reports.

### Viewing Reports

- **HTML Coverage**: Open `test-reports/tarpaulin-report.html` in your browser
- **JUnit XML**: Can be consumed by CI/CD systems, test viewers, or code quality tools

## Test Coverage

- ✅ QR code generation for various inputs
- ✅ PNG format output
- ✅ Error handling
- ✅ Special characters and Unicode
- ✅ API endpoints (when worker is running)

## Continuous Integration

These tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Run unit tests
  run: cargo test --lib

- name: Start worker
  run: wrangler dev --port 8787 &
  
- name: Run API tests
  run: bash tests/api_tests.sh
```

