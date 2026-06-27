#!/usr/bin/env python
"""Generate every data-driven table and figure for the paper and the UAI 2026 rebuttal.

Run from this directory (uses relative ./res, ./data, ../paper, ../pyCausalFS):

    cd experiments && ../.venv/bin/python analyse.py

Outputs
-------
LaTeX (into the paper tree, picked up by a normal LaTeX rebuild):
  ../paper/tab/syn.tex             raw to_latex dump of synthetic F1 (mean/std)
  ../paper/tab/real.tex            raw to_latex dump of real-world F1 (mean/std)
  ../paper/tab/syn-neat-auto.tex   publication-ready synthetic F1 table
  ../paper/tab/real-neat-auto.tex  publication-ready real-world F1 table
  ../paper/tab/real-runtime-neat-auto.tex  merged F1+runtime table the body \\input{}s (Table 2)
  ../paper/tab/runtime.tex         real-world runtimes (rebuttal, Reviewer 3)
  ../paper/tab/kl-ablation.tex     kOMB k-l grid F1 (rebuttal, Reviewer 5)
  ../paper/tab/dsep.tex            minimal d-separator stats (rebuttal, Reviewer 5)
  ../paper/fig/syn_metrics/syn_metrics_grid.{png,pdf}   metrics-vs-size grid (Reviewers 1, 3)

Machine-readable mirrors:
  ./output/*.csv

Note: the legacy hand-curated ../paper/tab/{syn,real}-neat.tex are NOT overwritten; their
auto-generated equivalents land under the *-neat-auto.tex names. The merged benchmark table the
body now \\input{}s -- ../paper/tab/real-runtime-neat-auto.tex -- IS regenerated here, over the
curated method subset REAL_BODY_METHODS.
"""

import glob
import pickle
import sys
from pathlib import Path

sys.path.append("../pyCausalFS/pyCausalFS/")

import matplotlib.pyplot as plt
import networkx as nx
import numpy as np
import pandas as pd
from matplotlib.lines import Line2D

from CBD.MBs.common.realMB import realMB

# --------------------------------------------------------------------------- #
# Configuration
# --------------------------------------------------------------------------- #
RES = Path("res")
DATA = Path("data")
PAPER_TAB = Path("../paper/tab")
PAPER_FIG = Path("../paper/fig")
OUT = Path("output")

SYN_FUNCS = ["parity", "exactly-1", "exactly-2", "and", "or"]
SYN_SIZES = [50, 100, 200, 300, 400, 500, 750, 1000]
# The synthetic F1 table + ablation report a single sample size (run_synthetic.py writes the full
# kOMB grid + baselines at every size into res/{func}_{size}/). 100 is the small-sample regime
# where empirical unfaithfulness makes the baselines fail on and/or/exactly-* while kOMB's
# higher-order tests still recover the MB -- this matches the table in the paper. The
# metrics-vs-size figure shows the full convergence trajectory across SYN_SIZES.
SYN_TABLE_SIZE = 100
REAL_DATASETS = ["Alarm1", "Barley", "Insurance", "Mildew"]

# Baseline methods in display order; kOMB-k-l variants are appended after these.
BASELINE_ORDER = ["BAMB", "GSMB", "HITON_MB", "IAMB", "LRH", "MMMB", "PCMB", "STMB"]

# Curated method subset shown in the merged benchmark table in the body (Table 2). Poorer
# baselines (IAMB, LRH, PCMB, STMB) and extra kOMB-k-l cells are omitted to conserve space;
# edit this list to change which rows the body table shows when results are regenerated.
REAL_BODY_METHODS = ["BAMB", "GSMB", "HITON_MB", "MMMB", "kOMB-1-1", "kOMB-2-2"]

# Methods shown in the metrics-vs-size grid (the only ones run at every sample size).
PLOT_METHODS = ["GSMB", "kOMB-1-1", "kOMB-2-2"]

# Datasets shown in the metrics-vs-size grid figure -- a compact subset of SYN_FUNCS
# ('and' and 'exactly-1' are omitted) so the per-cell labels read large in the paper.
GRID_FUNCS = ["parity", "exactly-2", "or"]

# Shorter column headers used in the neat synthetic tables.
DATASET_DISPLAY = {"exactly-1": "ex-1", "exactly-2": "ex-2"}

