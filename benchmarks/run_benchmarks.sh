#!/bin/bash
set -e
cd "$(dirname "$0")/.."

AETHER="./target/release/aether"

echo "═══════════════════════════════════════════════════"
echo "  Aether vs Python Benchmark Suite"
echo "═══════════════════════════════════════════════════"
echo ""

# Ensure release build
cargo build --release 2>/dev/null

time_cmd() {
    local start end elapsed
    start=$(date +%s%N)
    eval "$1" > /dev/null 2>&1
    end=$(date +%s%N)
    elapsed=$(( (end - start) / 1000000 ))
    echo "$elapsed"
}

run_bench() {
    local name="$1"
    local ae_file="$2"
    local py_file="$3"

    printf "%-20s" "$name"

    # Aether interpreter
    ae_ms=$(time_cmd "$AETHER run $ae_file")

    # Python
    py_ms=$(time_cmd "python3 $py_file")

    # Ratio
    if [ "$py_ms" -gt 0 ]; then
        ratio=$(echo "scale=1; $ae_ms * 100 / $py_ms / 100" | bc 2>/dev/null || echo "?")
    else
        ratio="?"
    fi

    printf "Aether: %6dms  Python: %6dms  Ratio: %sx\n" "$ae_ms" "$py_ms" "$ratio"
}

echo "Benchmark            Aether (interp)    Python 3          Ratio"
echo "──────────────────── ───────────────── ───────────────── ──────"

run_bench "Fibonacci(30)" "benchmarks/fib.ae" "benchmarks/fib.py"
run_bench "Loop 1M" "benchmarks/loop.ae" "benchmarks/loop.py"
run_bench "String ops 10K" "benchmarks/string.ae" "benchmarks/string.py"
run_bench "List ops 50K" "benchmarks/list_ops.ae" "benchmarks/list_ops.py"
run_bench "Class ops 10K" "benchmarks/class_ops.ae" "benchmarks/class_ops.py"

echo ""
echo "── Parallel Speedup ──"
$AETHER run benchmarks/parallel.ae

echo ""
echo "── Startup Time ──"
echo 'print("")' > /tmp/aether_empty.ae
ae_start=$(time_cmd "$AETHER run /tmp/aether_empty.ae")
py_start=$(time_cmd "python3 -c 'print(\"\")'")
echo "  Aether startup: ${ae_start}ms"
echo "  Python startup: ${py_start}ms"

echo ""
echo "── Binary Size ──"
ls -lh target/release/aether | awk '{print "  Aether binary: " $5}'

echo ""
echo "── Memory Usage ──"
ae_mem=$(/usr/bin/time -v $AETHER run /tmp/aether_empty.ae 2>&1 | grep "Maximum resident" | awk '{print $NF}')
py_mem=$(/usr/bin/time -v python3 -c 'print("")' 2>&1 | grep "Maximum resident" | awk '{print $NF}')
echo "  Aether: ${ae_mem}KB"
echo "  Python: ${py_mem}KB"

rm -f /tmp/aether_empty.ae
echo ""
echo "Done."
