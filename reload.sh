#!/bin/bash

pkill -f '^target/debug/rust_test'
cargo run --bin rust_test