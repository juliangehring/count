#!/usr/bin/env bash

set -ex

# Incorporate TARGET env var to the build and test process
cargo build --target "$TARGET"

cargo test --target "$TARGET"

cargo run --target "$TARGET" -- --top 3 tests/data/toplevel-1k.txt
