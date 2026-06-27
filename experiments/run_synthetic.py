#!/usr/bin/env python
"""Re-run the synthetic experiments (parity, exactly-1, exactly-2, and, or) across all sample
sizes, with the full kOMB grid + baselines, into res/{func}_{size}/.

Synthetic problems are tiny (4 variables): each method runs in milliseconds and NO synthetic
table reports runtime, so the F1 results depend only on the LIMMB build, not the machine. Hence
this runs locally, and batches each (func, size, method) cell into one process (40 runs =
4 targets x 10 versions) rather than using the per-target hard-kill scheduler that the real
datasets need -- synthetic never times out, and 27k one-task processes would just be slow.

    cd experiments && ../.venv/bin/python run_synthetic.py [--jobs N] [--skip-existing]

Config: alpha=0.01, versions v1..v10, every target (the 4 variables), and a kOMB seed
of IAMB(data, target, alpha)[0]. analyse.py reads res/{func}_{size}/ via collect(SYN_FUNCS, ...)
for the metrics-vs-size figure (all sizes) and for the F1 table + ablation (size 1000).
"""

import argparse
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
    DATA, RES, ALPHA, NRUN, silence_native_output, load_samples, target_list,
)
from run_baselines import BASELINES

SYN_FUNCS = ["parity", "exactly-1", "exactly-2", "and", "or"]
SYN_SIZES = [50, 100, 200, 300, 400, 500, 750, 1000]
KS = [0, 1, 2]   # synthetic problems are 4-variable; k=3 would be degenerate
LS = [1, 2, 3]
USE_GTEST = True
KOMB_RE = re.compile(r"kOMB-(\d+)-(\d+)$")


def methods_list():
    return [f"kOMB-{k}-{l}" for k in KS for l in LS] + list(BASELINES)


def _run_one(data, method, target, alpha):
    """Run a single method on one target; returns (mb, n_ci)."""
    m = KOMB_RE.match(method)
    if m:
        k, l = int(m.group(1)), int(m.group(2))
        seed = set(IAMB(data, target, alpha, True)[0])
        return LIMMB.run_komb(data.to_numpy(), target, seed, k, l, alpha, USE_GTEST)
    return BASELINES[method](data, target, alpha)


def _cell_worker(task):
    """Run one (func, size, method) cell -- all targets x versions -- and write its pkl."""
    func, size, method, targets, nrun, alpha = task
    silence_native_output()
    recs = []
    for run in range(nrun):
        data = load_samples(DATA / func / f"{func}_s{size}_v{1 + run}.txt")
        for t in targets:
            t0 = time.process_time()
            mb, n_ci = _run_one(data, method, t, alpha)
            dt = time.process_time() - t0
            recs.append({"method": method, "target": t, "run": run, "time": dt,
                         "n_ci": n_ci, "mb": list(mb), "timeout": False})
    recs.sort(key=lambda r: (r["run"], r["target"]))
    out = RES / f"{func}_{size}" / f"{method}.pkl"
    out.parent.mkdir(parents=True, exist_ok=True)
    tmp = out.parent / (out.name + ".tmp")
    with open(tmp, "wb") as f:
        pickle.dump(recs, f)
    os.replace(tmp, out)            # atomic: a kill mid-write can't leave a corrupt pkl
    return func, size, method, len(recs)


def main():
    p = argparse.ArgumentParser("run_synthetic")
    p.add_argument("--funcs", nargs="+", default=SYN_FUNCS)
    p.add_argument("--sizes", nargs="+", type=int, default=SYN_SIZES)
    p.add_argument("--methods", nargs="+", default=methods_list(),
                   help=f"default: full kOMB grid + baselines ({len(methods_list())} methods)")
    p.add_argument("--jobs", type=int, default=max(1, (os.cpu_count() or 2) - 2),
                   help="parallel processes (default: cpu_count - 2)")
    p.add_argument("--alpha", type=float, default=ALPHA)
    p.add_argument("--nrun", type=int, default=NRUN)
    p.add_argument("--skip-existing", action="store_true",
                   help="skip a (func,size,method) cell whose pkl already exists (to resume)")
    args = p.parse_args()

    tasks = []
    for func in args.funcs:
        targets = target_list(func)   # all 4 variables (top-10 of 4)
        for size in args.sizes:
            for method in args.methods:
                if args.skip_existing and (RES / f"{func}_{size}" / f"{method}.pkl").exists():
                    continue
                tasks.append((func, size, method, targets, args.nrun, args.alpha))

    if not tasks:
        print("Nothing to do (drop --skip-existing to re-run).")
        return

    print(f"Cells to run: {len(tasks)} "
          f"({len(args.funcs)} funcs x {len(args.sizes)} sizes x {len(args.methods)} methods) "
          f"| jobs: {args.jobs}")
    done = 0
    with mp.Pool(args.jobs) as pool:
        for func, size, method, n in pool.imap_unordered(_cell_worker, tasks):
            done += 1
            if done % 50 == 0 or done == len(tasks):
                print(f"  [{done}/{len(tasks)}] latest: {func}_{size}/{method} ({n} recs)")
    print("Done.")


if __name__ == "__main__":
    mp.set_start_method("spawn", force=True)
    main()
