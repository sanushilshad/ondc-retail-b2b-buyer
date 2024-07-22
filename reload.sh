#!/bin/bash

pkill -f '^target/debug/ondc_b2b_buyer'
cargo run --bin ondc_b2b_buyer