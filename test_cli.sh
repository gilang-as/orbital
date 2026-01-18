#!/bin/bash
# Test CLI in QEMU - send commands via serial

cd /Volumes/Works/Projects/orbital

# Start QEMU in background with serial output to stdout
# Use -serial mon:stdio to get both monitor and serial, then -serial pty for additional serial
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-orbital/debug/bootimage-orbital.bin \
  -m 256 \
  -cpu qemu64 \
  -nographic \
  -serial stdio \
  2>&1 &

QEMU_PID=$!

# Give QEMU time to boot
sleep 3

# Send test commands
{
  sleep 1
  echo "help"
  sleep 1
  echo "ping"
  sleep 1
  echo "uptime"
  sleep 1
  echo "ps"
  sleep 1
  echo "pid"
  sleep 1
  echo "exit"
  sleep 1
} | tee /tmp/commands.txt

# Kill QEMU
kill $QEMU_PID 2>/dev/null
wait $QEMU_PID 2>/dev/null

echo "Test complete"
