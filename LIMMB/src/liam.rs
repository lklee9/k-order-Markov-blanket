use num::traits::bounds::UpperBounded;
use pyo3::call;
use rand::seq::index::sample;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::iter::once;
use std::usize::MAX;

use crate::dataset::{self, DataSet};
use crate::g2test::{
    self, chi_square_p_val, g2, g2_df, g2_df_eff, g2_stat,
    g2_stat_with_probs, TestResult,
};
use crate::mb::ResultMB;
use crate::metadata::MetaData;
use crate::prob_map::{self, ProbabilityMap};
use crate::prob_tids::ProbabilityTIDs;
use crate::probability::Probability;

pub fn nested_assoc_mine(
    mb: &BTreeSet<usize>,
    prob: &ProbabilityMap,
    init_atts: &BTreeSet<usize>,
    att_t: usize,
    max_sub_mb_size: usize,
    alpha: f64,
) -> Option<ResultMB> {
    let data: &DataSet = prob.get_dataset();
    let mut num_ci: usize = 0;
    // Calculate max sub-mb df multiplier
    let mut mb_nvals: Vec<usize> =
        mb.iter().map(|a| data.nvals[*a]).collect();
    mb_nvals.sort_by(|a, b| b.cmp(a));
    let mut max_fixed_df: usize = data.nvals[att_t];
    for i in 0..usize::min(mb.len(), max_sub_mb_size) {
        max_fixed_df *= mb_nvals[i];
    }
    // Get remaining atts and sort by pval
    let mut atts: Vec<usize> = init_atts.iter().cloned().collect();
    atts.sort_by(|a, b| data.nvals[*b].cmp(&data.nvals[*a]));
    // Mining loop
    let mut stack_to_add: Vec<Vec<usize>> =
        vec![atts.iter().cloned().collect()];
    let mut stack_current: Vec<BTreeSet<usize>> = vec![BTreeSet::new()];
    let mut stack_prob_all: Vec<ProbabilityMap> = vec![prob.clone()];
    let mut iter: usize = 0;
    while !stack_current.is_empty() {
        println!("");
        assert![stack_current.len() == stack_to_add.len()];
        if stack_to_add.is_empty() {
            continue;
        }
        iter += 1;
        let cur: BTreeSet<usize> = stack_current.pop().unwrap();
        let atts: Vec<usize> = stack_to_add.pop().unwrap();
        let prob_all: ProbabilityMap = stack_prob_all.pop().unwrap();
        print!("\r\t{}. cur: [", iter);
        for v in cur.iter() {
            print!("{},", v);
        }
        print!("]");
        print!("\t future: [");
        for v in atts.iter() {
            print!("{},", v);
        }
        print!("]");
        let cur_df: usize = (cur
            .iter()
            .map(|a| data.nvals[*a])
            .reduce(|acc, n| acc * n)
            .unwrap_or(1)
            - 1)
            * max_fixed_df;
        if cur_df * 5 > data.sample_size {
            print!("\tDF TOO LARGE!");
            continue;
        }
        // Only check stuff if cur is non empty
        if cur.len() > 0 {
            let tmp_res = inner_mb_mine(
                mb,
                &cur,
                att_t,
                &prob_all,
                max_sub_mb_size,
                alpha,
            );
            num_ci += tmp_res.num_ci;
            if tmp_res.is_in_mb {
                return Some(ResultMB {
                    mb: cur.union(mb).cloned().collect(),
                    num_ci_test: num_ci,
                });
            }
            // if not in MB and atts empty skip
            if atts.is_empty() {
                print!("\t empty res...");
                continue;
            }
        }
        // set up next iteration
        // filter atts based on future df
        let mut tmp_all_prob = prob_all.clone();
        let mut next_probs: Vec<ProbabilityMap> = Vec::with_capacity(atts.len());
        for i in 0..atts.len() {
            next_probs.push(tmp_all_prob.clone());
            tmp_all_prob = tmp_all_prob.remove_att(atts[i]);
        }
        for i in (0..atts.len()).rev() {
            let a = atts[i];
            let mut next = cur.clone();
            next.insert(a);
            stack_current.push(next);
            stack_to_add.push(atts[(i + 1)..atts.len()].to_vec());
            stack_prob_all.push(next_probs[i].clone());
        }
    }
    return None;
}

pub struct InnerMineRes {
    xs: BTreeSet<usize>,
    is_in_mb: bool,
    num_ci: usize,
    pval: f64,
}

