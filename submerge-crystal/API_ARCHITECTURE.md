# Submerge Crystal - Layered API Architecture

## Overview

This document outlines the proposed **layered API architecture** where each component (including Crystal) has its own internal API, and an outer gateway API provides selective access to inner component APIs.

## Benefits of This Approach

### 1. **Separation of Concerns**
- Each component owns its domain-specific API
- Clear boundaries between different functional areas
- Reduces coupling between components

### 2. **Security & Access Control**
- Outer API acts as a **gateway/facade**
- Can implement authentication, rate limiting, input validation
- Selective exposure prevents accidental access to internal operations

### 3. **Versioning & Evolution**
- Components can evolve their APIs independently
- Outer API provides stable interface while inner APIs change
- Easier to deprecate/migrate specific functionality

### 4. **Testing & Development**
- Each component API can be tested in isolation
- Easier to mock dependencies
- Components can be developed independently

## Suggested Architecture

```rust
// Component-level APIs (internal)
mod crystal {
    pub struct CrystalApi {
        storage: Arc<PostgreSQLStorage>,
    }
    
    impl CrystalApi {
        pub async fn get_block_traces(&self, hash: &[u8]) -> Result<BlockTraces> { ... }
        pub async fn get_ingestion_status(&self) -> Result<IngestionStatus> { ... }
        pub async fn force_reprocess_block(&self, number: u64) -> Result<()> { ... }
    }
}

// Outer API (public gateway)
mod api {
    pub struct SubmergeApi {
        crystal_api: Arc<CrystalApi>,
        cortex_api: Arc<CortexApi>,  // future
        // ... other component APIs
    }
    
    impl SubmergeApi {
        // Selective exposure with additional logic
        pub async fn get_block_data(&self, block_id: BlockIdentifier) -> ApiResult<BlockData> {
            // Validation, auth, rate limiting
            self.validate_request(&block_id)?;
            
            // Transform internal result to public API format
            let traces = self.crystal_api.get_block_traces(&block_id.hash).await?;
            Ok(BlockData::from_traces(traces))
        }
        
        // Admin-only endpoints
        pub async fn admin_reprocess_block(&self, auth: AdminAuth, number: u64) -> ApiResult<()> {
            auth.verify()?;
            self.crystal_api.force_reprocess_block(number).await?;
            Ok(())
        }
    }
}
```

## Implementation Strategy

### Phase 1: Extract Crystal API

Create `src/crystal_api.rs`:

```rust
use std::sync::Arc;
use crate::persistence::CrystalPostgreSQLStorage;
use submerge_base::types::submerge::BlockTraces;
use submerge_persistence::postgres::PostgreSQLStorage;

pub struct CrystalApi {
    storage: Arc<PostgreSQLStorage>,
}

impl CrystalApi {
    pub fn new(storage: Arc<PostgreSQLStorage>) -> Self {
        Self { storage }
    }

    // Block query operations
    pub async fn get_block_traces_by_hash(&self, hash: &[u8]) -> CrystalResult<Option<BlockTraces>> {
        self.storage.get_block_traces_by_hash(hash)
            .await
            .map_err(CrystalError::Database)
    }

    pub async fn get_block_traces_by_number(&self, number: u64) -> CrystalResult<Vec<BlockTraces>> {
        self.storage.get_block_traces_by_number(number)
            .await
            .map_err(CrystalError::Database)
    }

    // Operational endpoints
    pub async fn get_ingestion_metrics(&self) -> CrystalResult<IngestionMetrics> {
        // Implementation to gather metrics
        todo!()
    }

    pub async fn get_error_summary(&self) -> CrystalResult<Vec<ProcessingError>> {
        // Query trace_error table
        todo!()
    }
    
    // Admin operations  
    pub async fn force_reprocess_range(&self, start: u64, end: u64) -> CrystalResult<()> {
        // Implementation for reprocessing blocks
        todo!()
    }

    pub async fn clear_error_log(&self) -> CrystalResult<()> {
        // Clear trace_error table
        todo!()
    }

    // Health check
    pub async fn health_check(&self) -> CrystalResult<HealthStatus> {
        // Check database connectivity, latest block status, etc.
        todo!()
    }
}

// Crystal-specific types
#[derive(Debug, Serialize)]
pub struct IngestionMetrics {
    pub latest_processed_block: u64,
    pub blocks_per_second: f64,
    pub total_blocks_processed: u64,
    pub error_count: u32,
    pub database_connection_status: String,
}

#[derive(Debug, Serialize)]
pub struct ProcessingError {
    pub block_number: u64,
    pub block_hash: String,
    pub error_message: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String, // "healthy", "degraded", "unhealthy"
    pub database_connected: bool,
    pub latest_block_age_seconds: u64,
    pub error_rate: f64,
}

// Crystal-specific errors
#[derive(Debug, thiserror::Error)]
pub enum CrystalError {
    #[error("Block not found: {hash}")]
    BlockNotFound { hash: String },
    #[error("Invalid block number: {number}")]
    InvalidBlockNumber { number: u64 },
    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),
    #[error("Processing error: {message}")]
    Processing { message: String },
}

pub type CrystalResult<T> = Result<T, CrystalError>;
```

