# Submerge Crystal - TODO

## Code Quality & Refactoring

### 1. Extract Constants and Configuration
- [ ] **Extract hard-coded values**
  - Move worker count (10) to configuration
  - Extract hex parsing constants (64 char length)
  - Create constants for storage keys ("System", "ExtrinsicCount", etc.)
  - Define timeout and retry constants

- [ ] **Centralize configuration management**
  - Create `Config` struct with validation
  - Support environment variables via `.env` files
  - Add configuration validation at startup
  - Implement configuration reload without restart

### 2. Method Complexity Reduction
- [ ] **Refactor `ingest_block` method** (lib.rs:61-162)
  - Extract extrinsic/event count parsing into separate function
  - Extract block validation logic into separate function
  - Extract storage key processing into helper functions
  - Reduce method from 100+ lines to <50 lines

- [ ] **Break down `run` method** (lib.rs:204-325)
  - Extract batch processing logic
  - Extract subscription handling logic
  - Create separate methods for different execution modes

- [ ] **Simplify persistence methods**
  - Extract common SQL query patterns
  - Create reusable transaction helpers
  - Reduce code duplication in database operations

### 3. Error Handling Improvements
- [ ] **Replace generic `anyhow::Error` with domain-specific errors**
  - Create `CrystalError` enum with specific variants
  - Add context-aware error messages
  - Implement proper error conversion chains
  - Add structured error logging

- [ ] **Improve error recovery**
  - Add exponential backoff for retries
  - Implement circuit breaker pattern for external services
  - Add graceful degradation strategies
  - Better error propagation with context

### 4. Type Safety Enhancements
- [ ] **Create newtype wrappers**
  - `BlockNumber(u64)` for type safety
  - `BlockHash(Vec<u8>)` with validation
  - `RuntimeVersion(u32)` for clarity
  - Strong typing for hex strings

- [ ] **Add validation helpers**
  - Block hash format validation
  - Block number range validation
  - Hex string validation utilities
  - Input sanitization functions

## Performance Optimizations

### 5. Database Performance
- [ ] **Implement batch processing**
  - Process multiple blocks in single transactions
  - Batch insert operations where possible
  - Optimize query patterns for bulk operations
  - Add connection pooling tuning

- [ ] **Add database indexing strategy**
  - Index frequently queried columns (`block_number`, `runtime_version`)
  - Composite indexes for common query patterns
  - Analyze query performance and optimize
  - Add database performance monitoring

- [ ] **Optimize memory usage**
  - Stream large result sets instead of loading all into memory
  - Implement pagination for API responses
  - Add memory usage monitoring
  - Optimize data structure sizes

### 6. Concurrency Improvements
- [ ] **Improve concurrent processing**
  - Replace single `IS_BUSY` flag with more granular locking
  - Implement work-stealing for block processing
  - Add parallel processing for independent operations
  - Optimize async task spawning

- [ ] **Connection pool optimization**
  - Tune pool size based on workload
  - Add connection health checks
  - Implement connection recycling
  - Monitor pool utilization

## API Enhancements

### 7. REST API Improvements
- [ ] **Add comprehensive API endpoints**
  - `/health` endpoint for monitoring
  - `/metrics` endpoint for Prometheus scraping
  - `/status` endpoint for service status
  - `/blocks` endpoint with pagination

- [ ] **Implement proper HTTP handling**
  - Add CORS support for web clients
  - Implement request rate limiting
  - Add request/response compression
  - Proper HTTP status codes and error responses

- [ ] **API Documentation**
  - Generate OpenAPI/Swagger specification
  - Add endpoint documentation with examples
  - Create API client libraries
  - Add API versioning strategy

### 8. Response Format Standardization
- [ ] **Standardize API response format**
  - Consistent error response structure
  - Add metadata to responses (pagination, timing)
  - Implement response caching where appropriate
  - Add response validation

## Observability & Monitoring

### 9. Enhanced Metrics
- [ ] **Add business metrics**
  - Block ingestion rate (blocks/second)
  - Processing lag (current vs latest block)
  - Error rate by error type
  - Database query performance metrics