pub fn inner_mb_mine(
    att_mb: &BTreeSet<usize>,
    att_xs: &BTreeSet<usize>,
    att_t: usize,
    prob_xtc: &ProbabilityMap,
    max_sub_mb_size: usize,
    alpha: f64,
) -> InnerMineRes {
    let data: &DataSet = prob_xtc.get_dataset();
    let mut num_ci: usize = 0;
    // Get remaining atts and sort by pval
    let mut atts: Vec<usize> = att_mb.iter().cloned().collect();
    atts.sort_by(|a, b| data.nvals[*a].cmp(&data.nvals[*b]));
    // Mining loop
    let mut stack_to_add: Vec<Vec<usize>> =
        vec![atts.iter().cloned().collect()];
    let mut stack_cur_mb: Vec<BTreeSet<usize>> = vec![BTreeSet::new()];
    let mut stack_prev_rej: Vec<bool> = vec![false];
    let mut iter: usize = 0;
    let mut all_rej: bool = true;
    while !stack_cur_mb.is_empty() {
        println!("");
        assert![stack_cur_mb.len() == stack_to_add.len()];
        if stack_to_add.is_empty() {
            continue;
        }
        iter += 1;
        let mut prev_rej: bool = stack_prev_rej.pop().unwrap();
        let cur_mb: BTreeSet<usize> = stack_cur_mb.pop().unwrap();
        let atts: Vec<usize> = stack_to_add.pop().unwrap();
        print!("\r\t\t{}. cur: [", iter);
        for v in cur_mb.iter() {
            print!("{},", v);
        }
        print!("]");
        print!("\tfuture: [");
        for v in atts.iter() {
            print!("{},", v);
        }
        print!("]");
        // Only check stuff if cur is non empty
        num_ci += 1;
        let cur_df = g2_df(data, att_t, &att_xs, &cur_mb);
        if cur_df * 5 > data.sample_size {
            print!("\tDF TOO LARGE!");
            continue;
        }
        // Check if current atts reject H0, return cur if rejected
        let cur_stat = g2_stat(&prob_xtc, att_t, att_xs, &cur_mb);
        let cur_pval = chi_square_p_val(cur_stat, cur_df);
        print!(
            "\tres_stat: {}, res_df: {}, res_p: {}, prev_rej: {}",
            cur_stat, cur_df, cur_pval, prev_rej
        );
        if cur_mb.len() > 0 {
            if cur_pval <= alpha && !prev_rej {
                return InnerMineRes {
                    xs: att_xs.clone(),
                    is_in_mb: true,
                    num_ci: num_ci,
                    pval: cur_pval
                };
            } else if cur_pval > alpha && prev_rej {
                return InnerMineRes {
                    xs: att_xs.clone(),
                    is_in_mb: false,
                    num_ci: num_ci,
                    pval: cur_pval
                };
            }
            
        }
        // if exit conditions not reached, update flags
        if cur_pval > alpha {
            all_rej = false;
        } else {
            prev_rej = true;
        }
        // if H0 rejected and atts empty skip
        if atts.is_empty() {
            print!("\t empty res...");
            continue;
        }
        // set up next iteration
        if cur_mb.len() >= max_sub_mb_size {
            continue;
        }
        for i in 0..atts.len() {
            let a = atts[i];
            let mut next = cur_mb.clone();
            next.insert(a);
            stack_cur_mb.push(next);
            stack_to_add.push(atts[(i + 1)..atts.len()].to_vec());
            stack_prev_rej.push(prev_rej);
        }
    }
    return InnerMineRes {
        xs: att_xs.clone(),
        is_in_mb: all_rej,
        num_ci: num_ci,
        pval: 0.0
    };
}


