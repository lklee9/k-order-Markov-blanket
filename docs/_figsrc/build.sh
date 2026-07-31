#!/bin/sh
# Build the project page's figures: docs/_figsrc/*.typ -> docs/assets/fig/*.svg
#
# Run from anywhere:  sh docs/_figsrc/build.sh
#
# The outputs are COMMITTED. Do not build these in CI: the SVGs embed Frutiger
# glyph outlines, and a machine without Frutiger installed produces a file that
# looks plausible but is metrically wrong — Typst treats a missing font as a
# warning and still exits 0. Hence the stderr grep below, which turns that
# warning into a hard failure.
set -e
cd "$(dirname "$0")"
out=../assets/fig
mkdir -p "$out"

for src in collider.typ rung-k0.typ rung-k1.typ rung-k2.typ bench.typ; do
    name=$(basename "$src" .typ)
    err=$(typst compile "$src" "$out/$name.svg" 2>&1 >/dev/null) || {
        printf 'FAILED: %s\n%s\n' "$src" "$err" >&2
        exit 1
    }
    if printf '%s' "$err" | grep -qi 'unknown font family'; then
        printf 'FAILED: %s substituted a missing font:\n%s\n' "$src" "$err" >&2
        exit 1
    fi
    [ -n "$err" ] && printf '%s: %s\n' "$name" "$err" >&2
    printf '%-14s %6s bytes\n' "$name.svg" "$(wc -c < "$out/$name.svg" | tr -d ' ')"
done
