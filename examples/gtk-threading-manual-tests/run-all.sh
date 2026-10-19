#!/usr/bin/env bash
# Runs all threading test cases one after another, and prints a summary at the end.
#
# Usage: ./run-all.sh [gtk4|gtk3]   (default: gtk4)
#
# Env:
#   TIMEOUT=120   seconds after which a case is considered hung
#   EXIT_RUNS=3   how many times to run the racy `exit_after_dialog` case

set -u

FEATURE="${1:-gtk4}"
TIMEOUT="${TIMEOUT:-60}"
EXIT_RUNS="${EXIT_RUNS:-3}"

cd "$(dirname "$0")"

case "$FEATURE" in
gtk3 | gtk4) ;;
*)
    echo "Unknown feature '$FEATURE', expected one of: gtk4, gtk3" >&2
    exit 1
    ;;
esac

echo "Building with --features $FEATURE"
cargo build --features "$FEATURE" --bins || exit 1

BIN_DIR="target/debug"

CASES=(
    main_sync
    main_async_block_on
    async_awaited_elsewhere
    worker_sync
    parallel_sync
    parallel_async
    sequential_mixed
    dropped_future
)
for _ in $(seq "$EXIT_RUNS"); do
    CASES+=(exit_after_dialog)
done

CASES+=(
    gtk_callback_sync
    gtk_callback_async
    gtk_worker_sync
    gtk_nested_sync
    gtk_init_c
    "gtk_before_loop"
    "gtk_before_loop --c"
    gtk_off_main_thread
)

RESULTS=()
FAILED=0

for case in "${CASES[@]}"; do
    echo
    echo "=== $case ==="
    # shellcheck disable=SC2086 # word splitting is intended, to pass args
    timeout "$TIMEOUT" "$BIN_DIR"/$case
    status=$?

    if [ "$status" -eq 0 ]; then
        result="ok"
    elif [ "$status" -eq 124 ]; then
        result="HANG (timed out after ${TIMEOUT}s)"
    elif [ "$status" -gt 128 ]; then
        result="CRASH (signal $((status - 128)))"
    else
        result="FAIL (exit code $status)"
    fi

    [ "$result" = "ok" ] || FAILED=$((FAILED + 1))
    RESULTS+=("$(printf '%-24s %s' "$case" "$result")")
done

echo
echo "=== Summary ($FEATURE) ==="
printf '%s\n' "${RESULTS[@]}"
echo
echo "$FAILED of ${#CASES[@]} runs failed"
[ "$FAILED" -eq 0 ]
