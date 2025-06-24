# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Submerge is an open-source indexer, analysis, KYT, and AML compliance platform for the Polkadot ecosystem. Built by Helikon Labs and led by Kutsal Kaan Bilgin, it's a comprehensive Rust-based blockchain data platform addressing critical gaps in Polkadot's data infrastructure.

### Project Scope & Goals
- **Primary Mission**: Create complete data infrastructure for Polkadot ecosystem with indexing, analysis, and compliance capabilities
- **Supported Networks**: 21 blockchain networks across Polkadot and Kusama ecosystems (including Astar, Acala, and system parachains)
- **Business Model**: "Stake to Access" (S2A) - free core application for DOT holders, premium features based on DOT stake
- **Timeline**: 2-milestone project over 6 months, started December 2024, alpha release scheduled July-August 2025

### Multi-Component Architecture
**Submerge Crystal**: High-performance indexer for blockchain data ingestion
**Submerge Fractal**: Data analysis and processing component  
**Submerge Bloom**: Real-time monitoring and alerting (formerly Sentinel)
**Submerge Mycelium**: API and integration layer

### Compliance & Security Integration
- **KYT/AML Integration**: Merkle Science (completed), Chainalysis, Scorechain, OFAC APIs (in progress)
- **Real-time Monitoring**: Sanctions screening and risk assessment
- **Compliance Features**: Transaction tracking, address clustering, risk scoring

### Infrastructure Specifications
- **Storage Requirements**: 123.4TB NVMe storage across two Supermicro servers
- **Processing Power**: 128 cores/256 threads, 2TB DDR5 RAM
- **Location**: Physical infrastructure in İstanbul
- **Database**: PostgreSQL with advanced partitioning (500k block ranges)

### Funding & Development
- **Total Budget**: $180,000 USDT (Milestone 1)
- **Team Size**: 6 full-time equivalent developers
- **Open Source**: All components will be fully open-sourced with comprehensive documentation

## Development Commands

### Essential Commands
- `./format.sh` - Format all Rust code using rustfmt
- `./clippy.sh` - Run strict linting with custom clippy configuration
- `./test.sh` - Run full test suite with automatic database reset
- `cargo build --release` - Build optimized release binaries
- `cargo run --bin submerge-crystal` - Run primary data ingestion service

### Database Management
- `./scripts/reset-test-db.sh` - Reset test database to clean state
- `./scripts/reset-dev-db.sh` - Reset development database
- `psql -d submerge_test -f migrations/schema.sql` - Apply database schema

### Network Operations
- `cargo run --bin submerge-crystal -- --network polkadot` - Index Polkadot
- `cargo run --bin submerge-crystal -- --network kusama` - Index Kusama
- `cargo run --bin submerge-crystal -- --network westend` - Index Westend testnet

## Architecture Overview

### Monorepo Structure
The project is organized as a Rust workspace with 21 specialized crates:

**Core Services:**
- `submerge-crystal` - Primary blockchain data ingestion and processing service
- `submerge-warp` - HTTP API server for querying indexed data
- `submerge-server` - Additional HTTP services and endpoints

**Data Layer:**
- `submerge-database` - PostgreSQL interactions with advanced partitioning (500k block ranges)
- `submerge-models` - Database models and data structures
- `submerge-migrations` - SQL schema migrations with timestamps

**Substrate Integration:**
- `submerge-substrate` - Substrate client wrapper and blockchain interactions
- `submerge-chainspecs` - Network configuration and chain specifications
- `submerge-types` - Substrate-specific type definitions and codecs

**Utilities:**
- `submerge-supervisor` - Service management, monitoring, and crash recovery
- `submerge-metrics` - Prometheus metrics collection and reporting
- `submerge-utils` - Shared utilities and helper functions

### Key Architectural Patterns

**BaseService Trait Pattern:**
All services implement `BaseService` with standardized lifecycle management (start, stop, health checks).

**Database Partitioning Strategy:**
PostgreSQL tables are partitioned by block ranges (500k blocks per partition) for optimal query performance and maintenance.

**Supervisor Pattern:**
Services are managed by a supervisor that handles crashes, restarts, and health monitoring.

**Configuration-Driven Networks:**
Network support is added via chainspec JSON files in `chainspecs/` directory rather than hardcoded logic.

## Development Guidelines

### Code Quality Standards
- Requires **nightly Rust 1.83.0+** toolchain
- Strict clippy configuration with cognitive complexity warnings
- **HashMap/HashSet are disallowed** - use `rustc-hash::FxHashMap/FxHashSet` instead
- All services must implement proper async patterns with Tokio
- Database interactions must use the partitioning-aware query patterns

### Testing Approach
- Integration tests require PostgreSQL test database
- Use `./test.sh` which automatically resets test database
- Test database name: `submerge_test`
- Database tests run in transactions that are rolled back
- Network-specific tests use Westend testnet data

### Common Patterns for New Features

**Adding a New Service:**
1. Create new crate in workspace
2. Implement `BaseService` trait
3. Add to supervisor configuration
4. Include Prometheus metrics integration
5. Add database migration if needed

**Adding Network Support:**
1. Add chainspec JSON file to `chainspecs/`
2. Update network enum in `submerge-chainspecs`
3. Add network-specific configuration
4. Test with `--network <new_network>` flag

**Database Schema Changes:**
1. Create timestamped SQL file in `migrations/`
2. Test with both dev and test databases
3. Ensure partitioning compatibility
4. Update relevant model structs

### Configuration Notes
- Uses `rustc-hash` for all hash maps/sets (faster than std HashMap)
- Database connections use connection pooling via `sqlx`
- HTTP servers use Actix-web framework with JSON responses
- All async code uses Tokio runtime
- Logging via `tracing` crate with structured output
- Metrics exported on `/metrics` endpoint for Prometheus scraping

## Troubleshooting

### Common Issues
- **Build failures:** Ensure nightly Rust 1.83.0+ is installed
- **Database connection errors:** Check PostgreSQL is running and `submerge_test` database exists
- **Test failures:** Run `./scripts/reset-test-db.sh` to clean test database state
- **Performance issues:** Check if database partitions are properly created for block ranges

### Database Partition Management
Partitions are automatically created for 500k block ranges. For new networks, ensure partition creation logic accounts for the network's block numbering scheme.