// pub fn prune_xs(
//     att_t: usize,
//     att_xs: &BTreeSet<usize>,
//     att_mb: &BTreeSet<usize>,
//     prob_all: &ProbabilityMap,
//     df_limit: usize,
//     alpha: f64,
// ) -> PruneRes {
//     let data: &DataSet = prob_all.get_dataset();
//     let mut xtc: BTreeSet<usize> =
//         att_mb.union(att_xs).cloned().collect();
//     xtc.insert(att_t);
//     let prob_xtc: ProbabilityMap = prob_all.marginalize(&xtc);
//     let mut num_ci: usize = 0;
//     // Get remaining atts and sort by pval
//     let mut atts: Vec<usize> = att_mb.iter().cloned().collect();
//     atts.sort_by(|a, b| data.nvals[*a].cmp(&data.nvals[*b]));
//     // Mining loop
//     let mut stack_to_add: Vec<Vec<usize>> =
//         vec![atts.iter().cloned().collect()];
//     let mut stack_cur_mb: Vec<BTreeSet<usize>> = vec![BTreeSet::new()];
//     let mut iter: usize = 0;
//     while !stack_cur_mb.is_empty() {
//         println!("");
//         assert![stack_cur_mb.len() == stack_to_add.len()];
//         if stack_to_add.is_empty() {
//             continue;
//         }
//         iter += 1;
//         let cur_mb: BTreeSet<usize> = stack_cur_mb.pop().unwrap();
//         let atts: Vec<usize> = stack_to_add.pop().unwrap();
//         print!("\r\t\t{}. cur: [", iter);
//         for v in cur_mb.iter() {
//             print!("{},", v);
//         }
//         print!("]");
//         print!("\tfuture: [");
//         for v in atts.iter() {
//             print!("{},", v);
//         }
//         print!("]");
//         // Only check stuff if cur is non empty
//         if cur_mb.len() > 0 {
//             num_ci += 1;
//             let cur_df = g2_df(data, att_t, &att_xs, &cur_mb);
//             if cur_df > df_limit {
//                 continue;
//             }
//             // Check if current atts reject H0, return cur if rejected
//             let cur_stat = g2_stat(&prob_xtc, att_t, att_xs, &cur_mb);
//             let cur_pval = chi_square_p_val(cur_stat, cur_df);
//             print!(
//                 "\tres_stat: {}, res_df: {}, res_p: {}",
//                 cur_stat, cur_df, cur_pval
//             );
//             if cur_pval > alpha {
//                 // H0 not rejected, xs might be cond indep to y | mb
//                 return PruneRes {
//                     xs: att_xs.clone(),
//                     to_prune: true,
//                     num_ci: num_ci,
//                     pval: cur_pval,
//                 };
//             }
//             // if H0 not rejected and atts empty skip
//             if atts.is_empty() {
//                 print!("\t empty res...");
//                 continue;
//             }
//         }
//         // set up next iteration
//         for i in 0..atts.len() {
//             let a = atts[i];
//             let mut next = cur_mb.clone();
//             next.insert(a);
//             stack_cur_mb.push(next);
//             stack_to_add.push(atts[(i + 1)..atts.len()].to_vec());
//         }
//     }
//     return PruneRes {
//         xs: att_xs.clone(),
//         to_prune: false,
//         num_ci: num_ci,
//         pval: 0.0,
//     };
// }

pub fn prune(
    cmb: &mut BTreeSet<usize>,
    prob: &ProbabilityMap,
    att_target: usize,
    alpha: f64,
    df_limit: usize,
) -> usize {
    println!("\nPruning...");
    // let mut cmb: BTreeSet<usize> = init_mb.clone();
    let mut num_ci: usize = 0;
    let data = prob.get_dataset();
    let mut cur_atts: BTreeSet<usize> = cmb.clone();
    cur_atts.insert(att_target);
    let mut pruned_prob: ProbabilityMap = prob.marginalize(&cur_atts);
    let mut converged: bool = false;
    while !converged && cmb.len() > 0 {
        // Sort cmb by statistic
        let mut reses: Vec<InnerMineRes> = Vec::with_capacity(cmb.len());
        for x in cmb.clone() {
            println!("\n\tpruning {}...", x);
            cmb.remove(&x);
            let res = inner_mb_mine(
                &cmb,
                &BTreeSet::from([x]),
                att_target,
                prob,
                df_limit,
                alpha,
            );
            num_ci += res.num_ci;
            cmb.insert(x);
            if !res.is_in_mb {
                reses.push(res);
            }
        }
        if reses.is_empty() {
            converged = true;
            continue;
        }
        reses.sort_by(|a, b| b.pval.partial_cmp(&a.pval).unwrap());
        let worse_res: &InnerMineRes = &reses[0];
        let x = worse_res.xs.last().unwrap();
        println!(
            "\n\t worse var: {}... nvals: {}, pval: {}, in_mb? {}",
            x,
            data.nvals[*x],
            worse_res.pval,
            worse_res.is_in_mb
        );
        if worse_res.is_in_mb {
            converged = true;
        } else {
            cmb.remove(x);
            pruned_prob = pruned_prob.remove_att(*x);
        }
    }
    return num_ci;
}
