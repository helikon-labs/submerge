# Comprehensive Testing Strategy for Submerge Crystal

## Overview

This document outlines the approach for building a comprehensive test suite that achieves >80% code coverage for the `submerge-crystal` crate.

## Testing Philosophy

1. **Test Pyramid**: Unit tests (70%) > Integration tests (25%) > E2E tests (5%)
2. **Test Isolation**: Each test should be independent and repeatable
3. **Fast Feedback**: Unit tests should run in milliseconds
4. **Realistic Integration**: Integration tests use real database but isolated data
5. **Coverage Goals**: >80% line coverage, 100% for critical paths

## Test Infrastructure

### 1. Test Dependencies

Add to `Cargo.toml`:

```toml
[dev-dependencies]
# Existing
serde_json = { workspace = true }

# New additions
mockall = "0.12"  # For mocking dependencies
tokio-test = "0.4"  # For async test utilities
testcontainers = "0.15"  # For database containerization
testcontainers-modules = "0.3"  # Pre-built containers
wiremock = "0.6"  # For HTTP mocking (legacy decode API)
assert_matches = "1.5"  # Better assertion messages
criterion = "0.5"  # For benchmarks (optional)
```

### 2. Test Database Setup

**Option A: Testcontainers (Recommended)**
- Use PostgreSQL container for integration tests
- Automatic cleanup after tests
- Isolated test environment

**Option B: Shared Test Database**
- Use existing `get_test_postgres()` helper
- Requires manual database setup
- Faster but less isolated

### 3. Test Utilities Module

Create `tests/common/mod.rs` with:
- Database setup/teardown helpers
- Test data fixtures (blocks, events, extrinsics, etc.)
- Mock builders for complex types
- Assertion helpers

## Test Structure

```
tests/
├── common/
│   ├── mod.rs              # Test utilities
│   ├── fixtures.rs        # Test data builders
│   ├── database.rs        # DB setup/teardown
│   └── mocks.rs           # Mock implementations
├── unit/
│   ├── api/
│   │   ├── block_test.rs
│   │   ├── call_test.rs
│   │   ├── event_test.rs
│   │   ├── extrinsic_test.rs
│   │   ├── metadata_test.rs
│   │   └── trace_test.rs
│   ├── persistence/
│   │   ├── block_test.rs
│   │   ├── event_test.rs
│   │   └── metadata_test.rs
│   ├── types/
│   │   ├── error_test.rs
│   │   └── metadata_test.rs
│   └── worker/
│       └── processor_test.rs
├── integration/
│   ├── api/
│   │   ├── block_api_test.rs
│   │   ├── call_api_test.rs
│   │   ├── event_api_test.rs
│   │   ├── extrinsic_api_test.rs
│   │   ├── metadata_api_test.rs
│   │   └── trace_api_test.rs
│   ├── persistence/
│   │   └── persistence_test.rs
│   └── worker/
│       └── worker_test.rs
└── e2e/
    └── full_flow_test.rs
```

## Detailed Test Plans

### 1. Unit Tests for API Handlers (`api/v1/*.rs`)

#### Test Strategy
- **Mock dependencies**: Use `mockall` to mock `PostgreSQLStorage` and `WorkerManager`
- **Test each handler independently**: Focus on request/response transformation
- **Cover all error paths**: Invalid inputs, not found, database errors

#### Example: `block_test.rs`

```rust
// Test cases:
1. get_blocks()
   - ✅ Valid query with pagination
   - ✅ Empty result set
   - ✅ Invalid page number (< 1)
   - ✅ Invalid page size (> max)
   - ✅ Invalid author address
   - ✅ Database error handling
   - ✅ Pagination edge cases (last page, single item)

2. get_blocks_by_reference()
   - ✅ Valid block number
   - ✅ Valid block hash (with 0x prefix)
   - ✅ Valid block hash (without 0x prefix)
   - ✅ Invalid block reference format
   - ✅ Block not found (number)
   - ✅ Block not found (hash)
   - ✅ Multiple blocks with same number (pruned scenario)
```

#### Coverage Target: 90%+ for all API handlers

### 2. Unit Tests for Persistence Layer (`persistence/mod.rs`, `persistence/api/*.rs`)

#### Test Strategy
- **Use test database**: Real PostgreSQL for realistic testing
- **Test CRUD operations**: Create, Read, Update, Delete
- **Test transactions**: Rollback scenarios, conflict handling
- **Test batch operations**: Verify batch size limits

#### Example: `persistence/block_test.rs`

```rust
// Test cases:
1. ingest_block()
   - ✅ Insert new block
   - ✅ Update existing block (ON CONFLICT)
   - ✅ Transaction rollback on error
   - ✅ Batch insertion

2. get_block_by_hash()
   - ✅ Existing block
   - ✅ Non-existent block
   - ✅ Database connection error

3. get_blocks_by_number()
   - ✅ Single block
   - ✅ Multiple blocks (pruned scenario)
   - ✅ No blocks found

4. update_block_status()
   - ✅ Status transition (proposed → finalized)
   - ✅ Invalid status
   - ✅ Non-existent block
```

