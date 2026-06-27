import pickle
import os

import numpy as np
from pgmpy.factors.discrete import TabularCPD
from pgmpy.models import DiscreteBayesianNetwork
from pgmpy.sampling import BayesianModelSampling
from pgmpy.readwrite import BIFReader

def sample_bn(bn, nsamp=10000, seed=0):
    sampler = BayesianModelSampling(bn)
    sample = sampler.forward_sample(size=nsamp, seed=seed)
    # with open("../data/xor_graph.txt", "w") as f:
    #     for line in adj:
    #         s = " ".join(map(str, line))
    #         f.write(s + "\n")
    # with open(f"./data/{logicf}_data.pkl", "wb") as f:
    #     pickle.dump(sample, f)
    # np.savetxt("../data/xor_data.txt", sample.values, fmt="%d")
    y = sample["Y"]
    sample.drop(labels=["Y"], axis=1, inplace=True)
    sample.insert(0, "Y", y)
    return sample

def load_bn_and_sample(name):
    reader = BIFReader(f"./data/{name}/{name}.bif")
    return reader.get_model()
    samp = sample_bn(bn, samp_size, i+1)
    np.savetxt(
        f"./data/{func}/{func}_s5000_v{i+1}.txt",
        samp.values,
        fmt="%d",
    )
    


def create_nvar_cpd(nvars, f="parity", k=0):
    n_cond_comb = 2**nvars
    eps = 0.1
    y_vals = np.ones((2, n_cond_comb)) * eps
    if f == "parity":
        for i in range(n_cond_comb):
            v = i.bit_count() % 2
            y_vals[v][i] = 1 - eps
    if f == "parity-plus":
        for i in range(n_cond_comb):
            v = ((i.bit_count() % 2) * (i % 2))
            y_vals[v][i] = 1 - eps
    if f == "exactly-k":
        for i in range(n_cond_comb):
            v = int(i.bit_count() == k)
            y_vals[v][i] = 1 - eps
    if f == "exactly":
        for i in range(n_cond_comb):
            v = int(i.bit_count() == k)
            y_vals[v][i] = 1 - eps
    if f == "prime":
        for i in range(n_cond_comb):
            is_prime = lambda n: n > 1 and all(
                n % i for i in range(2, int(n**0.5) + 1)
            )
            v = int(is_prime(i.bit_count()))
            y_vals[v][i] = 1 - eps
    elif f == "random":
        ps = np.random.rand(n_cond_comb)
        for i in range(n_cond_comb):
            y_vals[0][i] = ps[i]
            y_vals[1][i] = 1 - ps[i]
    elif f == "and":
        for i in range(n_cond_comb - 1):
            y_vals[0][i] = 1 - eps
        y_vals[1][n_cond_comb - 1] = 1 - eps
    elif f == "or":
        y_vals[0][0] = 1 - eps
        for i in range(1, n_cond_comb):
            y_vals[1][i] = 1 - eps
    return y_vals


def create_nvar_bn(f="parity", nvar=3, nind=3, k=0):
    var_names = [f"X{i+1}" for i in range(nvar)]
    ind_names = [f"X{i+1+nvar}" for i in range(nind)]
    bn = DiscreteBayesianNetwork([(v, "Y") for v in var_names])
    bn.add_nodes_from(ind_names)
    adj = np.zeros((nvar + nind + 1, nvar + nind + 1))
    for i in range(nvar):
        adj[i + 1][0] = 1
    cpd_xs = []
    for var in var_names + ind_names:
        cpd_xs.append(
            TabularCPD(
                variable=var, variable_card=2, values=[[0.5], [0.5]]
            )
        )
    y_vals = create_nvar_cpd(len(var_names), f, k)
    cpd_y = TabularCPD(
        variable="Y",
        variable_card=2,
        evidence=var_names,
        evidence_card=[2 for _ in var_names],
        values=y_vals,
    )
    bn.add_cpds(cpd_y, *cpd_xs)
    bn.check_model()
    return bn


def parse_n(n):
    return 0 if n[0] == "Y" else int(n[1])


def bn_to_adj(bn):
    adj = np.zeros((len(bn.nodes()), len(bn.nodes())), dtype=int)
    for n, cs in bn.adjacency():
        i = parse_n(n)
        for c in cs.keys():
            j = parse_n(c)
            adj[i][j] = 1
    return adj


if __name__ == "__main__":
    funcs = ["parity", "and", "or", 'exactly-1', 'exactly-2']
    samp_sizes = [50, 100, 200, 300, 400, 500, 750, 1000]
    # samp_size = 500
    for func in funcs:
        print(f"Generating {func}...")
        func_split = func.split('-')
        k = 0 if len(func_split) == 1 else func_split[-1]
        print(f"Generating {func_split} with k={k}...")
        bn = create_nvar_bn(func_split[0], 3, 0, k)
        adj = bn_to_adj(bn)
        os.makedirs(f"./data/{func}", exist_ok=True)
        with open(f"./data/{func}/{func}_graph.txt", "w") as f:
            for line in adj:
                s = "  ".join(map(str, line))
                f.write(s + "\n")
        for i in range(10):
            for samp_size in samp_sizes:
                samp = sample_bn(bn, samp_size, i+1)
                np.savetxt(
                    f"./data/{func}/{func}_s{samp_size}_v{i+1}.txt",
                    samp.values,
                    fmt="%d",
                )
    for k in range(1, 3):
        func = f'exactly-{k}'
        print(f"Generating {func}...")
        bn = create_nvar_bn("exactly-k", 3, 0, k)
        adj = bn_to_adj(bn)
        os.makedirs(f"./data/{func}", exist_ok=True)
        with open(f"./data/{func}/{func}_graph.txt", "w") as f:
            for line in adj:
                s = "  ".join(map(str, line))
                f.write(s + "\n")
        for i in range(10):
            for samp_size in samp_sizes:
                samp = sample_bn(bn, samp_size, i+1)
                np.savetxt(
                    f"./data/{func}/{func}_s{samp_size}_v{i+1}.txt",
                    samp.values,
                    fmt="%d",
                )
        
