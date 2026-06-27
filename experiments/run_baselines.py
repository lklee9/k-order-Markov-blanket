#!/usr/bin/env python
"""Re-run the non-kOMB baseline MB-discovery methods on the real-world datasets, in parallel,
under the SAME scheduler as run_ablation.py.

The point is runtime comparability: when the kOMB grid is re-run, the baselines must be re-timed
on the same machine (and with the same parallelism) for the paper's runtime table to be a fair
comparison. This script overwrites any existing res/{dataset}/{METHOD}.pkl timings.

Run from this directory (uses relative ./data, ./res, ../pyCausalFS):

    cd experiments && python run_baselines.py [--jobs N] ...

Config: alpha=0.01, size 5000, versions v1..v10, the top-10 largest-MB targets. The baselines are
the pyCausalFS methods BAMB, GSMB, HITON_MB, IAMB, LRH, MMMB, PCMB, STMB. By default every method
is re-run (overwriting its pkl); pass --skip-existing to resume an interrupted run. For timings to
be comparable, do NOT run this concurrently with run_ablation.py (that would oversubscribe the
cores) -- run one after the other.
"""

import argparse
import os
import pickle
import sys
import time

import multiprocessing as mp

sys.path.append("../pyCausalFS/pyCausalFS/")

from CBD.MBs.BAMB import BAMB
from CBD.MBs.GSMB import GSMB
from CBD.MBs.HITON.HITON_MB import HITON_MB
from CBD.MBs.IAMB import IAMB
from CBD.MBs.LCMB import LRH
from CBD.MBs.MMMB.MMMB import MMMB
from CBD.MBs.PCMB.PCMB import PCMB
from CBD.MBs.STMB import STMB

from run_common import (
    DATA, RES, REAL_DATASETS, ALPHA, SIZE, NRUN, Task,
    silence_native_output, load_samples, target_list, schedule, print_summary,
)

# name -> callable(data, target, alpha) -> (mb, n_ci).
BASELINES = {
    "BAMB": lambda d, t, a: BAMB(d, t, a, True),
    "GSMB": lambda d, t, a: GSMB(d, t, a, True),
    "HITON_MB": lambda d, t, a: HITON_MB(d, t, a, True),
    "IAMB": lambda d, t, a: IAMB(d, t, a, True),
    "LRH": lambda d, t, a: LRH(d, t, a, True),
    "MMMB": lambda d, t, a: MMMB(d, t, a, True),
    "PCMB": lambda d, t, a: PCMB(d, t, a, True),
    "STMB": lambda d, t, a: STMB(d, t, a, True),
}

TMP_DIR = RES / ".baselines_tmp"


def _worker(dataset, method, run, target, alpha, size, out_path):
    """Run one (dataset, method, version, target) task and pickle its result dict to out_path."""
    silence_native_output()
    fp = DATA / dataset / f"{dataset}_s{size}_v{1 + run}.txt"
    data = load_samples(fp)
    t0 = time.process_time()
    mb, n_ci = BASELINES[method](data, target, alpha)
    dt = time.process_time() - t0
    rec = {"method": method, "target": target, "run": run, "time": dt,
           "n_ci": n_ci, "mb": list(mb), "timeout": False}
    with open(out_path, "wb") as f:
        pickle.dump(rec, f)


def _cell_label(cell):
    dataset, method = cell
    return f"{dataset} {method}"


def run_all(datasets, methods, jobs, timeout, alpha, size, nrun, skip_existing):
    tasks, expected = [], {}
    for dataset in datasets:
        targets = target_list(dataset)
        (RES / dataset).mkdir(parents=True, exist_ok=True)
        for method in methods:
            if skip_existing and (RES / dataset / f"{method}.pkl").exists():
                continue
            cell = (dataset, method)
            for run in range(nrun):
                for t in targets:
                    fail = {"method": method, "target": t, "run": run,
                            "time": float(timeout), "n_ci": -1, "mb": [], "timeout": True}
                    tasks.append(Task(cell=cell,
                                      uid=f"{dataset}_{method}_{run}_{t}",
                                      args=(dataset, method, run, t, alpha, size),
                                      fail_rec=fail,
                                      label=f"{dataset} {method} run={run} target={t}"))
            expected[cell] = nrun * len(targets)

    if not tasks:
        print("Nothing to do (drop --skip-existing to re-run methods whose pkl exists).")
        return

    print(f"Method-cells to run ({len(expected)}):")
    for cell in sorted(expected, key=_cell_label):
        print(f"  {_cell_label(cell)}  ({expected[cell]} tasks)")
    print(f"Total tasks: {len(tasks)} | jobs: {jobs} | per-target cap: {timeout}s\n")

    out_path_for = lambda cell: RES / cell[0] / f"{cell[1]}.pkl"
    results = schedule(tasks, expected, out_path_for, _worker, jobs, timeout, TMP_DIR)
    print_summary(results, expected, "Baseline runtime summary", _cell_label)


def main():
    p = argparse.ArgumentParser("run_baselines")
    p.add_argument("--datasets", nargs="+", default=REAL_DATASETS)
    p.add_argument("--methods", nargs="+", default=list(BASELINES),
                   help=f"baseline methods to run (default: all of {list(BASELINES)})")
    p.add_argument("--jobs", type=int, default=max(1, (os.cpu_count() or 2) - 2),
                   help="parallel processes (default: cpu_count - 2)")
    p.add_argument("--timeout", type=float, default=1800.0,
                   help="per-target wall-clock cap in seconds (baselines are fast; the cap is a "
                        "safety net and keeps the methodology identical to run_ablation.py)")
    p.add_argument("--alpha", type=float, default=ALPHA)
    p.add_argument("--size", type=int, default=SIZE)
    p.add_argument("--nrun", type=int, default=NRUN)
    p.add_argument("--skip-existing", action="store_true",
                   help="skip a method whose pkl already exists (to resume an interrupted run)")
    args = p.parse_args()

    run_all(args.datasets, args.methods, args.jobs, args.timeout,
            args.alpha, args.size, args.nrun, args.skip_existing)


if __name__ == "__main__":
    mp.set_start_method("spawn", force=True)
    main()
