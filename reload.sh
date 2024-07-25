#!/bin/bash

pkill -f '^target/debug/ondc_retail_b2b_buyer'
cargo run --bin ondc_retail_b2b_buyer