#!/usr/bin/env python3
"""Fetch the real-world benchmark datasets and regenerate the synthetic ones
used by the High-Order Markov Blanket experiments.

Real-world data come from the MMHC supplement (Tsamardinos, Brown & Aliferis,
"The max-min hill-climbing Bayesian network structure learning algorithm",
Machine Learning, 2006):

    https://pages.mtu.edu/~lebrown/supplements/mmhc_paper/mmhc_index.html

Each ``<stem>_data.zip`` archive holds the 10 sampled datasets at sizes
500/1000/5000 plus ``<Name>_graph.txt`` (the true network's adjacency matrix).
The archives are *not* redistributed in this repository; this script downloads
them into ``experiments/data/<Name>/``, matching the layout the runners and
``analyse.py`` expect (``DATA / name / f"{name}_s{size}_v{ver}.txt"``).

Synthetic data are produced locally by ``data_gen.py``.

Usage:
    python get_data.py                      # paper datasets + synthetic
    python get_data.py --all                # all 11 real-world networks + synthetic
    python get_data.py --datasets Alarm1 Insurance
    python get_data.py --no-synthetic       # only download real-world data
    python get_data.py --no-real            # only (re)generate synthetic data
    python get_data.py --force              # re-download even if already present
"""
from __future__ import annotations

import argparse
import io
import subprocess
import sys
import urllib.request
import zipfile
from pathlib import Path

BASE_URL = "https://pages.mtu.edu/~lebrown/supplements/mmhc_paper"
HERE = Path(__file__).resolve().parent
DATA = HERE / "data"

# Local dataset directory -> remote zip stem. The archives' internal files are
# already named "<LocalName>_s*_v*.txt" / "<LocalName>_graph.txt" (verified for
# the paper set); for any mismatch the prefix is normalised on extraction.
PAPER_DATASETS = {
    "Alarm1": "alarm",
    "Barley": "barley",
    "Insurance": "ins",
    "Mildew": "mildew",
}
EXTRA_DATASETS = {
    "Child": "child",
    "Child10": "child10",
    "Gene": "gene",
    "HailFinder": "hailf",
    "Link": "link",
    "Munin1": "munin",
    "Pigs": "pigs",
}
CATALOGUE = {**PAPER_DATASETS, **EXTRA_DATASETS}


def _download(url: str, timeout: int = 120) -> bytes:
    with urllib.request.urlopen(url, timeout=timeout) as resp:  # noqa: S310 (fixed public host)
        return resp.read()


def fetch_real(name: str, stem: str, *, force: bool) -> None:
    """Download <stem>_data.zip and extract it into experiments/data/<name>/."""
    dest = DATA / name
    if (dest / f"{name}_graph.txt").exists() and not force:
        print(f"  [skip] {name}: already present")
        return
    url = f"{BASE_URL}/{stem}_data.zip"
    print(f"  [get ] {name}: {url}")
    with zipfile.ZipFile(io.BytesIO(_download(url))) as zf:
        members = [m for m in zf.namelist() if m.endswith(".txt")]
        # Detect the archive's internal prefix from its *_graph.txt member so we
        # can normalise it to the local dataset name (handles e.g. casing).
        graphs = [m for m in members if m.endswith("_graph.txt")]
        internal = Path(graphs[0]).name[: -len("_graph.txt")] if graphs else name
        dest.mkdir(parents=True, exist_ok=True)
        for m in members:
            fname = Path(m).name
            if internal and internal != name and fname.startswith(internal):
                fname = name + fname[len(internal):]
            (dest / fname).write_bytes(zf.read(m))
    n_samples = len(list(dest.glob(f"{name}_s*_v*.txt")))
    print(f"         -> {dest}  ({n_samples} sample files + graph)")


def generate_synthetic() -> None:
    """Run data_gen.py to (re)create the synthetic datasets under data/<func>/."""
    print("Generating synthetic datasets via data_gen.py ...")
    subprocess.run([sys.executable, "data_gen.py"], cwd=HERE, check=True)


def main(argv=None) -> int:
    p = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    p.add_argument(
        "--datasets", nargs="+", metavar="NAME",
        help=f"real-world datasets to fetch (choices: {', '.join(sorted(CATALOGUE))})",
    )
    p.add_argument("--all", action="store_true", help="fetch all real-world networks")
    p.add_argument("--no-synthetic", action="store_true", help="skip synthetic generation")
    p.add_argument("--no-real", action="store_true", help="skip real-world download")
    p.add_argument("--force", action="store_true", help="re-download even if present")
    args = p.parse_args(argv)

    if args.datasets:
        unknown = [d for d in args.datasets if d not in CATALOGUE]
        if unknown:
            p.error(f"unknown dataset(s): {unknown}; choices: {sorted(CATALOGUE)}")
        wanted = {d: CATALOGUE[d] for d in args.datasets}
    elif args.all:
        wanted = CATALOGUE
    else:
        wanted = PAPER_DATASETS

    failures = []
    if not args.no_real:
        print(f"Fetching {len(wanted)} real-world dataset(s) into {DATA} ...")
        for name, stem in wanted.items():
            try:
                fetch_real(name, stem, force=args.force)
            except Exception as exc:  # keep going on a single dataset failure
                print(f"  [FAIL] {name}: {exc}", file=sys.stderr)
                failures.append(name)

    if not args.no_synthetic:
        generate_synthetic()

    if failures:
        print(f"Done with {len(failures)} failure(s): {failures}", file=sys.stderr)
        return 1
    print("Done.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
