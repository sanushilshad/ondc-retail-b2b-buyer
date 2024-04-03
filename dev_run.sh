#!/bin/bash

PIDS=($(ps aux | grep 'dev2' | grep 'rust_test' | awk '{print $2}'))

if [ ${#PIDS[@]} -gt 0 ];then
    echo "Running Ports are ${PIDS}"
    for i in "${PIDS[@]}"
        do
           echo "Killing process with PID - $i"
           kill -9 $i
           PID_KILLED=1
        done
fi
source env.sh
cargo-watch -qc -w src -x "run --bin rust_test" -x clippy
# cargo watch -x "run --bin rust_test"
# cargo-watch -qc -w src -x "run --bin rust_test"