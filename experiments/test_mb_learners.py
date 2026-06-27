import numpy as np
import pandas as pd
import time
import sys
sys.path.append("../pyCausalFS/pyCausalFS/")

from CBD.evaluation_MBalgorithm import evaluation
from CBD.MBs.common.realMB import realMB

from CBD.MBs.MMMB.MMMB import MMMB
from CBD.MBs.HITON.HITON_MB import HITON_MB
from CBD.MBs.PCMB.PCMB import PCMB
from CBD.MBs.IPCMB.IPCMB import IPC_MB
from CBD.MBs.GSMB import GSMB
from CBD.MBs.IAMB import IAMB
from CBD.MBs.fast_IAMB import fast_IAMB
from CBD.MBs.inter_IAMB import inter_IAMB
from CBD.MBs.IAMBnPC import IAMBnPC
from CBD.MBs.interIAMBnPC import interIAMBnPC
from CBD.MBs.KIAMB import KIAMB
from CBD.MBs.STMB import STMB
from CBD.MBs.BAMB import BAMB
from CBD.MBs.FBEDk import FBED
from CBD.MBs.MBOR import MBOR
from CBD.MBs.LCMB import LRH

from data_gen import create_nvar_bn, sample_bn

from SSD.MBs.SLLMB import SLL
from SSD.MBs.S2TMB import S2TMB
from SSD.MBs.S2TMB import S2TMB_p

import LIMMB


name_to_learner = {
    "MMMB": lambda d, t, a, k: MMMB(d, t, a, True),
    "IAMB": lambda d, t, a, k: IAMB(d, t, a, True),
    "KIAMB": lambda d, t, a,k: KIAMB(d, t, a, k, True),
    "IAMBnPC": lambda d, t, a, k: IAMBnPC(d, t, a, True),
    "inter_IAMB": lambda d, t, a, k: inter_IAMB(d, t, a, True),
    # "interIAMBnPC": lambda d, t, a, k: interIAMBnPC(d, t, a),
    # "fast_IAMB": lambda d, t, a, k: fast_IAMB(d, t, a, True),
    "GSMB": lambda d, t, a, k: GSMB(d, t, a, True),
    "HITON_MB": lambda d, t, a, k: HITON_MB(d, t, a, True),
    "PCMB": lambda d, t, a, k: PCMB(d, t, a, True),
    "IPCMB": lambda d, t, a, k: IPC_MB(d, t, a, True),
    "STMB": lambda d, t, a, k: STMB(d, t, a, True),
    "BAMB": lambda d, t, a, k: BAMB(d, t, a, True),
    "FBEDk": lambda d, t, a, k: FBED(d, t, k, a, True),
    "FBED0": lambda d, t, a, k: FBED(d, t, 0, a, True),
    "FBED1": lambda d, t, a, k: FBED(d, t, 10, a, True),
    "FBEDinf": lambda d, t, a, k: FBED(d, t, np.inf, a, True),
    "MBOR": lambda d, t, a, k: MBOR(d, t, a, True),
    "LRH": lambda d, t, a, k: LRH(d, t, a, True),
    # "SLL": lambda d, t, a, k: (SLL(d, t)[1], None),
    # "S2TMB":lambda d, t, a, k: (S2TMB(d, t)[1], None),
    # "S2TMB_p":lambda d, t, a, k: (S2TMB_p(d, t)[1], None),
    # "LIMMB": lambda d, t, a, k: LIMMB.learn_mbs(d.to_numpy(), t, a, len(d.columns))
}

def eval_learner(method, data, target, alpha=0.05, k=0.5):
    if method in name_to_learner.keys():
        start_time = time.process_time()
        MB, ci_num = name_to_learner[method](data, target, alpha, k)
        end_time = time.process_time()
    else:
        raise Exception("method input error!")
    return MB, ci_num, end_time - start_time

# bn = create_logicf_bn("and")
# sample = sample_bn(bn, 1000)
# res = {}
# for m in name_to_learner.keys():
#     MB, n_ci, t = eval_learner(m, sample, 1, k=1.0)
#     res[m] = (MB, n_ci, t)
# for m, (MB, n_ci, t) in res.items():
#     print(m, ":", MB, "\n\t num ci: ", n_ci, "\n\t time taken:", t)

# bn = create_logicf_bn("xor")
# sample = sample_bn(bn, 1000)
# res = {}
# for m in name_to_learner.keys():
#     MB, n_ci, t = eval_learner(m, sample, 0, k=1.0)
#     res[m] = (MB, n_ci, t)
# for m, (MB, n_ci, t) in res.items():
#     print(m, ":", MB, "\n\t num ci: ", n_ci, "\n\t time taken:", t)


# start_time = time.process_time()
# mb, n_ci = LIMMB.learn_mbs(sample.to_numpy(), 0, 0.05)
# end_time = time.process_time()
# print(mb, "\n\t num ci: ", n_ci, "\n\t time taken:", end_time - start_time)

# bn = create_nvar_bn("and", 5, 10)
# sample = sample_bn(bn, 1000)
# res = {}
# for m in name_to_learner.keys():
#     try:
#         MB, n_ci, t = eval_learner(m, sample, 0, k=1.0)
#     except:
#         MB = None
#         n_ci = None
#         t = None
#     res[m] = (MB, n_ci, t)
# for m, (MB, n_ci, t) in res.items():
#     print(m, ":", MB, "\n\t num ci: ", n_ci, "\n\t time taken:", t)


# bn = create_nvar_bn("parity", 5, 5)
# sample = sample_bn(bn, 10000)
# res = {}
# for m in name_to_learner.keys():
#     MB, n_ci, t = eval_learner(m, sample, 0, k=1.0)
#     res[m] = (MB, n_ci, t)
# for m, (MB, n_ci, t) in res.items():
#     print(m, ":", MB, "\n\t num ci: ", n_ci, "\n\t time taken:", t)


