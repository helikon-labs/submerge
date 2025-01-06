#!/usr/bin/env bash
cargo +nightly clippy --all-targets -- -D warnings -W clippy::cognitive_complexity