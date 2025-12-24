# Testing Guide

This document explains how to run tests and generate test reports for the QR Code Generator.

## Prerequisites

### Install Test Tools

**cargo-tarpaulin** (for code coverage):
```bash
cargo install cargo-tarpaulin
```

## Quick Start

### Run All Tests
```bash
npm run test:all
```

### Generate Test Reports
```bash
npm run test:reports
```

## Test Commands

### Unit Tests (Rust)

**Run unit tests:**
```bash
npm run test:unit
# or
cargo test --lib
```

**Generate JUnit XML:**
```bash
npm run test:junit
# Output: test-reports/junit.xml
```

### API Integration Tests

**Run API tests (requires running worker):**
```bash
# Terminal 1: Start worker
wrangler dev --port 8787

# Terminal 2: Run tests
npm run test:api
```

**Generate JUnit XML:**
```bash
npm run test:api:junit
# Output: test-reports/api-tests-junit.xml
```

### Code Coverage

**HTML Coverage Report:**
```bash
npm run test:coverage
# Opens: test-reports/tarpaulin-report.html
```

**XML Coverage Report (for CI/CD):**
```bash
npm run test:coverage:xml
# Output: test-reports/cobertura.xml
```

## Report Locations

All reports are generated in the `test-reports/` directory:

- `tarpaulin-report.html` - HTML coverage report
- `cobertura.xml` - XML coverage report
- `junit.xml` - Rust unit test results (JUnit format)
- `api-tests-junit.xml` - API test results (JUnit format)

## Viewing Reports

### HTML Coverage Report
Open `test-reports/tarpaulin-report.html` in your browser to see:
- Line-by-line code coverage
- File coverage percentages
- Overall project coverage statistics
- Uncovered lines highlighted

### JUnit XML Reports
These can be consumed by:
- **CI/CD Systems**: GitHub Actions, GitLab CI, Jenkins, CircleCI
- **Test Viewers**: TestRail, Allure, ReportPortal
- **Code Quality Tools**: SonarQube, CodeClimate

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin
        
      - name: Run tests with coverage
        run: |
          npm install
          npm run test:coverage:xml
          npm run test:junit
          
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          file: ./test-reports/cobertura.xml
          
      - name: Upload test results
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: test-reports/
```

## Test Coverage Goals

- **Target**: 80%+ code coverage
- **Critical paths**: 100% coverage (QR generation, error handling)
- **Focus areas**: Core QR generation logic, API endpoints

## Troubleshooting

### cargo-tarpaulin not found
```bash
cargo install cargo-tarpaulin
```

### API tests failing
- Ensure worker is running: `wrangler dev --port 8787`
- Check worker logs for errors
- Verify port 8787 is available

### JUnit XML not generating
- Check that `test-reports/` directory exists
- Verify test script has execute permissions: `chmod +x tests/*.sh`