### Phase 2: Create Gateway API

Create `src/gateway_api.rs`:

```rust
use std::sync::Arc;
use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use crate::crystal_api::{CrystalApi, CrystalError};

pub struct GatewayApi {
    crystal: Arc<CrystalApi>,
    auth: Arc<AuthService>,
    rate_limiter: Arc<RateLimiter>,
}

impl GatewayApi {
    pub fn new(
        crystal: Arc<CrystalApi>,
        auth: Arc<AuthService>,
        rate_limiter: Arc<RateLimiter>,
    ) -> Self {
        Self {
            crystal,
            auth,
            rate_limiter,
        }
    }

    // Public endpoints with validation
    pub async fn get_block(&self, request: GetBlockRequest) -> ApiResponse<BlockData> {
        // Input validation
        request.validate()?;
        
        // Rate limiting
        self.rate_limiter.check_limit(&request.client_id).await?;
        
        // Transform and forward
        match request.identifier {
            BlockIdentifier::Hash(hash) => {
                let hash_bytes = hex::decode(&hash)
                    .map_err(|_| ApiError::invalid_input("Invalid block hash format"))?;
                
                match self.crystal.get_block_traces_by_hash(&hash_bytes).await? {
                    Some(traces) => Ok(ApiResponse::success(BlockData::from_traces(traces))),
                    None => Err(ApiError::not_found("Block not found")),
                }
            }
            BlockIdentifier::Number(num) => {
                let traces = self.crystal.get_block_traces_by_number(num).await?;
                Ok(ApiResponse::success(BlockData::from_traces_list(traces)))
            }
        }
    }
    
    // Admin endpoints with auth
    pub async fn admin_get_metrics(&self, auth: AdminToken) -> ApiResponse<IngestionMetrics> {
        self.auth.verify_admin(&auth).await?;
        let metrics = self.crystal.get_ingestion_metrics().await?;
        Ok(ApiResponse::success(metrics))
    }

    pub async fn admin_get_errors(&self, auth: AdminToken) -> ApiResponse<Vec<ProcessingError>> {
        self.auth.verify_admin(&auth).await?;
        let errors = self.crystal.get_error_summary().await?;
        Ok(ApiResponse::success(errors))
    }

    pub async fn admin_reprocess_blocks(
        &self,
        auth: AdminToken,
        request: ReprocessRequest,
    ) -> ApiResponse<()> {
        self.auth.verify_admin(&auth).await?;
        request.validate()?;
        
        self.crystal
            .force_reprocess_range(request.start_block, request.end_block)
            .await?;
        
        Ok(ApiResponse::success(()))
    }

    // Health endpoint
    pub async fn health(&self) -> ApiResponse<HealthStatus> {
        let health = self.crystal.health_check().await?;
        Ok(ApiResponse::success(health))
    }
}
```

### Phase 3: Define Public API Types

Create `src/api_types.rs`:

