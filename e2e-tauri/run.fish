#!/usr/bin/env fish
# Run WebDriver E2E tests against the real Tauri app.
# Requires: tauri-driver (cargo install tauri-driver), built app

set -l TAURI_DRIVER_PID ""

function cleanup
    if test -n "$TAURI_DRIVER_PID"
        kill $TAURI_DRIVER_PID 2>/dev/null
        echo "Stopped tauri-driver (PID: $TAURI_DRIVER_PID)"
    end
end

trap cleanup EXIT

# Start tauri-driver in the background
echo "Starting tauri-driver..."
tauri-driver &
set TAURI_DRIVER_PID $last_pid

# Give it a moment to start
sleep 2

# Run WebDriverIO tests
echo "Running WebDriver E2E tests..."
bunx wdio run wdio.conf.ts
set -l EXIT_CODE $status

cleanup
exit $EXIT_CODE
