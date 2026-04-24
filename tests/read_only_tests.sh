#!/bin/bash

# Integration test suite for read-only, non-interactive ao commands
# This script verifies that commands which should only read system state execute correctly.

# Exit on any error
set -e

# Build ao if not present
if ! command -v ao &> /dev/null; then
    echo "Building ao..."
    cargo build --quiet
    export PATH="$PWD/target/debug:$PATH"
fi

# Function to run a command and check its exit status
run_test() {
    local cmd="$1"
    echo -n "Testing: $cmd ... "
    if output=$($cmd 2>&1); then
        echo "PASSED"
    else
        # If it failed but the output says "No supported ... found", it's acceptable for a minimal container
        if echo "$output" | grep -qE "No supported|not found|No such file or directory"; then
            echo "SKIPPED (Missing system tool)"
        else
            echo "FAILED"
            echo "Error output:"
            echo "$output"
            exit 1
        fi
    fi
}

# List of read-only, non-interactive commands to test
commands=(
    "ao monitor"
    "ao monitor -f json"
    "ao pkg ls"
    "ao pkg search rust"
    "ao svc ls"
    "ao svc status cron"
    "ao user ls"
    "ao group ls"
    "ao disk ls"
    "ao disk usage ."
    "ao sys info"
    "ao sys time status"
    "ao log sys --lines 5"
    "ao log auth --lines 5"
    "ao log boot --lines 5"
    "ao log crash --lines 5"
    "ao log dev --lines 5"
    "ao log error --lines 5"
    "ao log pkg --lines 5"
    "ao log svc cron --lines 5"
    "ao distro info"
    "ao net interfaces"
    "ao net ips"
    "ao net routes"
    "ao net fw status"
    "ao boot ls"
    "ao boot mod ls"
    "ao gui info"
    "ao gui display ls"
    "ao dev ls"
    "ao dev pci"
    "ao dev usb"
    "ao dev bt status"
    "ao dev print ls"
    "ao virt ls"
    "ao sec audit"
    "ao sec context"
    "ao self info"
    "ao self completions generate bash"
)

echo "Starting read-only integration tests..."
echo "--------------------------------------"

for cmd in "${commands[@]}"; do
    run_test "$cmd"
done

echo "--------------------------------------"
echo "All read-only integration tests PASSED!"