# Display names for methods; the paper writes the Grow-Shrink method as "GS".
METHOD_DISPLAY = {"GSMB": "GS"}

pd.set_option("display.max_rows", None)
pd.set_option("display.max_columns", None)
pd.set_option("display.width", 120)


# --------------------------------------------------------------------------- #
# Core metric layer
# --------------------------------------------------------------------------- #
def num_vars(name):
    """Number of variables in a dataset (the dimension of its DAG)."""
    return int(np.loadtxt(DATA / name / f"{name}_graph.txt").shape[0])


def normalize_method(method):
    """Code names the real-world results 'LIAM-*'; the paper calls the method 'kOMB'."""
    return method.replace("LIAM", "kOMB")


def load_results(name, size=None):
    """Read every method pkl for a dataset into a dict of column lists.

    size=None reads the aggregate folder res/{name}/; an integer reads res/{name}_{size}/.
    """
    folder = name if size is None else f"{name}_{size}"
    cols = {k: [] for k in ("data", "method", "target", "run", "time", "n_ci", "mb", "timeout")}
    files = glob.glob(str(RES / folder / "*.pkl"))
    # If the grid was re-run, a fresh native 'kOMB-k-l.pkl' supersedes the legacy 'LIAM-k-l.pkl'
    # of the same cell; skip the legacy duplicate so a method is never double-counted.
    native = {Path(fp).stem for fp in files if Path(fp).stem.startswith("kOMB-")}
    for fp in files:
        stem = Path(fp).stem
        if stem.startswith("LIAM") and stem.replace("LIAM", "kOMB") in native:
            continue
        with open(fp, "rb") as f:
            for rec in pickle.load(f):
                cols["data"].append(name)
                for k in ("method", "target", "run", "time", "n_ci", "mb"):
                    cols[k].append(rec[k])
                # run_ablation.py marks capped runs; older pkls have no such key.
                cols["timeout"].append(bool(rec.get("timeout", False)))
    return cols


def eval_res(name, reses, size=None):
    """Per-(target, run) precision/recall/F1/distance against the true Markov blanket.

    The empty-MB edge-case handling matches the original analysis scripts exactly.
    """
    realmb, _ = realMB(num_vars(name), str(DATA / name / f"{name}_graph.txt"))
    out = {k: [] for k in ("data", "size", "method", "target", "run", "time",
                           "n_ci", "timeout", "precision", "recall", "F1", "distance")}
    timeouts = reses.get("timeout", [False] * len(reses["run"]))
    for i in range(len(reses["run"])):
        out["data"].append(reses["data"][i])
        out["size"].append(size)
        out["method"].append(normalize_method(reses["method"][i]))
        out["target"].append(reses["target"][i])
        out["run"].append(reses["run"][i])
        out["time"].append(reses["time"][i])
        out["n_ci"].append(reses["n_ci"][i])
        out["timeout"].append(timeouts[i])

        cur_mb = set(reses["mb"][i])
        target = reses["target"][i]
        length_tp = len(set(realmb[target]).intersection(cur_mb))
        length_real = len(realmb[target])
        length_res = len(cur_mb)

        if length_real == 0:
            if length_res == 0:
                out["precision"].append(1)
                out["recall"].append(1)
                out["distance"].append(0)
                out["F1"].append(1)
            else:
                out["precision"].append(0)
                out["recall"].append(1)
                out["distance"].append(2 ** 0.5)
                out["F1"].append(0)
        elif length_res != 0:
            precision = length_tp / length_res
            recall = length_tp / length_real
            out["precision"].append(precision)
            out["recall"].append(recall)
            out["distance"].append(((1 - precision) ** 2 + (1 - recall) ** 2) ** 0.5)
            out["F1"].append(
                2 * precision * recall / (precision + recall)
                if precision + recall != 0 else 0
            )
        else:
            out["precision"].append(0)
            out["recall"].append(0)
            out["distance"].append(2 ** 0.5)
            out["F1"].append(0)
    return pd.DataFrame(out)


