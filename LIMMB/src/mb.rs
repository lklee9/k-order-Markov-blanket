use num::traits::bounds::UpperBounded;
use pyo3::call;
use rand::seq::index::sample;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::iter::once;
use std::usize::MAX;

use crate::dataset::{self, DataSet};
use crate::g2test::{
    self, chi_square_p_val, g2, g2_df, g2_stat, g2_stat_with_probs,
    g2_stat_with_tids, TestResult,
};
use crate::metadata::MetaData;
use crate::prob_map::{self, ProbabilityMap};
use crate::prob_tids::ProbabilityTIDs;

// compute conditional g test between T and x
// sort p value
// iteratively reject H0 with the lowest p value
// i.e. H0 is conditionally indep, so rejection means x is in MB(T)

fn filter_sort_mb_candidates(
    mbs: HashMap<BTreeSet<usize>, f64>,
) -> Vec<BTreeSet<usize>> {
    let min_size = mbs.keys().map(|mb| mb.len()).min().unwrap();
    println!("min_size: {}", min_size);
    let mut min_mbs: Vec<BTreeSet<usize>> = mbs
        .iter()
        .filter(|mb| mb.0.len() == min_size)
        .map(|mb| mb.0.clone())
        .collect();
    min_mbs.sort_by(|a, b| mbs[b].partial_cmp(&mbs[a]).unwrap());
    println!("no min mbs: {}", min_mbs.len());
    return min_mbs;
}

pub struct ResultMB {
    pub mb: BTreeSet<usize>,
    pub num_ci_test: usize,
}

pub fn forward_select(
    res: &mut ResultMB,
    prob: &ProbabilityMap,
    att_target: usize,
    alpha: f64,
) {
    let mut cmb: BTreeSet<usize> = res.mb.clone();
    let mut num_ci: usize = res.num_ci_test;
    let mut converged: bool = false;
    let mut atts: BTreeSet<usize> =
        prob.get_atts().difference(&cmb).cloned().collect();
    atts.remove(&att_target);
    let mut it: usize = 0;
    while !converged && !atts.is_empty() {
        it += 1;
        println!(
            "{}: mb size: {} \t att rem: {}",
            it,
            cmb.len(),
            atts.len()
        );
        converged = true;
        // Add variable with the largest heuristic (stat) value
        num_ci += atts.len();
        println!("\t getting test results...");
        let mut test_reses: Vec<TestResult> = atts
            .iter()
            .map(|&v| g2(prob, att_target, &once(v).collect(), &cmb))
            .filter(|res| res.pval <= alpha)
            .collect();
        if !test_reses.is_empty() {
            println!("\t processing test results...");
            test_reses
                .sort_by(|a, b| b.stat.partial_cmp(&a.stat).unwrap());
            for y in &test_reses[0].vars {
                atts.remove(y);
                cmb.insert(*y);
            }
            converged = false;
        }
    }
    res.mb = cmb;
    res.num_ci_test = num_ci;
}

