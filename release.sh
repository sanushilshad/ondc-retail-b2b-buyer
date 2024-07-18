#!/bin/bash
source env.sh
executable_name="$APPLICATION__NAME" 
directory="./target/release"
PIDS=($(ps aux | grep "$APPLICATION__ACCOUNT_NAME" | grep "$APPLICATION__NAME" | awk '{print $2}'))

if [ ${#PIDS[@]} -gt 0 ];then
    echo "Running Ports are ${PIDS}"
    for i in "${PIDS[@]}"
        do
           echo "Killing process with PID - $i"
           kill -9 $i
           PID_KILLED=1
        done
fi

# run the test cases
if cargo test; then
    echo "Tests passed."
else
    echo "Tests failed." >&2
    exit 1
fi

# clean
if cargo clean; then
    echo "Clean successful."
else
    echo "Clean failed." >&2
    exit 1
fi

if cargo build --bin "$executable_name" --release; then
    echo "Build successful."
else
    echo "Build failed." >&2
    exit 1
fi

# Clean up the target directory
if [ -d "$directory" ]; then
    find "$directory" -mindepth 1 ! -name "$executable_name" -exec rm -rf {} +
    echo "Successfully cleaned up the build directory."
else
    echo "Directory $directory does not exist." >&2
    exit 1
fi

echo "Successfully executed build script."
"$directory/$executable_name"

