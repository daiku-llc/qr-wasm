# Test Suite

This directory contains automated tests for the QR Code Generator worker.

## Test Types

### 1. Unit Tests (Rust)

Located in `src/lib.rs`, these test the core QR code generation logic:

- **Basic QR generation**: Tests that QR codes can be generated from various inputs
- **Empty string handling**: Verifies empty strings are handled correctly
- **Long string handling**: Tests with very long input strings
- **SVG rendering**: Validates SVG output format
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
- POST `/api/generate` with SVG format
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

## Test Coverage

- ✅ QR code generation for various inputs
- ✅ SVG format output
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

