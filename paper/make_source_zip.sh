#!/usr/bin/env bash
#
# make_source_zip.sh — bundle the LaTeX source needed to compile the paper
# into one zip, suitable for an arXiv / camera-ready source submission.
#
# How it decides what to include: it compiles main.tex once so that latexmk's
# input record (main.fls) and the bibliography (main.bbl) are current, then
# packs EXACTLY the files that the build read from this directory. That means
# only the figures/tables actually used are included (stale assets such as the
# unused fig/syn_metrics/*.png previews are left out automatically), and it
# stays correct if figures/inputs are added or removed later. The .bbl and the
# .bib source are added explicitly (arXiv does not run BibTeX), while build
# artefacts (.aux, .log, .fls, …) and the output main.pdf are excluded.
#
# By default it then VERIFIES the bundle by extracting and compiling it in a
# clean temporary directory, so a missing file fails loudly instead of at
# submission time.
#
# Usage:
#   bash make_source_zip.sh [output.zip]    # default output: main-source.zip
#   SKIP_VERIFY=1 bash make_source_zip.sh   # build the zip but skip the test compile
#
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"          # run from the paper/ directory

JOB=main
OUT="${1:-${JOB}-source.zip}"
case "$OUT" in /*) ;; *) OUT="$PWD/$OUT" ;; esac   # make the output path absolute

command -v latexmk >/dev/null || { echo "ERROR: latexmk not found in PATH." >&2; exit 1; }
command -v zip     >/dev/null || { echo "ERROR: zip not found in PATH."     >&2; exit 1; }
command -v unzip   >/dev/null || { echo "ERROR: unzip not found in PATH."   >&2; exit 1; }

echo "==> Compiling $JOB.tex to refresh $JOB.fls and $JOB.bbl ..."
latexmk -pdf -interaction=nonstopmode -halt-on-error "$JOB.tex" >/dev/null

echo "==> Selecting source files from $JOB.fls ..."
LIST="$(mktemp)"
{
  grep '^INPUT ' "$JOB.fls" | sed -e 's/^INPUT //' -e 's#^\./##'   # everything the build read
  printf '%s\n' "$JOB.bbl"                                          # compiled bibliography (for arXiv)
  ls -1 ./*.bib 2>/dev/null | sed 's#^\./##' || true               # .bib source(s)
} | sort -u | while IFS= read -r f; do
  case "$f" in /*) continue ;; esac          # skip TeX Live / system files (absolute paths)
  [ -f "$f" ] || continue                    # must exist in this tree
  case "$f" in                               # skip build artefacts and the output PDF
    *.aux|*.fls|*.fdb_latexmk|*.log|*.out|*.blg|*.bcf|*.run.xml| \
    *.toc|*.lof|*.lot|*.synctex.gz|*.nav|*.snm|*.vrb|"$JOB.pdf") continue ;;
  esac
  printf '%s\n' "$f"
done | sort -u > "$LIST"

COUNT="$(wc -l < "$LIST" | tr -d ' ')"
echo "==> Including $COUNT files:"
sed 's/^/      /' "$LIST"

rm -f "$OUT"
zip -q "$OUT" -@ < "$LIST"
rm -f "$LIST"
echo "==> Wrote $OUT ($(du -h "$OUT" | cut -f1))"

# ---- Verify the bundle compiles on its own --------------------------------
if [ "${SKIP_VERIFY:-}" = "1" ]; then
  echo "==> SKIP_VERIFY=1 set; not test-compiling the bundle."
  exit 0
fi
echo "==> Verifying: extracting + compiling the bundle in a clean temp dir ..."
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
( cd "$TMP" && unzip -q "$OUT" && latexmk -pdf -interaction=nonstopmode "$JOB.tex" >build.log 2>&1 ) || true
if [ -f "$TMP/$JOB.pdf" ] && ! grep -qiE "^!|Emergency stop|Fatal error" "$TMP/build.log"; then
  pages="$(grep -aoE "\([0-9]+ pages" "$TMP/build.log" | grep -oE "[0-9]+" | tail -1)"
  echo "==> OK: the bundle compiles standalone (${pages:-?} pages). Ready to submit: $OUT"
else
  echo "ERROR: the bundle did NOT compile cleanly — a source file is likely missing." >&2
  cp "$TMP/build.log" "$PWD/zip-verify-fail.log" 2>/dev/null || true
  echo "       Saved the failing log to $PWD/zip-verify-fail.log" >&2
  exit 1
fi