def collect(names, sizes=None):
    """Concatenate per-row metrics across datasets (and sample sizes, if given)."""
    dfs = []
    if sizes is None:
        for name in names:
            dfs.append(eval_res(name, load_results(name)))
    else:
        for size in sizes:
            for name in names:
                dfs.append(eval_res(name, load_results(name, size), size))
    return pd.concat(dfs, ignore_index=True)


def order_methods(methods):
    """Baselines (fixed order) first, then kOMB-k-l variants sorted by (k, l).

    Drops the bare 'kOMB' (normalized from the incomplete legacy 'LIAM' runs).
    """
    methods = set(methods)
    bases = [m for m in BASELINE_ORDER if m in methods]
    koms = sorted(
        (m for m in methods if m.startswith("kOMB-")),
        key=lambda m: tuple(int(p) for p in m.split("-")[1:]),
    )
    return bases + koms


# --------------------------------------------------------------------------- #
# Table builders
# --------------------------------------------------------------------------- #
def pivot_mean_std(df, value, index="method"):
    """method x (data, {mean,std}) pivot, e.g. for value='F1' or value='time'."""
    return (
        df.pivot_table(values=value, index=index, columns="data",
                       aggfunc=["mean", "std"])
        .swaplevel(0, 1, 1)
        .sort_index(axis=1)
    )


def pivot_to_latex(pivot, float_fmt="{:.6f}"):
    """LaTeX for a method x (data, {mean,std}) pivot (no jinja2 / Styler dependency)."""
    datasets = list(dict.fromkeys(c[0] for c in pivot.columns))
    stats = ["mean", "std"]
    ncol = len(datasets) * len(stats)
    lines = [rf"\begin{{tabular}}{{l{'r' * ncol}}}", r"\toprule"]
    h1 = ["{}"] + [rf"\multicolumn{{{len(stats)}}}{{c}}{{{_disp_data(d)}}}" for d in datasets]
    lines.append(" & ".join(h1) + r" \\")
    lines.append(" & ".join([pivot.index.name or "method"] + stats * len(datasets)) + r" \\")
    lines.append(r"\midrule")
    for method, row in pivot.iterrows():
        cells = [str(method).replace("_", r"\_")]
        for d in datasets:
            for s in stats:
                val = row.get((d, s), np.nan)
                cells.append("" if pd.isna(val) else float_fmt.format(val))
        lines.append(" & ".join(cells) + r" \\")
    lines += [r"\bottomrule", r"\end{tabular}"]
    return "\n".join(lines) + "\n"


def simple_df_to_latex(df, float_fmt="{:.3f}"):
    """LaTeX for a flat DataFrame (no MultiIndex), avoiding the Styler/jinja2 path."""
    def fmt(v):
        if isinstance(v, float):
            return "" if pd.isna(v) else float_fmt.format(v)
        return str(v).replace("_", r"\_")
    lines = [rf"\begin{{tabular}}{{{'l' * df.shape[1]}}}", r"\toprule",
             " & ".join(str(c).replace("_", r"\_") for c in df.columns) + r" \\",
             r"\midrule"]
    for _, row in df.iterrows():
        lines.append(" & ".join(fmt(v) for v in row) + r" \\")
    lines += [r"\bottomrule", r"\end{tabular}"]
    return "\n".join(lines) + "\n"


def write_raw_latex(pivot, path):
    Path(path).write_text(pivot_to_latex(pivot))


def _disp_data(d):
    return DATASET_DISPLAY.get(d, d)


def _method_labels(method):
    """(main label, sub label) for a method row in a neat table, LaTeX-escaped."""
    method = METHOD_DISPLAY.get(method, method)
    parts = method.split("-")
    if len(parts) == 3 and parts[0] == "kOMB" and parts[1].isdigit() and parts[2].isdigit():
        return "kOMB", rf"{{\scriptsize $(k={parts[1]},l={parts[2]})$}}"
    return method.replace("_", r"\_"), ""