pub fn mafia_tid(
    mb: &BTreeSet<usize>,
    prob: &ProbabilityMap,
    atom_probs: &HashMap<usize, ProbabilityTIDs>,
    init_atts: &BTreeSet<usize>,
    att_t: usize,
    max_mb_size: usize,
    alpha: f64,
) -> Option<(BTreeSet<usize>, usize)> {
    let data: &DataSet = prob.get_dataset();
    let mut num_ci: usize = 0;
    // Get remaining atts and sort by pval
    let mut atts: Vec<usize> = init_atts.iter().cloned().collect();
    let heuristic: HashMap<usize, f64> = atts
        .iter()
        .map(|&a| (a, g2(prob, att_t, &once(a).collect(), mb).pval))
        .collect();
    atts.sort_by(|a, b| {
        heuristic[b].partial_cmp(&heuristic[a]).unwrap()
    });
    // Create probs
    let prob_c: ProbabilityTIDs =
        ProbabilityTIDs::new_marg(data, mb.clone());
    let prob_map_c: ProbabilityMap =
        ProbabilityMap::new_marg(data, mb.clone());
    let mut tc: BTreeSet<usize> = mb.clone();
    tc.insert(att_t);
    let prob_tc: ProbabilityTIDs =
        ProbabilityTIDs::new_marg(data, tc.clone());
    let prob_map_tc: ProbabilityMap =
        ProbabilityMap::new_marg(data, tc.clone());
    let mut xtc: BTreeSet<usize> = mb.clone();
    xtc.extend(&atts);
    xtc.insert(att_t);
    let prob_xtc: ProbabilityTIDs =
        ProbabilityTIDs::new_marg(data, xtc);
    // Mining loop
    let mut smallest_pval_atts: BTreeSet<usize> =
        prob.get_atts().clone();
    let mut smallest_pval: f64 = 2.0;
    let mut df_to_passed_all_atts: HashMap<usize, BTreeSet<usize>> =
        HashMap::new();
    let mut stack_to_add_nvals: Vec<Vec<usize>> =
        vec![atts.iter().map(|&a| data.nvals[a]).collect()];
    let mut stack_to_add: Vec<Vec<usize>> =
        vec![atts.iter().cloned().collect()];
    let mut stack_current: Vec<BTreeSet<usize>> = vec![BTreeSet::new()];
    let mut stack_prob_xc: Vec<ProbabilityTIDs> = vec![prob_c.clone()];
    let mut stack_prob_all: Vec<ProbabilityMap> = vec![prob.clone()];
    let mut iter: usize = 0;
    let mut max_df: usize = MAX;
    while !stack_current.is_empty() {
        assert![stack_to_add_nvals.len() == stack_to_add.len()];
        if stack_to_add.is_empty() {
            continue;
        }
        let cur: BTreeSet<usize> = stack_current.pop().unwrap();
        let atts: Vec<usize> = stack_to_add.pop().unwrap();
        let nvals: Vec<usize> = stack_to_add_nvals.pop().unwrap();
        let prob_xc: ProbabilityTIDs = stack_prob_xc.pop().unwrap();
        let prob_xtc: ProbabilityTIDs =
            prob_xc.merge(&atom_probs[&att_t]);
        let prob_all: ProbabilityMap = stack_prob_all.pop().unwrap();
        iter += 1;
        print!("\r\t{}. cur: [", iter);
        for v in cur.iter() {
            print!("{},", v);
        }
        print!("]");
        // if reached max mb size, skip
        if cur.len() > max_mb_size {
            continue;
        }
        let cur_df = g2_df(data, att_t, &cur, mb);
        // Give up if current df is already too high
        if cur_df >= max_df {
            print!("\t skipped: {} >= {}", cur_df, max_df);
            continue;
        }
        // Only check stuff if cur is non empty
        if cur.len() > 0 {
            num_ci += 1;
            // Check if current atts reject H0, return cur if rejected
            let cur_stat = g2_stat_with_tids(
                &prob_xtc, &prob_xc, &prob_tc, &prob_c,
            );
            let cur_pval = chi_square_p_val(cur_stat, cur_df);
            print!(
                "\tres_stat: {}, res_df: {}, res_p: {}",
                cur_stat, cur_df, cur_pval
            );
            if cur_pval <= alpha {
                return Some((cur, num_ci));
            }
            // if H0 not rejected and atts empty skip
            if atts.is_empty() {
                continue;
            }
            // Get the upper bound statistic with the rem vars to add
            // if upper bound on stat can't reject H0 at next step
            // this branch is doomed, exit early
            let mut all_future_atts = cur.clone();
            all_future_atts.extend(&atts);
            let upper_stat: f64 = g2_stat_with_probs(
                &prob_all,
                &prob_all.remove_att(att_t),
                &prob_map_tc,
                &prob_map_c,
            );
            // let tmp_res: TestResult =
            //     g2(prob, att_t, &all_future_atts, mb);
            // println!("nvals: {}", nvals.len());
            // * nvals.iter().min().unwrap();
            let pval: f64 = chi_square_p_val(upper_stat, cur_df);
            // print!("\t upper pval: {}", pval);
            // print!("\t smallest pval: {}", smallest_pval);
            if pval > alpha {
                max_df = cur_df;
                continue;
                print!("\n\n\tall vars: [");
                for a in prob_all.get_atts() {
                    print!("{},", a);
                }
                println!("]");
                println!("\tupper g-stat: {}", upper_stat);
                println!("\tnext df: {}", cur_df);
                println!("\tupper pval: {}", pval);
                let mut similar_df: usize = 0;
                for df in df_to_passed_all_atts.keys() {
                    if *df >= cur_df {
                        similar_df = *df;
                        break;
                    }
                }
                let recent_passed = df_to_passed_all_atts
                    .get(&similar_df)
                    .unwrap_or(prob.get_atts());
                let cause: BTreeSet<usize> = recent_passed
                    .difference(prob_all.get_atts())
                    .cloned()
                    .collect();
                print!("\trecent vars: [");
                for a in recent_passed.iter() {
                    print!("{},", a);
                }
                println!("]");
                print!("\trecent - all vars: [");
                for a in cause.iter() {
                    print!("{},", a);
                }
                println!("]");
                if cause.len() > 0 {
                    return Some((cause, num_ci));
                } else {
                    continue;
                }
                // return None;
                // continue;
            } else {
                df_to_passed_all_atts
                    .insert(cur_df, prob_all.get_atts().clone());
            }
        }
        // set up next iteration
        // filter atts based on future df
        let mut next_atts: Vec<usize> = Vec::with_capacity(atts.len());
        for a in atts {
            let mut next_cur = cur.clone();
            next_cur.insert(a);
            let next_df = g2_df(data, att_t, &next_cur, mb);
            if next_df < max_df {
                next_atts.push(a);
            }
        }
        let mut all_next_prob: Vec<ProbabilityMap> =
            Vec::with_capacity(next_atts.len());
        all_next_prob.push(prob_all.clone());
        for i in (1..next_atts.len()).rev() {
            let a = next_atts[i];
            all_next_prob.push(
                all_next_prob[next_atts.len() - i - 1].remove_att(a),
            );
        }
        for i in 0..next_atts.len() {
            let a = next_atts[i];
            let mut next = cur.clone();
            next.insert(a);
            stack_current.push(next);
            // stack_to_add.push(atts[(i+1)..(atts.len())].to_vec());
            // stack_to_add_nvals.push(nvals[(i+1)..(atts.len())].to_vec());
            stack_to_add.push(next_atts[0..i].to_vec());
            stack_to_add_nvals.push(nvals[0..i].to_vec());
            stack_prob_xc.push(prob_xc.merge(&atom_probs[&a]));
            stack_prob_all
                .push(all_next_prob[next_atts.len() - i - 1].clone());
        }
    }
    return None;
}

