#!/usr/bin/env bash

set -x

cargo clippy --all-features --all-targets
cargo test --features "test"

exit
