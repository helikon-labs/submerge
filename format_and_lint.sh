#!/usr/bin/env bash
vacuum lint --ignore-file ./vacuum-ignore.yaml -d ./submerge-crystal/api-spec/submerge-crystal-api.json
cargo +nightly fmt
cargo +nightly clippy --all-targets -- -D warnings -W clippy::too_many_lines -W clippy::excessive_nesting