# start_time = time.process_time()
# mb, n_ci = LIMMB.IAMB(sample.to_numpy(), 0, 0.05)
# end_time = time.process_time()
# print(mb, "\n\t num ci: ", n_ci, "\n\t time taken:", end_time - start_time)

# start_time = time.process_time()
# mb, n_ci = LIMMB.learn_mbs(sample.to_numpy(), 0, 0.05)
# end_time = time.process_time()
# print(mb, "\n\t num ci: ", n_ci, "\n\t time taken:", end_time - start_time)

def load_samples(fileplath):
    mat = np.loadtxt(fileplath, dtype=int)
    df = pd.DataFrame(mat)
    return df

# dataname = "parity-plus"
# dataname = "parity"
# target = 2
# order = 4

# dataname = "Mildew"
# order = 2
# # target = 25
# target = 5


# dataname = "Insurance"
# order = 2
# target = 1
# target = 7
# target = 8
# target = 10

dataname = "Alarm1"
order = 2
target = 21
# target = 22

# dataname = "HailFinder"
# order = 2
# target = 0


# dataname = "Child"
# order = 2
# target = 5
# target = 8
# target = 9
# target = 13

sample = load_samples(f"./data/{dataname}/{dataname}_s5000_v1.txt")
# sample2 = load_samples(f"./data/{dataname}/{dataname}_s5000_v3.txt")
# sample = pd.concat([sample1, sample2], axis=0, ignore_index=True)
# res = {}
# for m in name_to_learner.keys():
#     print(m)
#     try:
#         MB, n_ci, t = eval_learner(m, sample, 0, k=1.0)
#     except:
#         MB = None
#         n_ci = None
#         t = None
#     res[m] = (MB, n_ci, t)
# for m, (MB, n_ci, t) in res.items():
#     print(m, ":", MB, "\n\t num ci: ", n_ci, "\n\t time taken:", t)


# start_time = time.process_time()
# mb, n_ci = LIMMB.IAMB(sample.to_numpy(), target, 0.01)
# end_time = time.process_time()
# print(mb, "\n\t num ci: ", n_ci, "\n\t time taken:", end_time - start_time)



# start_time = time.process_time()
# mb, n_ci = LIMMB.learn_mbs(sample.to_numpy(), 1, 0.01, len(sample.columns) -1, False)
# end_time = time.process_time()
# print(mb, "\n\t num ci: ", n_ci, "\n\t time taken:", end_time - start_time)


# start_time = time.process_time()
# mb, n_ci = LIMMB.learn_mbs(sample.to_numpy(), target, 0.01, len(sample.columns) -1, True)
# end_time = time.process_time()
# print(mb, "\n\t num ci: ", n_ci, "\n\t time taken:", end_time - start_time)

# sci_start_time = time.process_time()
# sci_mb, sci_n_ci = LIMMB.run_nested_assoc_mine(
#     sample.to_numpy(), target, set([]), 0.01, False)
# sci_end_time = time.process_time()



mb_iamb, n_ci_iamb, t_iamb = eval_learner("IAMB", sample, target, alpha=0.01, k=1.0)

mb_bamb, n_ci_bamb, t_bamb = eval_learner("MMMB", sample, target, alpha=0.01, k=1.0)

import logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    handlers=[
        logging.FileHandler("debug.log"),
        logging.StreamHandler()
    ]
)


start_time_old = time.process_time()
mb_old, n_ci_old = LIMMB.run_nested_assoc_mine(
    sample.to_numpy(), target, set(mb_iamb), order, 0.01, True)
end_time_old = time.process_time()

start_time_1 = time.process_time()
mb_1, n_ci_1 = LIMMB.run_komb(
    sample.to_numpy(), target, set(mb_iamb), 1, 2, 0.01, True)
end_time_1 = time.process_time()

start_time = time.process_time()
mb, n_ci = LIMMB.run_komb(
    sample.to_numpy(), target, set(mb_iamb), order, order, 0.01, True)
end_time = time.process_time()

start_time_21 = time.process_time()
mb_21, n_ci_21 = LIMMB.run_komb(
    sample.to_numpy(), target, set(mb_iamb), order, 1, 0.01, True)
end_time_21 = time.process_time()


print(
    "IAMB: \n", "\t mb:", mb_iamb, "\n\t num ci:",
    n_ci_iamb, "\n\t time taken:", t_iamb)
print(
    "MMMB: \n", "\t mb:", mb_bamb, "\n\t num ci:",
    n_ci_bamb, "\n\t time taken:", t_bamb)
print("LIAM 2: \n", "\t mb:", mb, "\n\t num ci:", n_ci, "\n\t time taken:", end_time - start_time)
print("LIAM 2-1: \n", "\t mb:", mb_21, "\n\t num ci:", n_ci_21, "\n\t time taken:", end_time_21 - start_time_21)
print("LIAM 1: \n", "\t mb:", mb_1, "\n\t num ci:", n_ci_1, "\n\t time taken:", end_time_1 - start_time_1)
print("LIAM old: \n", "\t mb:", mb_old, "\n\t num ci:", n_ci_old, "\n\t time taken:", end_time_old - start_time_old)
# print("LIAM (SCI): \n", "\t mb:", sci_mb, "\n\t num ci:", sci_n_ci, "\n\t time taken:", sci_end_time - sci_start_time)

print("="*60)


# start_time = time.process_time()
# res = LIMMB.run_mine(
#     sample.to_numpy(), target, set(mb_iamb), 0.01, len(sample.columns) -1)
# end_time = time.process_time()
# print("time taken:", end_time - start_time)

# for r in res:
#     print(r[0], r[1])