```rust
use serde::{Deserialize, Serialize};
use crate::crystal_api::{IngestionMetrics, ProcessingError, HealthStatus};

// Public API request types (stable)
#[derive(Debug, Deserialize)]
pub struct GetBlockRequest {
    pub identifier: BlockIdentifier,
    pub include_traces: Option<bool>,
    pub client_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BlockIdentifier {
    Hash(String),
    Number(u64),
}

#[derive(Debug, Deserialize)]
pub struct ReprocessRequest {
    pub start_block: u64,
    pub end_block: u64,
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AdminToken {
    pub token: String,
}

// Public API response types (stable)
#[derive(Debug, Serialize)]
pub struct BlockData {
    pub hash: String,
    pub number: u64,
    pub timestamp: u64,
    pub parent_hash: String,
    pub runtime_version: u32,
    pub is_finalized: bool,
    pub traces: Option<Vec<TraceData>>,
    pub extrinsic_count: u32,
    pub event_count: u32,
}

#[derive(Debug, Serialize)]
pub struct TraceData {
    pub index: u32,
    pub key: String,
    pub value: String,
    pub ext_id: String,
    pub method: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

// Validation implementations
impl GetBlockRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.client_id.is_empty() {
            return Err(ApiError::invalid_input("client_id is required"));
        }
        
        match &self.identifier {
            BlockIdentifier::Hash(hash) => {
                if hash.len() != 66 || !hash.starts_with("0x") {
                    return Err(ApiError::invalid_input("Invalid block hash format"));
                }
            }
            BlockIdentifier::Number(_) => {} // Always valid
        }
        
        Ok(())
    }
}

impl ReprocessRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.start_block > self.end_block {
            return Err(ApiError::invalid_input("start_block must be <= end_block"));
        }
        
        if self.end_block - self.start_block > 10000 {
            return Err(ApiError::invalid_input("Cannot reprocess more than 10000 blocks at once"));
        }
        
        Ok(())
    }
}

// Conversion implementations
impl BlockData {
    pub fn from_traces(traces: crate::crystal_api::BlockTraces) -> Self {
        Self {
            hash: traces.block_hash,
            number: traces.block_number,
            timestamp: 0, // Would need to be added to BlockTraces
            parent_hash: traces.block_parent_hash,
            runtime_version: traces.runtime_version,
            is_finalized: traces.is_finalized,
            traces: Some(
                traces.traces
                    .into_iter()
                    .map(TraceData::from)
                    .collect()
            ),
            extrinsic_count: 0, // Would need to be added
            event_count: 0,     // Would need to be added
        }
    }

    pub fn from_traces_list(traces_list: Vec<crate::crystal_api::BlockTraces>) -> Self {
        // Implementation for multiple blocks
        todo!()
    }
}

impl From<crate::crystal_api::SubmergeBlockTrace> for TraceData {
    fn from(trace: crate::crystal_api::SubmergeBlockTrace) -> Self {
        Self {
            index: trace.index,
            key: trace.key,
            value: trace.value,
            ext_id: trace.ext_id,
            method: trace.method.to_string(),
            parent_id: trace.parent_id,
        }
    }
}

// Error conversion
impl From<CrystalError> for ApiError {
    fn from(err: CrystalError) -> Self {
        match err {
            CrystalError::BlockNotFound { hash } => ApiError {
                code: "BLOCK_NOT_FOUND".to_string(),
                message: format!("Block {} not found", hash),
                details: None,
            },
            CrystalError::InvalidBlockNumber { number } => ApiError {
                code: "INVALID_BLOCK_NUMBER".to_string(),
                message: format!("Invalid block number: {}", number),
                details: None,
            },
            CrystalError::Database(_) => ApiError {
                code: "INTERNAL_ERROR".to_string(),
                message: "An internal error occurred".to_string(),
                details: None, // Don't expose internal details
            },
            CrystalError::Processing { message } => ApiError {
                code: "PROCESSING_ERROR".to_string(),
                message,
                details: None,
            },
        }
    }
}

impl ApiError {
    pub fn invalid_input(message: &str) -> Self {
        Self {
            code: "INVALID_INPUT".to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn not_found(message: &str) -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn unauthorized(message: &str) -> Self {
        Self {
            code: "UNAUTHORIZED".to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn rate_limited(message: &str) -> Self {
        Self {
            code: "RATE_LIMITED".to_string(),
            message: message.to_string(),
            details: None,
        }
    }
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
        }
    }
}
```

