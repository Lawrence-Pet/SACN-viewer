#!/bin/bash

# Test sACN Sender Script
echo "Compiling and running sACN test sender..."
echo "This will send test packets to universe 1"
echo "Press Ctrl+C to stop"
echo ""

# Compile the test sender
echo "Compiling test sender..."
rustc test_sender.rs -o test_sender

if [ $? -eq 0 ]; then
    echo "Running test sender..."
    ./test_sender
else
    echo "Failed to compile test sender"
    exit 1
fi
