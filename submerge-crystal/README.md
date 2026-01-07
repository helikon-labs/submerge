## Submerge Crystal

The main indexer component. Indexes genesis records, blocks, extrinsics, calls within extrinsics, events, traces, logs, and metadata (versions, pallets, calls, events, constants, errors). Deployed per chain.

### Code Quality Improvements

#### High Priority

##### 1. Comprehensive Test Suite
- **Unit Tests**: Add unit tests for core business logic
  - API handlers (`api/v1/*.rs`)
  - Persistence layer (`persistence/mod.rs`, `persistence/api/*.rs`)
  - Metadata parsing (`types/metadata/*.rs`)
  - Worker logic (`worker/processor/*.rs`)
  - Error handling (`types/api/error.rs`)

- **Integration Tests**: Add end-to-end API tests
  - Test all API endpoints with real database
  - Test worker processing flows
  - Test error scenarios and edge cases

- **Test Coverage Goal**: Aim for >80% code coverage

##### 2. Complete Inline Documentation
- Add `///` doc comments for all public APIs
- Document complex functions and algorithms
- Add examples in doc comments where helpful
- Document error conditions and return values
- Add doc tests (`#![doc(test)]`) for public APIs

#### Medium Priority

##### 3. Module Refactoring
- **Split large modules**:
  - `persistence/mod.rs` (~1205 lines) → Split into:
    - `persistence/block.rs`
    - `persistence/event.rs`
    - `persistence/extrinsic.rs`
    - `persistence/call.rs`
    - `persistence/metadata.rs`
    - `persistence/genesis.rs`
  
  - `worker/processor/mod.rs` (~759 lines) → Split into:
    - `worker/processor/block.rs`
    - `worker/processor/validation.rs`
    - Keep core logic in `mod.rs`

##### 4. Error Handling Improvements
- Review and add context to remaining `unwrap()`/`expect()` calls
- Replace `expect()` in `types/legacy.rs:93` with proper error handling
- Add validation for type conversions (e.g., `i32 as u32`)
- Consider custom error types for domain-specific errors

##### 5. Technical Debt Cleanup
- Remove or document `#[allow(dead_code)]` attributes:
  - `worker/mod.rs:55` - `WorkerStatus`
  - `worker/mod.rs:386` - `WorkerManager` methods
  - `api/mod.rs:93` - `ServiceState`
  - `api/v1/system.rs` - Multiple items
  
- Refactor functions with `#[allow(clippy::too_many_arguments)]`:
  - `persistence/mod.rs:163` - Consider using config structs
  - `worker/processor/extrinsic.rs:551` - Consider using config structs

- Refactor functions with `#[allow(clippy::too_many_lines)]`:
  - `types/metadata/mod.rs:291` - Split into smaller functions

#### Low Priority

##### 6. Performance Optimizations
- Make `INSERT_BATCH_SIZE` configurable (currently hardcoded to 1000)
- Expand caching strategy beyond `SESSION_VALIDATORS_CACHE`
- Consider connection pooling optimizations
- Add performance benchmarks for critical paths

##### 7. Enhanced Observability
- Add more structured logging context
- Add distributed tracing spans for request flows
- Enhance metrics with more detailed labels
- Add health check endpoints

##### 8. Configuration Improvements
- Extract magic numbers to constants or configuration
- Make retry delays and timeouts configurable
- Add validation for configuration values

### Current Code Quality Score: 8.0/10

#### Strengths
- ✅ Excellent architecture and module organization
- ✅ Strong type safety and Rust idioms
- ✅ Minimal public API surface (good encapsulation)
- ✅ Proper error handling with custom error types
- ✅ Well-designed REST API with OpenAPI documentation
- ✅ Good async/await patterns
- ✅ Proper transaction management

#### Areas for Improvement
- ⚠️ Testing coverage (currently minimal)
- ⚠️ Inline documentation (missing doc comments)
- ⚠️ Some large modules need refactoring
- ⚠️ Technical debt from `#[allow(...)]` attributes

#### Path to 10/10
1. Add comprehensive test suite → +1.0
2. Complete inline documentation → +0.5
3. Split large modules → +0.3
4. Remove technical debt → +0.2