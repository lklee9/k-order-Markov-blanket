#!/usr/bin/env python
"""Run the missing kOMB-k-l ablation cells on the real-world datasets, in parallel, with a
per-target wall-clock cap.

Each kOMB run is single-threaded, so we run one independent (dataset, k, l, version, target)
task per CPU core. Every task runs in its own process, which lets us hard-kill a task that
exceeds the cap (a pyo3 call holds the GIL, so threads cannot be interrupted). Tasks that hit
the cap are recorded with ``timeout=True`` -- the rate at which k=3 hits the cap is the evidence
that k=3 is too expensive. The scheduler itself lives in run_common.py.

Run from this directory (uses relative ./data, ./res, ../pyCausalFS):

    cd experiments && ../.venv/bin/python run_ablation.py [--jobs N] [--timeout SECS] ...

Each run uses: alpha=0.01, size 5000, data versions v1..v10, the top-10
largest-MB targets, and a kOMB seed of IAMB(data, target, alpha)[0]. Results for each cell land
in res/{dataset}/kOMB-{k}-{l}.pkl as a list[dict] (keys: method,target,run,time,n_ci,mb,timeout)
-- the layout analyse.py reads. The runner is resumable: a cell whose pkl already exists is
skipped unless --force.

To re-run the WHOLE grid on one machine with one cap (so runtimes are comparable across all
cells), pass --force:

    ../.venv/bin/python run_ablation.py --force --timeout 1800

This writes fresh kOMB-{k}-{l}.pkl files. The legacy LIAM-{k}-{l}.pkl results stay on disk but
analyse.py prefers the fresh kOMB-* files, so cells are never double-counted.
"""

import argparse
import glob
import os
import pickle
import re
import sys
import time

import multiprocessing as mp

sys.path.append("../pyCausalFS/pyCausalFS/")

from CBD.MBs.IAMB import IAMB

import LIMMB

from run_common import (
    DATA, RES, REAL_DATASETS, ALPHA, SIZE, NRUN, Task,
    silence_native_output, load_samples, target_list, schedule, print_summary,
)

USE_GTEST = True
TMP_DIR = RES / ".ablation_tmp"
_CELL_RE = re.compile(r"kOMB-(\d+)-(\d+)$")

# (dataset, k, l) cells that FULLY time out at the 1800s cap: every (run, target) task hits
# the wall-clock cap and returns no usable MB. These are skipped even with --force so a re-run
# does not waste hours re-confirming the timeout. Determined empirically from res/ (only
# Barley k=3, all l). Pass --include-skipped to run them anyway; edit this set if the cap, the
# datasets, or the algorithm's runtime changes.
SKIP_CELLS = {("Barley", 3, 1), ("Barley", 3, 2), ("Barley", 3, 3)}


def existing_cells(dataset):
    """(k, l) cells already present in res/{dataset}/ (normalizing LIAM->kOMB)."""
    cells = set()
    for fp in glob.glob(str(RES / dataset / "*.pkl")):
        name = os.path.basename(fp)[:-4].replace("LIAM", "kOMB")
        m = _CELL_RE.match(name)
        if m:
            cells.add((int(m.group(1)), int(m.group(2))))
    return cells


def missing_cells(dataset, ks, ls, force):
    have = set() if force else existing_cells(dataset)
    return [(k, l) for k in ks for l in ls if (k, l) not in have]


def _worker(dataset, k, l, run, target, alpha, size, out_path):
    """Run one (dataset, k, l, version, target) task and pickle its result dict to out_path."""
    silence_native_output()
    fp = DATA / dataset / f"{dataset}_s{size}_v{1 + run}.txt"
    data = load_samples(fp)
    t0 = time.process_time()
    seed = set(IAMB(data, target, alpha, True)[0])
    mb, n_ci = LIMMB.run_komb(data.to_numpy(), target, seed, k, l, alpha, USE_GTEST)
    dt = time.process_time() - t0
    rec = {"method": f"kOMB-{k}-{l}", "target": target, "run": run, "time": dt,
           "n_ci": n_ci, "mb": list(mb), "timeout": False}
    with open(out_path, "wb") as f:
        pickle.dump(rec, f)