def write_neat_latex(df, value, datasets, methods, path,
                     bold_best=True, lower_is_better=False, float_fmt="{:.4f}"):
    """Publication-ready table: rows=methods, cols=datasets, mean over a {\\scriptsize +-std} row.

    The per-column best mean is bolded (max F1, or min when lower_is_better).
    """
    g = (
        df[df.method.isin(methods) & df.data.isin(datasets)]
        .groupby(["method", "data"])[value]
        .agg(["mean", "std"])
    )
    best = {}
    for d in datasets:
        try:
            col = g.xs(d, level="data")["mean"]
            best[d] = col.min() if lower_is_better else col.max()
        except KeyError:
            best[d] = np.nan

    lines = [r"\setlength{\tabcolsep}{5pt}",
             rf"\begin{{tabular}}{{l{'c' * len(datasets)}}}",
             r"\toprule",
             "Method & " + " & ".join(_disp_data(d) for d in datasets) + r" \\",
             r"\midrule"]
    for m in methods:
        main, sub = _method_labels(m)
        means, stds = [], []
        for d in datasets:
            if (m, d) in g.index:
                mu, sd = g.loc[(m, d), "mean"], g.loc[(m, d), "std"]
                cell = float_fmt.format(mu)
                if bold_best and not np.isnan(best[d]) and abs(mu - best[d]) < 1e-9:
                    cell = rf"\textbf{{{cell}}}"
                means.append(cell)
                sd = 0.0 if np.isnan(sd) else sd
                stds.append(rf"{{\scriptsize $\pm${float_fmt.format(sd)}}}")
            else:
                means.append("--")
                stds.append("--")
        lines += ["", main,
                  "& " + " & ".join(means) + r" \\",
                  (sub + " " if sub else "") + "& " + " & ".join(stds) + r" \\"]
    lines += [r"\bottomrule", r"\end{tabular}"]
    Path(path).write_text("\n".join(lines) + "\n")


def write_combined_latex(df, datasets, methods, path,
                         f1_fmt="{:.4f}", time_fmt="{:.3f}", tabcolsep="4pt"):
    """Full-width benchmark table: rows=methods, two column groups (F1 | Runtime).

    Each group spans `datasets`; per method a mean row over a {\\scriptsize +-std} row, exactly as
    write_neat_latex. The per-dataset best F1 (max) and fastest runtime (min) are bolded. Emits
    only the tabular -- main.tex wraps it in a table* with the caption and label.
    """
    nd = len(datasets)
    sel = df[df.method.isin(methods) & df.data.isin(datasets)]
    g_f1 = sel.groupby(["method", "data"])["F1"].agg(["mean", "std"])
    g_time = sel.groupby(["method", "data"])["time"].agg(["mean", "std"])

    best_f1, best_time = {}, {}
    for d in datasets:
        try:
            best_f1[d] = g_f1.xs(d, level="data")["mean"].max()
        except KeyError:
            best_f1[d] = np.nan
        try:
            best_time[d] = g_time.xs(d, level="data")["mean"].min()
        except KeyError:
            best_time[d] = np.nan

    hdr = " & ".join(_disp_data(d) for d in datasets)
    lines = [rf"\setlength{{\tabcolsep}}{{{tabcolsep}}}",
             rf"\begin{{tabular}}{{l{'c' * nd}{'c' * nd}}}",
             r"\toprule",
             rf" & \multicolumn{{{nd}}}{{c}}{{F1 Score}} & \multicolumn{{{nd}}}{{c}}{{Runtime (s)}} \\",
             rf"\cmidrule(lr){{2-{nd + 1}}}\cmidrule(lr){{{nd + 2}-{2 * nd + 1}}}",
             rf"Method & {hdr} & {hdr} \\",
             r"\midrule"]
    for m in methods:
        main, sub = _method_labels(m)
        f1_means, f1_stds, t_means, t_stds = [], [], [], []
        for d in datasets:
            if (m, d) in g_f1.index:
                mu, sd = g_f1.loc[(m, d), "mean"], g_f1.loc[(m, d), "std"]
                cell = f1_fmt.format(mu)
                if not np.isnan(best_f1[d]) and abs(mu - best_f1[d]) < 1e-9:
                    cell = rf"\textbf{{{cell}}}"
                f1_means.append(cell)
                sd = 0.0 if np.isnan(sd) else sd
                f1_stds.append(rf"{{\scriptsize $\pm${f1_fmt.format(sd)}}}")
            else:
                f1_means.append("--")
                f1_stds.append("--")
            if (m, d) in g_time.index:
                mu, sd = g_time.loc[(m, d), "mean"], g_time.loc[(m, d), "std"]
                cell = time_fmt.format(mu)
                if not np.isnan(best_time[d]) and abs(mu - best_time[d]) < 1e-9:
                    cell = rf"\textbf{{{cell}}}"
                t_means.append(cell)
                sd = 0.0 if np.isnan(sd) else sd
                t_stds.append(rf"{{\scriptsize $\pm${time_fmt.format(sd)}}}")
            else:
                t_means.append("--")
                t_stds.append("--")
        lines += ["", main,
                  "& " + " & ".join(f1_means + t_means) + r" \\",
                  (sub + " " if sub else "") + "& " + " & ".join(f1_stds + t_stds) + r" \\"]
    lines += [r"\bottomrule", r"\end{tabular}"]
    Path(path).write_text("\n".join(lines) + "\n")


