"""Shared infrastructure for the parallel, time-capped MB-discovery runners
(run_ablation.py for kOMB, run_baselines.py for the pyCausalFS baselines).

Each task runs in its own process so a task exceeding the per-target wall-clock cap can be
hard-killed (the kOMB pyo3 call holds the GIL, so threads can't be interrupted). Workers write
their result dict to a temp file; the scheduler reads it on clean exit, or synthesizes a
timeout/crash record otherwise. Results are grouped by "cell" (a method on a dataset); each
cell's pkl is written once all its (version x target) tasks finish.
"""

import multiprocessing as mp
import os
import pickle
import sys
import time
from collections import defaultdict, namedtuple
from pathlib import Path

import numpy as np
import pandas as pd

sys.path.append("../pyCausalFS/pyCausalFS/")

from CBD.MBs.common.realMB import realMB

DATA = Path("data")
RES = Path("res")
REAL_DATASETS = ["Alarm1", "Barley", "Insurance", "Mildew"]

# Shared experiment defaults (get_mbs_for_data).
ALPHA = 0.01
SIZE = 5000
NRUN = 10
NTARGETS = 10

# kOMB writes verbose debug text straight to fd 1/2; over hundreds of tasks that is gigabytes of
# logs. Workers silence those fds unless RUN_DEBUG is set.
_DEBUG = bool(os.environ.get("RUN_DEBUG"))

# A unit of work. `cell` groups results into one output pkl; `uid` names the temp file (must be
# unique); `args` is passed to the worker before out_path; `fail_rec` is recorded verbatim if the
# task times out (or, with n_ci overridden, if it crashes); `label` is for log lines.
Task = namedtuple("Task", ["cell", "uid", "args", "fail_rec", "label"])


def silence_native_output():
    """Redirect fd 1/2 to /dev/null (native code prints debug straight to them)."""
    if _DEBUG:
        return
    devnull = os.open(os.devnull, os.O_WRONLY)
    os.dup2(devnull, 1)
    os.dup2(devnull, 2)
    os.close(devnull)


def load_samples(fp):
    return pd.DataFrame(np.loadtxt(fp, dtype=int))


def target_list(dataset, n_targets=NTARGETS):
    """Top-n largest-MB targets (shared target selection)."""
    gpath = DATA / dataset / f"{dataset}_graph.txt"
    n = int(np.loadtxt(gpath).shape[0])
    mb, _ = realMB(n, str(gpath))
    return sorted(range(len(mb)), key=lambda i: len(mb[i]), reverse=True)[:n_targets]


def schedule(tasks, expected, out_path_for, worker, jobs, timeout, tmp_dir):
    """Run `tasks` with up to `jobs` processes, hard-killing any that exceed `timeout` seconds.

    tasks        : list[Task]
    expected     : {cell: n_tasks}
    out_path_for : cell -> Path of the result pkl for that cell
    worker       : module-level fn, invoked as worker(*task.args, out_path)
    Returns the {cell: [record, ...]} results (also written to disk per cell as they complete).
    """
    tmp_dir = Path(tmp_dir)
    tmp_dir.mkdir(parents=True, exist_ok=True)

    results = defaultdict(list)
    running = {}                       # Process -> (task, out_path, start_monotonic)
    task_it = iter(tasks)
    n_done, n_total = 0, len(tasks)

    def flush_if_complete(cell):
        if len(results[cell]) == expected[cell]:
            recs = sorted(results[cell], key=lambda r: (r["run"], r["target"]))
            out = out_path_for(cell)
            with open(out, "wb") as f:
                pickle.dump(recs, f)
            n_to = sum(r["timeout"] for r in recs)
            print(f"  [written] {out}  ({len(recs)} recs, {n_to} timed out)")

    while True:
        # Fill the pool.
        while len(running) < jobs:
            task = next(task_it, None)
            if task is None:
                break
            out_path = tmp_dir / f"{task.uid}.pkl"
            if out_path.exists():
                out_path.unlink()
            p = mp.Process(target=worker, args=(*task.args, str(out_path)))
            p.start()
            running[p] = (task, out_path, time.monotonic())

        if not running:
            break  # task_it exhausted and nothing left running

        # Poll running processes.
        for p in list(running):
            task, out_path, start = running[p]
            if not p.is_alive():
                p.join()
                rec = None
                if p.exitcode == 0 and out_path.exists():
                    try:
                        with open(out_path, "rb") as f:
                            rec = pickle.load(f)
                    except Exception:
                        rec = None
                if rec is None:
                    rec = dict(task.fail_rec, n_ci=-2)  # crash (distinct from the cap's -1)
                results[task.cell].append(rec)
                if out_path.exists():
                    out_path.unlink()
                del running[p]
                n_done += 1
                flush_if_complete(task.cell)
            elif time.monotonic() - start > timeout:
                p.terminate()
                p.join(5)
                if p.is_alive():
                    p.kill()
                    p.join()
                if out_path.exists():
                    out_path.unlink()
                results[task.cell].append(dict(task.fail_rec))
                del running[p]
                n_done += 1
                print(f"  [timeout] {task.label} (> {timeout}s)  [{n_done}/{n_total}]")
                flush_if_complete(task.cell)

        time.sleep(0.2)

    return results


def print_summary(results, expected, title, cell_label):
    """Print per-cell completed/timeout/mean-time stats. cell_label(cell) -> display string."""
    print(f"\n=== {title} ===")
    print(f"{'cell':26s} {'done':>9s} {'timeout':>7s} {'compl%':>7s} {'mean_s':>9s} {'max_s':>9s}")
    for cell in sorted(expected, key=cell_label):
        recs = results.get(cell, [])
        ok = [r for r in recs if not r["timeout"] and r["n_ci"] != -2]
        n_to = sum(r["timeout"] for r in recs)
        times = [r["time"] for r in ok]
        compl = 100.0 * len(ok) / expected[cell] if expected[cell] else 0.0
        mean_s = sum(times) / len(times) if times else float("nan")
        max_s = max(times) if times else float("nan")
        print(f"{cell_label(cell):26s} {len(recs):>4d}/{expected[cell]:<4d} "
              f"{n_to:>7d} {compl:>6.1f}% {mean_s:>9.1f} {max_s:>9.1f}")
