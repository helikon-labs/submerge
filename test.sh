#!/usr/bin/env bash
./_scripts/reset-crystal-test-db.sh
cargo test -- --nocapture