# High-Order Markov Blanket Discovery
### via a *k*-Order Relaxation of the Faithfulness Assumption

**[Loong Kuan Lee](mailto:loong.kuan.lee@iais.fraunhofer.de)**, Ragavi Krishnamoorthy, Nico Piatkowski
Hybrid Intelligence · Fraunhofer IAIS · Germany

*Conference on Uncertainty in Artificial Intelligence (**UAI 2026**)*

[**Paper (arXiv)**](https://arxiv.org/abs/2607.26357) ·
[**PDF**](https://arxiv.org/pdf/2607.26357) ·
[**Code**](https://github.com/lklee9/k-order-Markov-blanket) ·
[**BibTeX**](#citing)

---

## The problem

Markov blanket (MB) discovery finds the minimal set of variables that shields a
target `Y` from all others — the backbone of feature selection, causal discovery,
and Bayesian / Markov network structure learning.

Nearly every constraint-based method rests on one load-bearing assumption,
**faithfulness**: every conditional independence in the distribution `P` must
appear as a separation in the graph `G`. Faithfulness breaks in two ways, and
either way a true blanket member looks irrelevant and is silently dropped:

- **Structurally** — on XOR / parity-type relations, `P` itself is unfaithful.
  No amount of data helps.
- **Empirically** — the network is faithful but the *sample* is not. An unlucky
  finite sample can make a genuine dependence vanish. More data fixes it.

## The relaxation

**`k`-order faithfulness** keeps the guarantee but weakens its conclusion: an
edge must still reveal itself in the data, *but up to `k` extra "witness"
variables may be needed to see it*.

| order | needs | covered by |
| --- | --- | --- |
| `k = 0` | no witness | standard faithfulness |
| `k = 1` | one witness (XOR) | 2-adjacency faithfulness ([Marx et al., 2021](https://proceedings.mlr.press/v161/marx21a.html)) |
| `k ≥ 2` | `k` witnesses (parity over `k+2` variables) | **this work** |

Parity over `k+2` variables needs order `k`, so no existing relaxation reaches
past `k = 1`.

## The algorithm

**kOMB** is a proof-of-concept modification of Grow-and-Shrink that grows the
candidate blanket by whole *witness sets*. It provably recovers the graphical
Markov blanket under `k`-order faithfulness, the Global Markov Property, and an
`l`-bounded separator assumption. The conditional-independence tests and the
kOMB search are implemented in Rust; the eight baselines come from
[pyCausalFS](https://github.com/wt-hu/pyCausalFS).

## Results

On four standard benchmark networks (`N = 5000`, mean F1 over the ten
largest-neighbourhood variables), a **single fixed order `k = 1` beats the best
of all eight baselines on all four networks**:

| network | mean states/var | best of 8 baselines | kOMB `k=1` | kOMB `k=2` |
| --- | --- | --- | --- | --- |
| Alarm1 | 2.8 | 0.769 | **0.780** | 0.821 |
| Insurance | 3.3 | 0.698 | **0.714** | 0.734 |
| Barley | 8.7 | 0.340 | **0.516** | 0.313 |
| Mildew | 15.4 | 0.495 | **0.622** | 0.436 |

The large, statistically clear gains are on the high-cardinality networks
(Barley `+0.18`, Mildew `+0.13`, both `> 2` SE); on Alarm1 and Insurance the
margin is within noise. Raising the search budget cuts both ways — it helps on
the low-cardinality networks and hurts on the high-cardinality ones, where the
independence tests run out of power.

On synthetic **parity** gates the contrast is stark: all eight baselines score
`0.03`, while kOMB with `k = 2` reaches **`1.00`** — exactly, in every run.

**The cost.** Runtime grows with the search budget: `k = 1` takes 5–58 s per
network, `k = 2` takes 14–336 s, and `k = 3` hits a 30-minute cap. `k = l ≤ 2`
is a robust default. Approximate variants that keep the guarantee while taming
the `k = 2` cost are the open problem.

## Citing
{: #citing }

```bibtex
@inproceedings{lee2026highorder,
  title     = {High-Order Markov Blanket Discovery via a k-Order Relaxation of the Faithfulness Assumption},
  author    = {Lee, Loong Kuan and Krishnamoorthy, Ragavi and Piatkowski, Nico},
  booktitle = {Proceedings of the Conference on Uncertainty in Artificial Intelligence (UAI)},
  year      = {2026},
  eprint    = {2607.26357},
  archivePrefix = {arXiv},
  primaryClass  = {cs.LG},
}
```

A machine-readable
[`CITATION.cff`](https://github.com/lklee9/k-order-Markov-blanket/blob/main/CITATION.cff)
is included in the repository.

---

<sub>Funded by the Federal Ministry of Research, Technology and Space of Germany
and the state of North Rhine-Westphalia as part of the Lamarr Institute for
Machine Learning and Artificial Intelligence.</sub>
