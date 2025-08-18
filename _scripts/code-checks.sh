#!/usr/bin/env bash
cd "${0%/*}" || exit # cd script directory
cd ..
cargo +nightly-2025-08-01 fmt
vacuum lint -d ./submerge-crystal/api-spec/submerge-crystal-api.yaml
cargo +nightly-2025-08-01 clippy --all-targets -- -D warnings -W clippy::cognitive_complexity