pub fn mafia(
    mb: &BTreeSet<usize>,
    prob: &ProbabilityMap,
    init_atts: &BTreeSet<usize>,
    att_t: usize,
    max_mb_size: usize,
    alpha: f64,
) -> Option<(BTreeSet<usize>, usize)> {
    let data: &DataSet = prob.get_dataset();
    let mut num_ci: usize = 0;
    // Get remaining atts and sort by pval
    let mut atts: Vec<usize> = init_atts.iter().cloned().collect();
    let heuristic: HashMap<usize, f64> = atts
        .iter()
        .map(|&a| (a, g2(prob, att_t, &once(a).collect(), mb).pval))
        .collect();
    atts.sort_by(|a, b| {
        heuristic[b].partial_cmp(&heuristic[a]).unwrap()
    });
    // Create probs
    let prob_c: ProbabilityMap = prob.marginalize(&mb);
    let mut tc: BTreeSet<usize> = mb.clone();
    tc.insert(att_t);
    let prob_tc: ProbabilityMap = prob.marginalize(&tc);
    // Mining time
    let mut stack_to_add_nvals: Vec<Vec<usize>> =
        vec![atts.iter().map(|&a| data.nvals[a]).collect()];
    let mut stack_to_add: Vec<Vec<usize>> =
        vec![atts.iter().cloned().collect()];
    let mut stack_current: Vec<BTreeSet<usize>> = vec![BTreeSet::new()];
    let mut stack_prob: Vec<ProbabilityMap> = vec![prob.clone()];
    let mut iter: usize = 0;
    let mut max_df: usize = MAX;
    while !stack_current.is_empty() {
        assert![stack_to_add_nvals.len() == stack_to_add.len()];
        if stack_to_add.is_empty() {
            continue;
        }
        let cur: BTreeSet<usize> = stack_current.pop().unwrap();
        let atts: Vec<usize> = stack_to_add.pop().unwrap();
        let nvals: Vec<usize> = stack_to_add_nvals.pop().unwrap();
        let par_prob: ProbabilityMap = stack_prob.pop().unwrap();
        iter += 1;
        print!("\n\t{}. cur: [", iter);
        for v in cur.iter() {
            print!("{},", v);
        }
        print!("]");
        // if reached max mb size, skip
        if cur.len() > max_mb_size {
            continue;
        }
        let cur_df = g2_df(data, att_t, &cur, mb);
        // Give up if current df is already too high
        if cur_df >= max_df {
            print!("\t skipped: {} >= {}", cur_df, max_df);
            continue;
        }
        let mut cur_all: BTreeSet<usize> =
            cur.clone().union(mb).cloned().collect();
        if cur.len() > 0 {
            num_ci += 1;
            // Check if current atts reject H0, return cur if rejected
            let cur_res: TestResult = g2(&par_prob, att_t, &cur, mb);
            print!(
                "\tres_stat: {}, res_df: {}, res_p: {}",
                cur_res.stat, cur_res.df, cur_res.pval
            );
            if cur_res.pval <= alpha {
                return Some((cur, num_ci));
            }
            // if H0 not rejected and atts empty skip
            if atts.is_empty() {
                print!("\t SKIPPING BECAUSE atts EMPTY!");
                continue;
            }
        }
        cur_all.extend(&atts);
        cur_all.insert(att_t);
        let cur_all_prob = par_prob.marginalize(&cur_all);
        // Only check stuff if cur is non empty
        if cur.len() > 0 {
            // Get the upper bound statistic with the rem vars to add
            // if upper bound on stat can't reject H0 at next step
            // this branch is doomed, exit early
            let tmp_prob_xc = cur_all_prob.remove_att(att_t);
            let stat: f64 = g2_stat_with_probs(
                &cur_all_prob,
                &tmp_prob_xc,
                &prob_tc,
                &prob_c,
            );
            // println!("nvals: {}", nvals.len());
            num_ci += 1;
            let df: usize = g2_df(prob.get_dataset(), att_t, &cur, mb);
            // * nvals.iter().min().unwrap();
            let pval: f64 = chi_square_p_val(stat, df);
            // println!("\n\tupper g-stat: {}", stat);
            // println!("\tnext df: {}", df);
            print!("\tupper pval: {}", pval);
            if pval > alpha {
                max_df = cur_df;
                continue;
            } 
        }
        // set up next iteration
        // filter atts based on future df
        let mut next_atts: Vec<usize> = Vec::with_capacity(atts.len());
        for a in atts {
            let mut next_cur = cur.clone();
            next_cur.insert(a);
            let next_df = g2_df(data, att_t, &next_cur, mb);
            if next_df < max_df {
                next_atts.push(a);
            }
        }
        for i in 0..next_atts.len() {
            let a = next_atts[i];
            let mut next = cur.clone();
            next.insert(a);
            stack_current.push(next);
            // stack_to_add.push(atts[(i+1)..(atts.len())].to_vec());
            // stack_to_add_nvals.push(nvals[(i+1)..(atts.len())].to_vec());
            stack_to_add.push(next_atts[0..i].to_vec());
            stack_to_add_nvals.push(nvals[0..i].to_vec());
            stack_prob.push(cur_all_prob.clone());
        }
    }
    return None;
}

