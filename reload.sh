#!/bin/bash

source env.sh
executable_name="$APPLICATION__NAME" 
directory="./target/release"
PIDS=($(ps aux | grep "$APPLICATION__ACCOUNT_NAME" | grep "$APPLICATION__NAME" | grep -v restart | awk '{print $2}'))

if [ ${#PIDS[@]} -gt 0 ];then
    echo "Running Ports are ${PIDS}"
    for i in "${PIDS[@]}"
        do
           echo "Killing process with PID - $i"
           kill -9 $i
           PID_KILLED=1
        done
fi
nohup "$directory/$executable_name" &