### Phase 4: Update Actix-Web Integration

Update `src/api.rs`:

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Result as ActixResult};
use std::sync::Arc;
use crate::gateway_api::GatewayApi;
use crate::api_types::*;

pub async fn run_gateway_api(
    gateway: Arc<GatewayApi>,
    host: &str, 
    port: u16
) -> anyhow::Result<()> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(gateway.clone()))
            .service(
                web::scope("/api/v1")
                    .route("/blocks/{identifier}", web::get().to(get_block_handler))
                    .route("/health", web::get().to(health_handler))
                    .service(
                        web::scope("/admin")
                            .route("/metrics", web::get().to(admin_metrics_handler))
                            .route("/errors", web::get().to(admin_errors_handler))
                            .route("/reprocess", web::post().to(admin_reprocess_handler))
                    )
            )
    })
    .workers(10)
    .bind(format!("{}:{}", host, port))?
    .run();

    server.await?;
    Ok(())
}

async fn get_block_handler(
    path: web::Path<String>,
    query: web::Query<GetBlockQuery>,
    gateway: web::Data<Arc<GatewayApi>>,
) -> ActixResult<HttpResponse> {
    let identifier = parse_block_identifier(&path)?;
    
    let request = GetBlockRequest {
        identifier,
        include_traces: query.include_traces,
        client_id: query.client_id.clone().unwrap_or_default(),
    };

    match gateway.get_block(request).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(error))),
    }
}

async fn health_handler(
    gateway: web::Data<Arc<GatewayApi>>,
) -> ActixResult<HttpResponse> {
    match gateway.health().await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()>::error(error))),
    }
}

// Additional handlers...
```

## File Structure

```
src/
├── lib.rs                 # Main Crystal service (block ingestion)
├── crystal_api.rs         # Crystal component API
├── gateway_api.rs         # Public gateway API
├── api_types.rs          # Public API request/response types
├── api.rs                # Actix-web HTTP handlers
├── persistence.rs        # Database operations
├── args.rs              # CLI arguments
└── metrics.rs           # Metrics collection
```

## API Endpoints

### Public Endpoints
- `GET /api/v1/blocks/{hash_or_number}` - Get block data
- `GET /api/v1/health` - Service health check

### Admin Endpoints (require authentication)
- `GET /api/v1/admin/metrics` - Ingestion metrics
- `GET /api/v1/admin/errors` - Processing error summary
- `POST /api/v1/admin/reprocess` - Force reprocess block range

## Advantages of This Architecture

### ✅ **Scalability**
- Components can be extracted to separate services later
- Gateway becomes load balancer/API gateway
- Natural microservices evolution path

### ✅ **Security**
- Single point of authentication/authorization
- Input validation and sanitization
- Prevent direct access to internal operations

### ✅ **Monitoring**
- Centralized request logging and metrics
- Component-level performance tracking
- Clear observability boundaries

### ✅ **API Evolution**
- Stable public API contract
- Internal APIs can change without breaking clients
- Easy to add new components without changing gateway structure

## Migration Steps

1. **Phase 1**: Extract Crystal API from existing code
2. **Phase 2**: Create gateway API with basic endpoints
3. **Phase 3**: Add authentication and rate limiting
4. **Phase 4**: Migrate existing HTTP handlers to use gateway
5. **Phase 5**: Add admin endpoints and monitoring
6. **Phase 6**: Extract other component APIs (Cortex, etc.)

## Testing Strategy

### Unit Tests
- Test each component API independently
- Mock dependencies for isolation
- Test error handling and edge cases

### Integration Tests
- Test gateway API with real component APIs
- Test authentication and authorization
- Test rate limiting and validation

### Performance Tests
- Load test public endpoints
- Test component API performance
- Validate caching and optimization

This architecture provides a solid foundation for scaling the Submerge platform while maintaining clean separation of concerns and API stability.