pub fn mine(
    res: &mut ResultMB,
    prob: &ProbabilityMap,
    att_target: usize,
    max_mb_size: usize,
    alpha: f64,
    use_tid: bool,
) {
    let data = prob.get_dataset();
    let mut cmb: BTreeSet<usize> = res.mb.clone();
    let mut num_ci: usize = res.num_ci_test;
    let mut atts: BTreeSet<usize> =
        prob.get_atts().difference(&cmb).cloned().collect();
    atts.remove(&att_target);
    let mut it: usize = 0;
    let mut converged: bool = false;
    let atom_probs: HashMap<usize, ProbabilityTIDs> = prob
        .get_atts()
        .iter()
        .map(|&a| {
            (a, ProbabilityTIDs::new_marg(data, once(a).collect()))
        })
        .collect();
    while !converged && !atts.is_empty() {
        it += 1;
        print!(
            "\n{}: mb size: {} \t att rem: {} \t mb: [",
            it,
            cmb.len(),
            atts.len()
        );
        for a in cmb.iter() {
            print!("{},", a);
        }
        println!("]");
        // let res: Option<(BTreeSet<usize>, usize)> =
        //     mafia(&cmb, prob, att_target, max_mb_size, alpha);
        let res: Option<(BTreeSet<usize>, usize)> = if use_tid {
            mafia_tid(
                &cmb,
                prob,
                &atom_probs,
                &atts,
                att_target,
                max_mb_size,
                alpha,
            )
        } else {
            mafia(&cmb, prob, &atts, att_target, max_mb_size, alpha)
        };
        match res {
            Some(r) => {
                num_ci += r.1;
                for y in r.0 {
                    cmb.insert(y);
                    atts.remove(&y);
                }
            }
            None => {
                converged = true;
            }
        }
    }
    print!(
        "\nfinal: mb size: {} \t att rem: {}\t mb: [",
        cmb.len(),
        atts.len()
    );
    for a in cmb.iter() {
        print!("{},", a);
    }
    println!("]");
    res.mb = cmb;
    res.num_ci_test = num_ci;
}