- [ ] **Implement distributed tracing**
  - Add OpenTelemetry integration
  - Trace request flows through system
  - Add span annotations for key operations
  - Implement trace sampling

- [ ] **Improve logging**
  - Switch to structured JSON logging
  - Add correlation IDs for request tracking
  - Implement log levels configuration
  - Add contextual logging information

### 10. Health Checks & Alerting
- [ ] **Implement comprehensive health checks**
  - Database connectivity health
  - Substrate RPC health
  - Service dependency health
  - Memory and CPU usage monitoring

- [ ] **Define SLIs/SLOs**
  - Block processing latency targets
  - API response time targets
  - Error rate thresholds
  - Availability targets

## Testing (from TEST_TODO.md)

### 11. Unit Tests (Missing)
- [ ] **Block Processing Logic Tests**
  - test_ingest_block_duplicate_detection
  - test_extrinsic_event_count_parsing
  - test_block_validation_logic

- [ ] **API Endpoint Tests**
  - test_block_hash_validation
  - test_block_number_parsing

- [ ] **Metrics Tests**
  - test_metrics_registration

### 12. Integration Tests (Expand existing)
- [ ] **Database Transaction Tests**
  - test_transaction_rollback_on_error
  - test_concurrent_block_ingestion

- [ ] **API Integration Tests**
  - test_api_endpoints_with_real_data
  - test_api_metrics_collection

### 13. Error Handling Tests
- [ ] **Network Failure Tests**
  - test_substrate_client_timeout
  - test_database_connection_failure

- [ ] **Data Validation Tests**
  - test_malformed_trace_data

### 14. Performance Tests
- [ ] **Load Tests**
  - test_block_ingestion_performance
  - test_api_concurrent_requests

### 15. Test Infrastructure
- [ ] **Mock Infrastructure**
  - Mock Substrate Client
  - Test Database Fixtures
  - Assertion Helpers

- [ ] **Property-Based Tests**
  - test_with_random_valid_block_data
  - test_storage_key_variations

## Architecture Improvements

### 16. Modularity Enhancements
- [ ] **Separate API into dedicated crate**
  - Move API code to `submerge-api` crate
  - Create shared types crate for API/Crystal communication
  - Implement proper service boundaries
  - Add API versioning support

- [ ] **Extract business logic**
  - Create domain service layer
  - Separate persistence from business logic
  - Implement repository pattern
  - Add service interfaces/traits

### 17. Configuration & Deployment
- [ ] **Deployment improvements**
  - Add Docker containerization
  - Create Kubernetes deployment manifests
  - Add health check endpoints for orchestration
  - Implement graceful shutdown handling

- [ ] **Configuration management**
  - Support multiple configuration sources
  - Add configuration validation
  - Implement feature flags
  - Add runtime configuration updates

## Documentation

### 18. Code Documentation
- [ ] **Add comprehensive docs**
  - Document public APIs with examples
  - Add architecture decision records (ADRs)
  - Create developer setup guide
  - Add troubleshooting documentation

- [ ] **README improvements**
  - Add usage examples
  - Document configuration options
  - Add performance tuning guide
  - Include monitoring setup instructions

## Priority

### High Priority (P0)
1. Method complexity reduction (security & maintainability)
2. Error handling improvements (reliability)
3. Unit test implementation (quality assurance)
4. Configuration management (operational readiness)

### Medium Priority (P1)
5. Database performance optimizations
6. API enhancements and documentation
7. Enhanced monitoring and metrics
8. Integration test expansion

### Low Priority (P2)
9. Architecture refactoring
10. Advanced performance optimizations
11. Deployment and infrastructure improvements
12. Advanced testing frameworks

## Implementation Notes

- **Dependencies**: Some items depend on others (e.g., error types before error handling tests)
- **Breaking Changes**: Configuration and API changes may require migration strategy
- **Performance**: Benchmark before and after performance optimizations
- **Testing**: Implement tests incrementally alongside refactoring
- **Documentation**: Update docs as code changes are made