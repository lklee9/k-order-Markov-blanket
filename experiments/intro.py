import pickle
import os

import numpy as np
from pgmpy.factors.discrete import TabularCPD
from pgmpy.models import DiscreteBayesianNetwork
from pgmpy.sampling import BayesianModelSampling
from pgmpy.readwrite import BIFReader

def create_nvar_cpd(nvars):
    n_cond_comb = 2**nvars
    eps = 0.1
    y_vals = np.ones((2, n_cond_comb)) * eps
    for i in range(n_cond_comb):
        v = 1 if i.bit_count() == 1 else 0
        y_vals[v][i] = 1 - eps
    return y_vals


def create_nvar_bn(nvar):
    x_names = [f"X{i+1}" for i in range(nvar)]
    adj = np.zeros((nvar +1, nvar + 1))
    bn = DiscreteBayesianNetwork([(v, "Y") for v in x_names])
    for i in range(len(x_names)):
        adj[i+1][0] = 1
    cpd_xs = []
    for var in x_names:
        cpd_xs.append(
            TabularCPD(
                variable=var, variable_card=2, values=[[0.5], [0.5]]
            )
        )
    cpd_y = TabularCPD(
        variable="Y",
        variable_card=2,
        evidence=x_names,
        evidence_card=[2 for _ in x_names],
        values=create_nvar_cpd(nvar),
    )            
    bn.add_cpds(cpd_y, *cpd_xs)
    bn.check_model()
    return adj, bn

def sample_bn(bn, nsamp=10, seed=0):
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

if __name__ == "__main__":
    samp_size = 1000
    adj, bn = create_nvar_bn(3)
    os.makedirs("./data/intro", exist_ok=True)
    with open("./data/intro/intro_graph.txt", "w") as f:
        for line in adj:
            s = "  ".join(map(str, line))
            f.write(s + "\n")
    for i in range(10):
        samp = sample_bn(bn, samp_size, i+1)
        np.savetxt(
            f"./data/intro/intro_s5000_v{i+1}.txt",
            samp.values,
            fmt="%d",
        )