def mean_std_csv(df, value, methods, datasets, path):
    (
        df[df.method.isin(methods) & df.data.isin(datasets)]
        .groupby(["method", "data"])[value]
        .agg(["mean", "std"])
        .to_csv(path)
    )


# --------------------------------------------------------------------------- #
# Figure
# --------------------------------------------------------------------------- #
def save_fig(fig, path_no_ext):
    """Save a figure as both PNG (preview) and PDF (vector, used by the paper)."""
    path_no_ext = Path(path_no_ext)
    path_no_ext.parent.mkdir(parents=True, exist_ok=True)
    for ext in ("png", "pdf"):
        fig.savefig(path_no_ext.with_suffix(f".{ext}"), dpi=300, bbox_inches="tight")


def plot_metrics_grid(df, datasets, methods, out_path_no_ext,
                      metrics=("F1", "precision", "recall")):
    """One row per metric, one column per dataset; each cell is mean(metric) vs sample size."""
    color_cycle = plt.rcParams.get("axes.prop_cycle").by_key().get(
        "color", ["C0", "C1", "C2", "C3"])
    method_colors = {m: color_cycle[i % len(color_cycle)] for i, m in enumerate(methods)}
    df_plot = df[df["method"].isin(methods)].copy()

    n_rows, n_cols = len(metrics), len(datasets)
    fig, axs = plt.subplots(n_rows, n_cols, figsize=(1.8 * n_cols, 1.0125 * n_rows),
                            sharex="col", sharey="row")
    axs = np.atleast_2d(axs)
    if n_cols == 1:
        axs = axs.reshape(n_rows, 1)

    methods_present = [m for m in methods if m in df_plot["method"].unique()]

    for r, metric in enumerate(metrics):
        for c, dataset in enumerate(datasets):
            ax = axs[r, c]
            if r == 0:
                ax.set_title(dataset, fontsize=10)
            sub = df_plot[df_plot["data"] == dataset]
            plot_df = (
                sub.pivot_table(values=metric, index="size", columns="method",
                                aggfunc="mean")
                .reindex(columns=[m for m in methods if m in sub["method"].unique()])
                .sort_index()
            )
            if plot_df.empty:
                ax.text(0.5, 0.5, "no data", ha="center", va="center",
                        transform=ax.transAxes)
            else:
                plot_df.plot(ax=ax, marker="o", markersize=3, linewidth=1, legend=False,
                             color=[method_colors.get(col, "k") for col in plot_df.columns])
            if c == 0:
                ax.annotate(metric, xy=(-0.15, 0.5), xycoords="axes fraction",
                            fontsize=10, rotation=90, va="center", ha="center")
            if r == n_rows - 1:
                ax.set_xlabel("size", fontsize=9)
            ax.set_ylim(0, 1.05)
            ax.set_yticks([0, 1])
            ax.set_xlim(0, 1050)
            ax.set_xticks([0, 1000])
            if r == n_rows - 1 and c != 0:
                # columns are tight -- blank the leading "0" so it doesn't collide
                # with the previous column's "1000"
                ax.set_xticklabels(["", "1000"])
            ax.tick_params(labelsize=8)
            if c != 0:  # shared y-axis: drop redundant y-ticks on non-first columns
                ax.tick_params(left=False)
            ax.grid(True, alpha=0.3)

    if methods_present:
        method_handles = [Line2D([0], [0], color=method_colors[m], marker="o", markersize=3, lw=1, label=m)
                          for m in methods_present]
        # Blank handle so the "method" label sits inline on the same row as the entries.
        title_handle = Line2D([], [], linestyle="none", marker="none", label="method")
        handles = [title_handle] + method_handles
        labels = ["method"] + [METHOD_DISPLAY.get(m, m) for m in methods_present]
        fig.tight_layout(rect=[0, 0.10, 1, 1])
        # Tighten the horizontal gap between columns to 0.375x of what tight_layout computed.
        fig.subplots_adjust(wspace=fig.subplotpars.wspace * 0.375)
        fig.legend(handles=handles, labels=labels,
                   loc="lower center", bbox_to_anchor=(0.5, 0.0),
                   ncol=len(methods_present) + 1, fontsize=8,
                   columnspacing=1.0, handletextpad=0.4, handlelength=1.5)
    else:
        fig.tight_layout()

    save_fig(fig, out_path_no_ext)
    plt.close(fig)