#### Coverage Target: 85%+ for persistence layer

### 3. Unit Tests for Error Handling (`types/api/error.rs`)

#### Test Strategy
- **Test all error variants**: Every enum variant
- **Test error conversions**: `From` implementations
- **Test HTTP responses**: Status codes, response bodies
- **Test error messages**: Verify user-friendly messages

#### Example: `types/error_test.rs`

```rust
// Test cases:
1. APIError::message()
   - ✅ All error variants return appropriate messages
   - ✅ Messages include context (hashes, numbers)

2. APIError::status_code()
   - ✅ Correct HTTP status for each variant
   - ✅ 400 for BadRequest
   - ✅ 404 for NotFound
   - ✅ 500 for InternalServerError

3. From implementations
   - ✅ From<anyhow::Error>
   - ✅ From<hex::FromHexError>
   - ✅ From<string::FromUtf8Error>

4. IntoResponse
   - ✅ Correct JSON structure
   - ✅ Correct status code
   - ✅ Error body format
```

#### Coverage Target: 100% (critical for API correctness)

### 4. Unit Tests for Metadata Parsing (`types/metadata/*.rs`)

#### Test Strategy
- **Test metadata versions**: V8-V15
- **Test parsing edge cases**: Missing fields, invalid data
- **Test type conversions**: PortableType handling

#### Example: `types/metadata_test.rs`

```rust
// Test cases:
1. Metadata::try_from()
   - ✅ V14 metadata parsing
   - ✅ V15 metadata parsing
   - ✅ Legacy versions (V8-V13)
   - ✅ Invalid metadata structure
   - ✅ Missing pallets
   - ✅ Missing events/calls/constants

2. get_metadata_type_by_id()
   - ✅ Valid type ID
   - ✅ Invalid type ID
   - ✅ Version-specific differences

3. get_pallet_metadata()
   - ✅ Valid pallet index
   - ✅ Invalid pallet index
   - ✅ Missing pallet
```

#### Coverage Target: 80%+ for metadata parsing

### 5. Unit Tests for Worker Logic (`worker/processor/*.rs`)

#### Test Strategy
- **Mock external dependencies**: RPC client, legacy decode API
- **Test processing logic**: Block processing, event extraction
- **Test error recovery**: Retry logic, error handling

#### Example: `worker/processor_test.rs`

```rust
// Test cases:
1. BlockProcessor::process_finalized_blocks_in_range()
   - ✅ Valid block range
   - ✅ Invalid range (start > end)
   - ✅ Empty range
   - ✅ RPC connection failure
   - ✅ Block processing failure
   - ✅ Retry logic

2. Event extraction
   - ✅ Valid events
   - ✅ Missing events
   - ✅ Invalid event data

3. Extrinsic processing
   - ✅ Signed extrinsics
   - ✅ Unsigned extrinsics
   - ✅ Nested calls
```

#### Coverage Target: 75%+ for worker logic

### 6. Integration Tests

#### Test Strategy
- **Real database**: Use testcontainers or test database
- **Full request flow**: HTTP request → handler → database → response
- **Test middleware**: Rate limiting, metrics, error handling
- **Test concurrent access**: Multiple requests, transactions

#### Example: `integration/api/block_api_test.rs`

```rust
// Test cases:
1. GET /api/v1/blocks
   - ✅ Full request/response cycle
   - ✅ Pagination works correctly
   - ✅ Query parameters filtering
   - ✅ Rate limiting enforcement
   - ✅ Metrics collection

2. GET /api/v1/blocks/{block_ref}
   - ✅ Block number lookup
   - ✅ Block hash lookup
   - ✅ 404 for non-existent blocks
   - ✅ Multiple blocks with same number

3. Error scenarios
   - ✅ Database connection failure
   - ✅ Invalid query parameters
   - ✅ Rate limit exceeded
```

#### Coverage Target: All API endpoints, critical paths

### 7. End-to-End Tests

#### Test Strategy
- **Full system test**: Database → Worker → API
- **Realistic scenarios**: Process blocks, query via API
- **Performance tests**: Load testing, concurrent requests

#### Example: `e2e/full_flow_test.rs`

```rust
// Test cases:
1. Complete indexing flow
   - ✅ Genesis processing
   - ✅ Block indexing
   - ✅ Event extraction
   - ✅ API query verification

2. Worker lifecycle
   - ✅ Worker start
   - ✅ Worker processing
   - ✅ Worker cancellation
   - ✅ Worker error recovery
```

## Implementation Approach

### Phase 1: Infrastructure Setup (Week 1)
1. Add test dependencies to `Cargo.toml`
2. Create `tests/common/` module structure
3. Set up test database helpers
4. Create test fixtures and builders

### Phase 2: Error Handling Tests (Week 1-2)
1. Test `APIError` enum thoroughly
2. Test error conversions
3. Test HTTP response generation
4. **Why first**: Errors are critical and relatively simple

