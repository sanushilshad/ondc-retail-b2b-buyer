#!/bin/bash

# pkill -U dev2
source env.sh
echo "User $USER"
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