# --------------------------------------------------------------------------- #
# d-separator statistics (rebuttal, Reviewer 5)
# --------------------------------------------------------------------------- #
def _read_dag(name):
    """Build a networkx DiGraph from the adjacency matrix (edge i->j iff A[i][j]==1)."""
    A = np.loadtxt(DATA / name / f"{name}_graph.txt")
    n = A.shape[0]
    g = nx.DiGraph()
    g.add_nodes_from(range(n))
    for i in range(n):
        for j in range(n):
            if A[i, j] == 1:
                g.add_edge(i, j)
    return g, n


def dsep_stats(datasets, topk=None, cover_thresh=3):
    """Mean/max size of minimal d-separators (restricted to the true MB) for target-non-MB pairs.

    By default every variable serves as a target; set topk to restrict to the topk largest-MB
    targets. For each target and every variable outside its MB we find the minimal d-separator
    drawn from that MB, and report the mean size, the worst-case (max) size, and the fraction of
    pairs whose separator has size <= cover_thresh.
    """
    rows = []
    for name in datasets:
        g, n = _read_dag(name)
        mb, _ = realMB(n, str(DATA / name / f"{name}_graph.txt"))
        targets = sorted(range(n), key=lambda t: (-len(mb[t]), t))
        if topk is not None:
            targets = targets[:topk]
        sizes, skipped = [], 0
        for t in targets:
            mb_t = set(mb[t])
            for v in range(n):
                if v == t or v in mb_t:
                    continue
                try:
                    sep = nx.find_minimal_d_separator(g, t, v, restricted=mb_t)
                except Exception:
                    skipped += 1
                    continue
                if sep is None:
                    skipped += 1
                    continue
                sizes.append(len(sep))
        arr = np.asarray(sizes, dtype=float)
        rows.append({
            "data": name,
            "targets": len(targets),
            "pairs": len(sizes),
            "skipped": skipped,
            "mean_sep_size": arr.mean() if arr.size else float("nan"),
            "max_sep_size": int(arr.max()) if arr.size else 0,
            f"cover_le_{cover_thresh}": (arr <= cover_thresh).mean() if arr.size else float("nan"),
        })
    return pd.DataFrame(rows)


def write_dsep_table():
    """Compute d-separator stats over all targets and write paper/tab/dsep.tex + output/dsep.csv.

    Returns the raw-column DataFrame; only the .tex gets human-readable headers.
    """
    df = dsep_stats(REAL_DATASETS)
    # The 'skipped' column is 0 throughout (a within-MB separator always exists); keep it in the
    # CSV for the record but drop it from the paper table to save width.
    tex = df.drop(columns=["skipped"]).rename(columns={
        "data": "Network", "targets": "Targets", "pairs": "Pairs",
        "mean_sep_size": r"Mean sep.\ size", "max_sep_size": "Max", "cover_le_3": r"Cover $\leq 3$",
    })
    (PAPER_TAB / "dsep.tex").write_text(simple_df_to_latex(tex))
    df.to_csv(OUT / "dsep.csv", index=False)
    return df