### Phase 3: Unit Tests - Core Logic (Week 2-3)
1. Metadata parsing tests
2. Type conversion tests
3. Validation logic tests
4. **Why next**: Core business logic, no external dependencies

### Phase 4: Unit Tests - API Handlers (Week 3-4)
1. Mock persistence layer
2. Test all API handlers
3. Test query parameter validation
4. Test error responses
5. **Why next**: High value, can use mocks

### Phase 5: Unit Tests - Persistence (Week 4-5)
1. Test database operations
2. Test transactions
3. Test batch operations
4. Test conflict handling
5. **Why next**: Requires database but isolated

### Phase 6: Integration Tests (Week 5-6)
1. Test full API endpoints
2. Test middleware
3. Test concurrent scenarios
4. **Why last**: Most complex, requires full setup

### Phase 7: E2E Tests (Week 6)
1. Full system tests
2. Performance benchmarks
3. **Why last**: Slowest, most resource-intensive

## Test Utilities to Create

### 1. Database Helpers (`tests/common/database.rs`)

```rust
pub async fn setup_test_db() -> PostgreSQLStorage;
pub async fn teardown_test_db(db: PostgreSQLStorage);
pub async fn clear_test_data(db: &PostgreSQLStorage);
pub async fn run_migrations(db: &PostgreSQLStorage);
```

### 2. Test Fixtures (`tests/common/fixtures.rs`)

```rust
pub fn create_test_block() -> BlockRow;
pub fn create_test_event() -> EventRow;
pub fn create_test_extrinsic() -> ExtrinsicRow;
pub fn create_test_call() -> CallRow;
pub fn create_test_metadata() -> Metadata;
pub fn create_test_block_hash() -> Vec<u8>;
pub fn create_test_block_number() -> u64;
```

### 3. Mock Builders (`tests/common/mocks.rs`)

```rust
pub struct MockPostgreSQLStorage;
pub struct MockSubstrateClient;
pub struct MockLegacyDecodeAPIClient;
pub struct MockWorkerManager;
```

### 4. Assertion Helpers (`tests/common/assertions.rs`)

```rust
pub fn assert_block_eq(actual: &BlockDTO, expected: &BlockRow);
pub fn assert_pagination_valid(pagination: &PaginationData);
pub fn assert_error_response(response: Response, expected: APIError);
```

## Coverage Measurement

### Tools
- `cargo tarpaulin` - Code coverage tool
- `cargo llvm-cov` - Alternative coverage tool

### Coverage Goals
- **Overall**: >80% line coverage
- **API handlers**: >90%
- **Error handling**: 100%
- **Persistence layer**: >85%
- **Metadata parsing**: >80%
- **Worker logic**: >75%

### Coverage Reports
- Generate HTML reports
- Track coverage over time
- Fail CI if coverage drops below threshold

## Continuous Integration

### Test Execution
```bash
# Unit tests (fast)
cargo test --lib

# Integration tests (slower)
cargo test --test '*'

# All tests with coverage
cargo tarpaulin --out Html
```

### CI Pipeline
1. Run unit tests
2. Run integration tests (if database available)
3. Generate coverage report
4. Fail if coverage < 80%

## Best Practices

1. **Test Naming**: `test_<function>_<scenario>_<expected_result>`
   - Example: `test_get_blocks_with_valid_pagination_returns_blocks`

2. **Arrange-Act-Assert**: Clear test structure
   ```rust
   #[tokio::test]
   async fn test_example() {
       // Arrange
       let db = setup_test_db().await;
       let block = create_test_block();
       
       // Act
       let result = db.ingest_block(...).await;
       
       // Assert
       assert!(result.is_ok());
   }
   ```

3. **Test Isolation**: Each test should be independent
   - Use transactions that rollback
   - Clean up test data
   - Don't rely on test execution order

4. **Test Data**: Use realistic but minimal data
   - Create fixtures for common structures
   - Use builders for complex objects
   - Keep tests readable

5. **Error Testing**: Test both success and failure paths
   - Happy path
   - Error conditions
   - Edge cases
   - Boundary conditions

## Estimated Effort

- **Infrastructure Setup**: 2-3 days
- **Error Handling Tests**: 1-2 days
- **Core Logic Tests**: 3-4 days
- **API Handler Tests**: 5-7 days
- **Persistence Tests**: 5-7 days
- **Integration Tests**: 5-7 days
- **E2E Tests**: 2-3 days

**Total**: ~4-6 weeks for comprehensive test suite

## Success Metrics

1. **Coverage**: >80% line coverage achieved
2. **Test Count**: 200+ test cases
3. **Test Speed**: Unit tests < 1 second, Integration < 30 seconds
4. **CI Integration**: All tests pass in CI
5. **Documentation**: Tests serve as documentation

## Next Steps

1. Review and approve this strategy
2. Set up test infrastructure
3. Begin with Phase 1 (Error Handling Tests)
4. Iterate and refine based on findings
5. Track progress toward coverage goals

