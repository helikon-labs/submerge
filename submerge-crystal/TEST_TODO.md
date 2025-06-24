# Submerge Crystal - Test Suggestions

## Unit Tests (Missing)

### 1. Block Processing Logic Tests
- [ ] **test_ingest_block_duplicate_detection**
  - Test that duplicate blocks are properly skipped
  - Verify "Block X had already been ingested" path works correctly
  - Ensure database queries are optimized for duplicate detection

- [ ] **test_extrinsic_event_count_parsing**
  - Test SCALE codec decoding of ExtrinsicCount/EventCount from trace data
  - Test edge cases: None values, malformed hex strings
  - Verify proper error handling when decode fails

- [ ] **test_block_validation_logic**
  - Test processed_extrinsic_count validation against expected count
  - Test key matching logic for ExtrinsicData storage entries
  - Verify error messages are descriptive when validation fails

### 2. API Endpoint Tests
- [ ] **test_block_hash_validation**
  - Test hash length validation (must be 64 hex characters)
  - Test rejection of invalid hex characters
  - Test proper handling of 0x prefix (should be stripped)

- [ ] **test_block_number_parsing**
  - Test valid and invalid block number parsing
  - Test boundary conditions (u64::MAX, negative numbers)
  - Test mixed hash/number input edge cases

### 3. Metrics Tests
- [ ] **test_metrics_registration**
  - Verify all metrics are properly registered with Prometheus
  - Test metric name consistency and prefixing
  - Ensure metrics don't panic on registration conflicts

## Integration Tests (Expand existing)

### 4. Database Transaction Tests
- [ ] **test_transaction_rollback_on_error**
  - Simulate error during block ingestion process
  - Verify no partial data is committed to database
  - Test that trace_error table is properly updated on failures

- [ ] **test_concurrent_block_ingestion**
  - Test race conditions with IS_BUSY atomic flag
  - Verify block processing order is maintained
  - Test behavior when multiple finalized blocks arrive quickly

### 5. API Integration Tests
- [ ] **test_api_endpoints_with_real_data**
  - Test /block/{number}/trace with various valid inputs
  - Test pagination handling for large result sets
  - Test proper error responses (404 for missing blocks, 400 for invalid input)

- [ ] **test_api_metrics_collection**
  - Verify request metrics are updated on API calls
  - Test response time tracking accuracy
  - Test status code counters for different response types

## Error Handling Tests

### 6. Network Failure Tests
- [ ] **test_substrate_client_timeout**
  - Mock RPC timeouts and connection failures
  - Verify errors are properly saved to trace_error table
  - Test retry logic and exponential backoff

- [ ] **test_database_connection_failure**
  - Test graceful handling of database disconnection
  - Verify connection pool recovery mechanisms
  - Test transaction retry logic

### 7. Data Validation Tests
- [ ] **test_malformed_trace_data**
  - Test handling of invalid hex in trace values
  - Test missing required fields in block headers
  - Test SCALE decode failures with malformed data

## Performance Tests

### 8. Load Tests
- [ ] **test_block_ingestion_performance**
  - Measure ingestion rate for large block ranges (1000+ blocks)
  - Test memory usage patterns under sustained load
  - Benchmark database insertion performance

- [ ] **test_api_concurrent_requests**
  - Test API performance under concurrent load (100+ requests)
  - Verify connection pool management under stress
  - Test response time degradation patterns

## Missing Test Infrastructure

### 9. Test Helpers Needed
- [ ] **Mock Substrate Client**
  - Create mock implementation of SubstrateClient for unit tests
  - Support configurable responses and error injection
  - Avoid network calls in unit test suite

- [ ] **Test Database Fixtures**
  - Create helper functions for setting up test database state
  - Provide sample block/trace data for consistent testing
  - Implement cleanup helpers for test isolation

- [ ] **Assertion Helpers**
  - Create custom assertions for database state verification
  - Add helpers for comparing block trace data structures
  - Implement metric value assertion helpers

### 10. Property-Based Tests
- [ ] **test_with_random_valid_block_data**
  - Generate random but valid block data for comprehensive testing
  - Test edge cases around block number boundaries
  - Test various chain specification configurations

- [ ] **test_storage_key_variations**
  - Test different storage key formats and patterns
  - Verify proper handling of various storage hashers
  - Test edge cases in key parsing logic

## Test Organization

### Current Test Files
- `src/persistence.rs` - Contains basic integration tests for database operations
- Missing: Unit test files, API test files, error handling test files

### Suggested Test Structure
```
tests/
├── unit/
│   ├── block_processing.rs
│   ├── api_validation.rs
│   └── metrics.rs
├── integration/
│   ├── database.rs
│   ├── api_endpoints.rs
│   └── error_handling.rs
├── performance/
│   ├── ingestion_benchmarks.rs
│   └── api_load_tests.rs
└── helpers/
    ├── mock_substrate.rs
    ├── test_fixtures.rs
    └── assertions.rs
```

## Priority

### High Priority
1. Block processing logic unit tests (core business logic)
2. Database transaction rollback tests (data integrity)
3. API endpoint validation tests (security)

### Medium Priority
4. Error handling and network failure tests
5. Performance benchmarks
6. Property-based testing framework

### Low Priority
7. Mock infrastructure improvements
8. Test organization refactoring