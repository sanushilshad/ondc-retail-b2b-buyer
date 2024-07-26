#!/bin/bash

pkill -f '^target/debug/ondc-retail-b2b-buyer'
cargo run --bin ondc-retail-b2b-buyer