def _cell_label(cell):
    dataset, k, l = cell
    return f"{dataset} kOMB-{k}-{l}"


def run_all(datasets, ks, ls, jobs, timeout, alpha, size, nrun, force, include_skipped=False):
    skip = set() if include_skipped else SKIP_CELLS
    tasks, expected, skipped = [], {}, []
    for dataset in datasets:
        targets = target_list(dataset)
        for (k, l) in missing_cells(dataset, ks, ls, force):
            if (dataset, k, l) in skip:
                skipped.append((dataset, k, l))
                continue
            cell = (dataset, k, l)
            for run in range(nrun):
                for t in targets:
                    fail = {"method": f"kOMB-{k}-{l}", "target": t, "run": run,
                            "time": float(timeout), "n_ci": -1, "mb": [], "timeout": True}
                    tasks.append(Task(cell=cell,
                                      uid=f"{dataset}_{k}_{l}_{run}_{t}",
                                      args=(dataset, k, l, run, t, alpha, size),
                                      fail_rec=fail,
                                      label=f"{dataset} kOMB-{k}-{l} run={run} target={t}"))
            expected[cell] = nrun * len(targets)
            (RES / dataset).mkdir(parents=True, exist_ok=True)

    if skipped:
        print("Skipping known-timeout cells (use --include-skipped to run them anyway):")
        for cell in sorted(skipped, key=_cell_label):
            print(f"  {_cell_label(cell)}  (fully timed out at the cap in a prior run)")
        print()

    if not tasks:
        print("Nothing to do: all requested cells already exist (use --force to re-run).")
        return

    print(f"Cells to run ({len(expected)}):")
    for cell in sorted(expected, key=_cell_label):
        print(f"  {_cell_label(cell)}  ({expected[cell]} tasks)")
    print(f"Total tasks: {len(tasks)} | jobs: {jobs} | per-target cap: {timeout}s\n")

    out_path_for = lambda cell: RES / cell[0] / f"kOMB-{cell[1]}-{cell[2]}.pkl"
    results = schedule(tasks, expected, out_path_for, _worker, jobs, timeout, TMP_DIR)
    print_summary(results, expected, "Ablation summary (the k=3 'too slow' evidence)", _cell_label)


def main():
    p = argparse.ArgumentParser("run_ablation")
    p.add_argument("--datasets", nargs="+", default=REAL_DATASETS)
    p.add_argument("--ks", nargs="+", type=int, default=[1, 2, 3])
    p.add_argument("--ls", nargs="+", type=int, default=[1, 2, 3])
    p.add_argument("--jobs", type=int, default=max(1, (os.cpu_count() or 2) - 2),
                   help="parallel processes (default: cpu_count - 2)")
    p.add_argument("--timeout", type=float, default=1800.0,
                   help="per-target wall-clock cap in seconds (default: 1800 = 30 min, ~1.3x the "
                        "slowest legitimate k<=2 run (~23 min) so those finish while k=3 is capped)")
    p.add_argument("--alpha", type=float, default=ALPHA)
    p.add_argument("--size", type=int, default=SIZE)
    p.add_argument("--nrun", type=int, default=NRUN)
    p.add_argument("--force", action="store_true",
                   help="re-run cells even if their pkl already exists")
    p.add_argument("--include-skipped", action="store_true",
                   help=f"also run the known-timeout cells in SKIP_CELLS ({sorted(SKIP_CELLS)})")
    args = p.parse_args()

    run_all(args.datasets, args.ks, args.ls, args.jobs, args.timeout,
            args.alpha, args.size, args.nrun, args.force, args.include_skipped)


if __name__ == "__main__":
    mp.set_start_method("spawn", force=True)
    main()