# --------------------------------------------------------------------------- #
# Main
# --------------------------------------------------------------------------- #
def main():
    OUT.mkdir(exist_ok=True)
    PAPER_TAB.mkdir(parents=True, exist_ok=True)
    written = []

    # ---- Synthetic F1 (full kOMB-k-l grid at the converged table size) ------
    df_syn = collect(SYN_FUNCS, [SYN_TABLE_SIZE])
    syn_methods = order_methods(df_syn.method)
    f1_syn = pivot_mean_std(df_syn, "F1")
    write_raw_latex(f1_syn, PAPER_TAB / "syn.tex")
    f1_syn.to_csv(OUT / "syn_f1.csv")
    write_neat_latex(df_syn, "F1", SYN_FUNCS, syn_methods, PAPER_TAB / "syn-neat-auto.tex")
    written += ["paper/tab/syn.tex", "paper/tab/syn-neat-auto.tex", "output/syn_f1.csv"]

    # ---- Real-world F1 + runtime -------------------------------------------
    df_real = collect(REAL_DATASETS)
    real_methods = order_methods(df_real.method)
    f1_real = pivot_mean_std(df_real, "F1")
    write_raw_latex(f1_real, PAPER_TAB / "real.tex")
    f1_real.to_csv(OUT / "real_f1.csv")
    write_neat_latex(df_real, "F1", REAL_DATASETS, real_methods, PAPER_TAB / "real-neat-auto.tex")
    written += ["paper/tab/real.tex", "paper/tab/real-neat-auto.tex", "output/real_f1.csv"]

    write_neat_latex(df_real, "time", REAL_DATASETS, real_methods, PAPER_TAB / "runtime.tex",
                     bold_best=False, float_fmt="{:.3f}")
    mean_std_csv(df_real, "time", real_methods, REAL_DATASETS, OUT / "runtime.csv")
    written += ["paper/tab/runtime.tex", "output/runtime.csv"]

    # Merged F1+runtime table the body \input{}s (curated subset; regenerated from results).
    write_combined_latex(df_real, REAL_DATASETS, REAL_BODY_METHODS,
                         PAPER_TAB / "real-runtime-neat-auto.tex")
    written += ["paper/tab/real-runtime-neat-auto.tex"]

    # ---- k-l ablation (synthetic + real, kOMB variants only) ----------------
    df_kl = pd.concat([df_syn, df_real], ignore_index=True)
    kl_methods = sorted(
        {m for m in df_kl.method if m.startswith("kOMB-")},
        key=lambda m: tuple(int(p) for p in m.split("-")[1:]),
    )
    kl_datasets = SYN_FUNCS + REAL_DATASETS
    # F1 is reported over COMPLETED runs only; capped (timed-out) runs are excluded so a
    # too-slow k=3 cell reads as low completion rather than a misleading F1=0.
    df_kl_done = df_kl[~df_kl["timeout"]]
    write_neat_latex(df_kl_done, "F1", kl_datasets, kl_methods, PAPER_TAB / "kl-ablation.tex")
    kl_tab = (
        df_kl_done[df_kl_done.method.isin(kl_methods) & df_kl_done.data.isin(kl_datasets)]
        .groupby(["method", "data"])["F1"].agg(["mean", "std"])
    )
    completion = (
        df_kl[df_kl.method.isin(kl_methods) & df_kl.data.isin(kl_datasets)]
        .assign(done=lambda d: ~d["timeout"])
        .groupby(["method", "data"])["done"].mean()
        .rename("completion")
    )
    kl_tab.join(completion).to_csv(OUT / "kl_ablation.csv")
    written += ["paper/tab/kl-ablation.tex", "output/kl_ablation.csv"]

    # ---- Metrics-vs-sample-size grid figure ---------------------------------
    df_sizes = collect(GRID_FUNCS, SYN_SIZES)
    plot_metrics_grid(df_sizes, GRID_FUNCS, PLOT_METHODS,
                      PAPER_FIG / "syn_metrics" / "syn_metrics_grid")
    written += ["paper/fig/syn_metrics/syn_metrics_grid.{png,pdf}"]

    # ---- d-separator statistics ---------------------------------------------
    df_dsep = write_dsep_table()
    written += ["paper/tab/dsep.tex", "output/dsep.csv"]

    print("\nd-separator statistics (restricted to true MB, all targets):")
    print(df_dsep.to_string(index=False))
    print("\nWrote:")
    for w in written:
        print(f"  {w}")


if __name__ == "__main__":
    main()
