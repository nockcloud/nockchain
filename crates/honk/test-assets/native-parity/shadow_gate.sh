#!/usr/bin/env bash
# Fast validation gate for the native-types construction port (the _n shadow).
#
# For each small fixture, compiles with and without HONK_NATIVE_TYPES and checks:
#   (1) no native-shadow assert trips (assert_native_eq panic) under the flag, and
#   (2) the output jam is byte-identical with and without the flag (additive).
#
# These fixtures are tiny, so even the O(n^2) not-yet-threaded native_of fallback
# runs in ~1-2s each. This is the routine gate for each cascade increment —
# full-kernel flag-on runs are O(n^2) during the migration and are NOT used here.
#
# Usage: crates/honk/test-assets/native-parity/shadow_gate.sh [honk_binary]
set -u
HONK="${1:-target/release/honk}"
PRELUDE="hoon/common/hoon.hoon"
DIR="crates/honk/test-assets/native-parity/exprs"
FIXTURES="core_chain fork loop_dec wet_turn"
TMP="${TMPDIR:-/tmp}"
ok=1
for f in $FIXTURES; do
  src="$DIR/$f.hoon"
  off="$TMP/sg_off_$f.jam"
  on="$TMP/sg_on_$f.jam"
  log="$TMP/sg_on_$f.log"
  timeout 60 "$HONK" --new --arbitrary --output "$off" --prelude "$PRELUDE" "$src" hoon >/dev/null 2>&1
  timeout 120 env HONK_NATIVE_TYPES=1 "$HONK" --new --arbitrary --output "$on" --prelude "$PRELUDE" "$src" hoon > "$log" 2>&1
  if grep -qiE "panic|native shadow mismatch" "$log"; then
    echo "  $f: ASSERT TRIP"; ok=0
  elif cmp -s "$off" "$on"; then
    echo "  $f: byte-identical OK"
  else
    echo "  $f: DIFFER"; ok=0
  fi
done
if [ "$ok" = 1 ]; then echo "native-shadow gate: PASS"; exit 0; else echo "native-shadow gate: FAIL"; exit 1; fi
