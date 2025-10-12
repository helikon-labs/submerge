#!/usr/bin/env bash
vacuum lint -d ./submerge-crystal/api-spec/submerge-crystal-api.yaml 
cargo +nightly fmt
cargo +nightly clippy --all-targets -- -D warnings -W clippy::cognitive_complexity