pub fn backward_select(
    res: &mut ResultMB,
    prob: &ProbabilityMap,
    att_target: usize,
    alpha: f64,
) {
    let mut cmb: BTreeSet<usize> = res.mb.clone();
    let mut num_ci: usize = res.num_ci_test;
    let mut converged: bool = false;
    let mut atts: BTreeSet<usize> =
        prob.get_atts().difference(&cmb).cloned().collect();
    atts.remove(&att_target);
    let mut tmp_cmb: BTreeSet<usize> = prob.get_atts().clone();
    tmp_cmb.remove(&att_target);
    let mut it: usize = 0;
    while !converged && !atts.is_empty() {
        it += 1;
        converged = false;
        println!(
            "{}: mb size: {} \t att rem: {}",
            it,
            cmb.len(),
            atts.len()
        );
        // Add variable with the largest heuristic (stat) value
        let mut y: Option<usize> = None;
        let mut lowest_pval: f64 = 1.0;
        for &x in atts.iter() {
            num_ci += 1;
            tmp_cmb.remove(&x);
            let res: TestResult =
                g2(prob, att_target, &once(x).collect(), &tmp_cmb);
            tmp_cmb.insert(x);
            if res.pval <= alpha && res.pval < lowest_pval {
                lowest_pval = res.pval;
                y = Some(x);
            }
        }
        match y {
            Some(x) => {
                cmb.insert(x);
                tmp_cmb.insert(x);
                atts.remove(&x);
            }
            None => {
                converged = true;
            }
        }
    }
    res.mb = cmb;
    res.num_ci_test = num_ci;
}

pub fn prune(
    res: &mut ResultMB,
    prob: &ProbabilityMap,
    att_target: usize,
    alpha: f64,
) {
    let mut cmb: BTreeSet<usize> = res.mb.clone();
    let mut num_ci: usize = res.num_ci_test;
    let data = prob.get_dataset();
    let mut cur_atts: BTreeSet<usize> = res.mb.clone();
    cur_atts.insert(att_target);
    let mut pruned_prob: ProbabilityMap = prob.marginalize(&cur_atts);
    let mut converged: bool = false;
    while !converged && cmb.len() > 0 {
        let mut reses: Vec<TestResult> = Vec::with_capacity(cmb.len());
        for x in cmb.clone() {
            cmb.remove(&x);
            num_ci += 1;
            let res =
                g2(&pruned_prob, att_target, &once(x).collect(), &cmb);
            cmb.insert(x);
            reses.push(res);
        }
        reses.sort_by(|a, b| a.stat.partial_cmp(&b.stat).unwrap());
        let worse_res: &TestResult = &reses[0];
        let x = worse_res.vars.last().unwrap();
        println!(
            "\t worse var: {}... nvals: {}, stat: {}, df: {}, pval: {}",
            x,
            data.nvals[*x],
            worse_res.stat,
            worse_res.df,
            worse_res.pval
        );
        if worse_res.pval > alpha {
            cmb.remove(x);
            pruned_prob = pruned_prob.remove_att(*x);
        } else {
            converged = true;
        }
    }
    res.mb = cmb;
    res.num_ci_test = num_ci;
}

pub fn find_mb_for_var(
    prob: &ProbabilityMap,
    att_target: usize,
    alpha: f64,
) -> ResultMB {
    let mut res = ResultMB {
        mb: BTreeSet::new(),
        num_ci_test: 0,
    };
    println!("Forward Selection...");
    forward_select(&mut res, prob, att_target, alpha);
    println!("Backward Selection...");
    mine(
        &mut res,
        prob,
        att_target,
        prob.get_atts().len(),
        alpha,
        false,
    );
    // backward_select(&mut res, prob, att_target, alpha);
    println!("Pruning...");
    prune(&mut res, prob, att_target, alpha);
    println!("Done!");
    res
}

pub fn LIAM_tid(
    prob: &ProbabilityMap,
    att_target: usize,
    alpha: f64,
) -> ResultMB {
    let mut res = ResultMB {
        mb: BTreeSet::new(),
        num_ci_test: 0,
    };
    println!("Forward Selection...");
    forward_select(&mut res, prob, att_target, alpha);
    println!("Backward Selection...");
    mine(
        &mut res,
        prob,
        att_target,
        prob.get_atts().len(),
        alpha,
        true,
    );
    // backward_select(&mut res, prob, att_target, alpha);
    println!("Pruning...");
    prune(&mut res, prob, att_target, alpha);
    println!("Done!");
    res
}
pub fn iamb(
    prob: &ProbabilityMap,
    att_target: usize,
    alpha: f64,
) -> ResultMB {
    let mut res = ResultMB {
        mb: BTreeSet::new(),
        num_ci_test: 0,
    };
    println!("Forward Selection...");
    forward_select(&mut res, prob, att_target, alpha);
    println!("Pruning...");
    prune(&mut res, prob, att_target, alpha);
    println!("Done!");
    res
}
