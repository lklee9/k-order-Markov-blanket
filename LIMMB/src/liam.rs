use itertools::Itertools;
use log::{info, trace};
use num::traits::bounds::UpperBounded;
use pyo3::call;
use rand::seq::index::sample;
use statrs::function::factorial::binomial;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::iter::once;
use std::time::Instant;
use std::usize::MAX;

use crate::ci_tests::{sci_min_sample_size, CITest, GTest, SCI};
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
    prob_atoms: &Vec<ProbabilityTIDs>,
    init_atts: &BTreeSet<usize>,
    att_t: usize,
    known_xs_deps: &mut HashMap<BTreeSet<usize>, Vec<BTreeSet<usize>>>,
    max_sub_mb_size: usize,
    alpha: f64,
    use_gtest: bool,
) -> Option<ResultMB> {
    let data: &DataSet = prob_atoms[0].get_dataset();
    let mut num_ci: usize = 0;
    // Get top 3 vars in mb with highest nvals
    let mut mb_vars_sorted: Vec<usize> =
        mb.clone().into_iter().collect();
    mb_vars_sorted.sort_by(|a, b| data.nvals[*a].cmp(&data.nvals[*b]));
    let top_nval_mb_vars: BTreeSet<usize> = mb_vars_sorted
        [0..(usize::min(3, mb.len()))]
        .iter()
        .cloned()
        .collect();
    // Get remaining atts and sort by pval
    let mut atts: Vec<usize> = init_atts.iter().cloned().collect();
    let tmp_prob_tc: ProbabilityTIDs = ProbabilityTIDs::new_marg(
        data,
        mb.union(&BTreeSet::from([att_t])).cloned().collect(),
    );
    let xs_g: HashMap<usize, f64> = atts
        .iter()
        .map(|a| {
            (
                *a,
                GTest::new(
                    &tmp_prob_tc.merge(&prob_atoms[*a]).into(),
                    att_t,
                    &BTreeSet::from([*a]),
                    mb,
                )
                .get_badness(),
            )
        })
        .collect();
    // let xs_g_mi: HashMap<usize, f64> = atts
    //     .iter()
    //     .map(|a| {
    //         (
    //             *a,
    //             GTest::new(
    //                 &prob_atoms[att_t].merge(&prob_atoms[*a]).into(),
    //                 att_t,
    //                 &BTreeSet::from([*a]),
    //                 &BTreeSet::new(),
    //             )
    //             .get_badness(),
    //         )
    //     })
    //     .collect();
    // atts.sort_by(|a, b| xs_g_mi[b].partial_cmp(&xs_g_mi[a]).unwrap());
    atts.sort_by(|a, b| xs_g[a].partial_cmp(&xs_g[b]).unwrap());
    atts.reverse();
    // Mining loop
    let mut queue_to_add: VecDeque<Vec<usize>> =
        VecDeque::from(vec![atts.iter().cloned().collect()]);
    let mut queue_current: VecDeque<BTreeSet<usize>> =
        VecDeque::from(vec![BTreeSet::new()]);
    let mut queue_prob: VecDeque<ProbabilityTIDs> =
        VecDeque::from(vec![prob_atoms[att_t].clone()]);
    let mut iter: usize = 0;
    let mut num_comb: f64 = 0.0;
    for k in 1..(max_sub_mb_size + 1) {
        num_comb += binomial(atts.len() as u64, k as u64);
    }
    let start = Instant::now();
    for k in 1..(max_sub_mb_size + 1) {
        for cur_vec in atts.iter().cloned().combinations(k) {
            let dur: u64 = start.elapsed().as_secs();
            print!(
                "\rk={}. {:.2}% nodes explored in {}s. ETA: {:.4}s.",
                k,
                ((100 * iter) as f64 / num_comb),
                dur,
                ((dur as f64) / (iter as f64)) * num_comb,
            );
            iter += 1;
            let cur: BTreeSet<usize> = cur_vec.into_iter().collect();
            let mut cur_prob = prob_atoms[att_t].clone();
            for a in cur.iter() {
                cur_prob = cur_prob.merge(&prob_atoms[*a]);
            }
            trace!("\r\t{}. cur: [", iter);
            for v in cur.iter() {
                trace!("{},", v);
            }
            trace!("]");
            trace!("\t future: [");
            for v in atts.iter() {
                trace!("{},", v);
            }
            trace!("]");
            // Only check stuff if cur is non empty
            // let too_weak: bool = if use_gtest && cur.len() <= 1 {
            let mut too_weak: bool = {
                let df: usize =
                    g2_df(data, att_t, &cur, &top_nval_mb_vars);
                df * 5 > data.sample_size
            };
            if too_weak {
                let mut card_x: usize = 1;
                for v in cur.iter() {
                    card_x = card_x * data.nvals[*v];
                }
                let mut card_cond: usize = 1;
                for v in top_nval_mb_vars.iter() {
                    card_cond *= data.nvals[*v];
                }
                let card_t: usize = data.nvals[att_t];
                too_weak =
                    sci_min_sample_size(card_x, card_t, card_cond)
                        > data.sample_size as f64;
            };
            if too_weak {
                trace!("\tTEST NOT STRONG ENOUGH!");
                continue;
            }
            // order atts in mb by pval in descending order
            let mb_att_to_badness: HashMap<usize, f64> = mb
                .iter()
                .map(|a| {
                    (
                        *a,
                        SCI::new(
                            &cur_prob.merge(&prob_atoms[*a]).into(),
                            att_t,
                            &cur,
                            &BTreeSet::from([*a]),
                        )
                        .badness,
                    )
                })
                .collect();
            let mut mb_sorted: Vec<usize> =
                mb.iter().cloned().collect();
            mb_sorted.sort_by(|a, b| {
                mb_att_to_badness[b]
                    .partial_cmp(&mb_att_to_badness[a])
                    .unwrap()
            });
            let mut accepted_xs: BTreeSet<usize> = cur.clone();
            loop {
                let mut tmp_xs: BTreeSet<usize> = BTreeSet::new();
                for x in accepted_xs.iter() {
                    let atom_set = BTreeSet::from([*x]);
                    let mut tmp_mb = mb_sorted.clone();
                    for y in accepted_xs.iter() {
                        if y != x {
                            tmp_mb.push(*y);
                        }
                    }
                    let tmp_res: InnerMineRes = find_deps(
                        &tmp_mb,
                        &atom_set,
                        att_t,
                        &accepted_xs
                            .difference(&atom_set)
                            .cloned()
                            .collect(),
                        prob_atoms,
                        &cur_prob,
                        known_xs_deps,
                        max_sub_mb_size,
                        alpha,
                        use_gtest,
                    );
                    num_ci += tmp_res.num_ci;
                    trace!("{} is in mb? {} \n", x, tmp_res.is_in_mb);
                    if tmp_res.is_in_mb {
                        tmp_xs.insert(*x);
                    }
                }
                if tmp_xs == accepted_xs {
                    break;
                }
                accepted_xs = tmp_xs;
            }
            if accepted_xs.len() > 0 {
                return Some(ResultMB {
                    mb: accepted_xs.union(mb).cloned().collect(),
                    num_ci_test: num_ci,
                });
            }
        }
    }
    // while !queue_current.is_empty() {
    //     trace!("\n");
    //     assert![queue_current.len() == queue_to_add.len()];
    //     if queue_to_add.is_empty() {
    //         continue;
    //     }
    //     iter += 1;
    //     let cur: BTreeSet<usize> = queue_current.pop_front().unwrap();
    //     let atts: Vec<usize> = queue_to_add.pop_front().unwrap();
    //     let cur_prob: ProbabilityTIDs = queue_prob.pop_front().unwrap();
    //     trace!("\r\t{}. cur: [", iter);
    //     for v in cur.iter() {
    //         trace!("{},", v);
    //     }
    //     trace!("]");
    //     trace!("\t future: [");
    //     for v in atts.iter() {
    //         trace!("{},", v);
    //     }
    //     trace!("]");
    //     // Only check stuff if cur is non empty
    //     if cur.len() > 0 {
    //         // let too_weak: bool = if use_gtest && cur.len() <= 1 {
    //         let mut too_weak: bool = {
    //             let df: usize =
    //                 g2_df(data, att_t, &cur, &top_nval_mb_vars);
    //             df * 5 > data.sample_size
    //         };
    //         if too_weak {
    //             let mut card_x: usize = 1;
    //             for v in cur.iter() {
    //                 card_x = card_x * data.nvals[*v];
    //             }
    //             let mut card_cond: usize = 1;
    //             for v in top_nval_mb_vars.iter() {
    //                 card_cond *= data.nvals[*v];
    //             }
    //             let card_t: usize = data.nvals[att_t];
    //             too_weak = sci_min_sample_size(card_x, card_t, card_cond)
    //                 > data.sample_size as f64;
    //         };
    //         if too_weak {
    //             trace!("\tTEST NOT STRONG ENOUGH!");
    //             continue;
    //         }
    //         // order atts in mb by pval in descending order
    //         let mb_att_to_badness: HashMap<usize, f64> = mb
    //             .iter()
    //             .map(|a| {
    //                 (
    //                     *a,
    //                     SCI::new(
    //                         &cur_prob.merge(&prob_atoms[*a]).into(),
    //                         att_t,
    //                         &cur,
    //                         &BTreeSet::from([*a]),
    //                     )
    //                     .badness,
    //                 )
    //             })
    //             .collect();
    //         let mut mb_sorted: Vec<usize> =
    //             mb.iter().cloned().collect();
    //         mb_sorted.sort_by(|a, b| {
    //             mb_att_to_badness[b]
    //                 .partial_cmp(&mb_att_to_badness[a])
    //                 .unwrap()
    //         });
    //         let mut accepted_xs: BTreeSet<usize> = cur.clone();
    //         while true {
    //             let mut tmp_xs: BTreeSet<usize> = BTreeSet::new();
    //             for x in accepted_xs.iter() {
    //                 let atom_set = BTreeSet::from([*x]);
    //                 let mut tmp_mb = mb_sorted.clone();
    //                 for y in accepted_xs.iter() {
    //                     if y != x {
    //                         tmp_mb.push(*y);
    //                     }
    //                 }
    //                 let tmp_res: InnerMineRes = find_deps(
    //                     &tmp_mb,
    //                     &atom_set,
    //                     att_t,
    //                     &accepted_xs.difference(&atom_set).cloned().collect(),
    //                     prob_atoms,
    //                     &cur_prob,
    //                     known_xs_deps,
    //                     max_sub_mb_size,
    //                     alpha,
    //                     use_gtest,
    //                 );
    //                 num_ci += tmp_res.num_ci;
    //                 trace!("{} is in mb? {} \n", x, tmp_res.is_in_mb);
    //                 if tmp_res.is_in_mb {
    //                     tmp_xs.insert(*x);
    //                 }
    //             }
    //             if tmp_xs == accepted_xs {
    //                 break;
    //             }
    //             accepted_xs = tmp_xs;
    //         }
    //         if accepted_xs.len() > 0 {
    //             return Some(ResultMB {
    //                 mb: accepted_xs.union(mb).cloned().collect(),
    //                 num_ci_test: num_ci,
    //             });
    //         }
    //         // if not in MB and atts empty skip
    //         if atts.is_empty() {
    //             trace!("\t empty res...");
    //             continue;
    //         }
    //     }
    //     if cur.len() >= max_sub_mb_size {
    //         continue;
    //     }
    //     // set up next iteration
    //     // filter atts based on future df
    //     for i in (0..atts.len()).rev() {
    //         let a = atts[i];
    //         let mut next = cur.clone();
    //         next.insert(a);
    //         queue_current.push_back(next);
    //         queue_to_add.push_back(atts[0..i].to_vec());
    //         queue_prob.push_back(cur_prob.merge(&prob_atoms[a]));
    //     }
    // }
    return None;
}

pub struct InnerMineRes {
    xs: BTreeSet<usize>,
    is_in_mb: bool,
    num_ci: usize,
    badness: f64,
}

fn find_deps(
    att_mb: &Vec<usize>,
    att_xs: &BTreeSet<usize>,
    att_t: usize,
    init_mb: &BTreeSet<usize>,
    prob_atoms: &Vec<ProbabilityTIDs>,
    prob_xt: &ProbabilityTIDs,
    known_xs_deps: &mut HashMap<BTreeSet<usize>, Vec<BTreeSet<usize>>>,
    max_sub_mb_size: usize,
    alpha: f64,
    use_gtest: bool,
) -> InnerMineRes {
    trace!("\n\t\tfind deps with mb: [");
    att_mb.iter().for_each(|a| trace!("{},", a));
    trace!("], with init_mb: [");
    init_mb.iter().for_each(|a| trace!("{},", a));
    trace!("]\n");
    let att_mb_set: BTreeSet<usize> = att_mb.iter().cloned().collect();
    assert![init_mb.is_subset(&att_mb_set)];
    // let eff_max_sub_mb_size: usize = max_sub_mb_size + init_mb.len();
    // let eff_max_sub_mb_size: usize = max_sub_mb_size - 1;
    let eff_max_sub_mb_size: usize = max_sub_mb_size;
    let mut num_ci: usize = 0;
    // Mining loop
    let mut atts: Vec<usize> = att_mb.clone();
    atts.retain(|a| !init_mb.contains(a));
    let mut min_res: InnerMineRes = inner_mb_mine(
        att_mb,
        att_xs,
        att_t,
        init_mb,
        prob_atoms,
        prob_xt,
        known_xs_deps,
        max_sub_mb_size,
        alpha,
        use_gtest,
    );
    num_ci += min_res.num_ci;
    let mut prob_xtc = prob_xt.clone();
    for a in init_mb {
        prob_xtc = prob_xtc.merge(&prob_atoms[*a]);
    }
    let mut found_seps: Vec<BTreeSet<usize>> = Vec::new();
    let mut stack_to_add: Vec<Vec<usize>> = vec![atts];
    let mut stack_added: Vec<BTreeSet<usize>> = vec![init_mb.clone()];
    let mut stack_prob: Vec<ProbabilityTIDs> = vec![prob_xtc];
    let mut num_dep: usize = 0;
    let mut iter: usize = 0;
    while !stack_added.is_empty() {
        iter += 1;
        assert![stack_added.len() == stack_to_add.len()];
        let cur_mb: BTreeSet<usize> = stack_added.pop().unwrap();
        let to_add: Vec<usize> = stack_to_add.pop().unwrap();
        let cur_prob: ProbabilityTIDs = stack_prob.pop().unwrap();
        // Trace traces
        trace!("\n\t\t{}. dep: [", iter);
        cur_mb.iter().for_each(|v| trace!("{},", v));
        trace!("], to add: [");
        to_add.iter().for_each(|v| trace!("{},", v));
        trace!("]");
        // Check if cur_mb is subset of a know sep
        let mut is_seperated: bool = false;
        for sep in found_seps.iter() {
            if cur_mb.is_subset(sep) {
                is_seperated = true;
            }
        }
        if !is_seperated {
            // Run CI Test
            num_ci += 1;
            let num_var = cur_mb.len() + att_xs.len() + 1;
            // let ci: Box<dyn CITest> = if use_gtest && num_var <= 5 {
            let ci: Box<dyn CITest> = if use_gtest {
                let tmp = GTest::new(
                    &cur_prob.clone().into(),
                    att_t,
                    att_xs,
                    &cur_mb,
                );
                trace!(
                    "\tres_stat: {}, res_df: {}, res_p: {}",
                    tmp.stat,
                    tmp.df,
                    tmp.pval,
                );
                Box::new(tmp)
            } else {
                let tmp = SCI::new(
                    &cur_prob.clone().into(),
                    att_t,
                    att_xs,
                    &cur_mb,
                );
                trace!("\tres_stat: {}", tmp.stat);
                Box::new(tmp)
            };
            // If CI Test is strong enough check result of test
            if !ci.is_too_weak() && cur_mb.len() <= eff_max_sub_mb_size
            {
                if ci.is_not_cond_indep(alpha) && cur_mb != *init_mb {
                    // This subset is gives depedency, return current mb
                    // trace!("\n\t\tFOUND DEPS: [");
                    // cur_mb.iter().for_each(|a| trace!("{},", a));
                    // trace!("]");
                    num_dep += 1;
                    let inner_res = inner_mb_mine(
                        att_mb,
                        att_xs,
                        att_t,
                        &cur_mb,
                        prob_atoms,
                        prob_xt,
                        known_xs_deps,
                        max_sub_mb_size,
                        alpha,
                        use_gtest,
                    );
                    num_ci += inner_res.num_ci;
                    if inner_res.badness < min_res.badness {
                        min_res = inner_res;
                    }
                }
                trace!("min res in mb? {}\n", min_res.is_in_mb);
                if min_res.is_in_mb {
                    return min_res;
                }
            } else {
                trace!(
                    "\tTEST TOO WEAK! {} {}<={}",
                    ci.is_too_weak(),
                    cur_mb.len(),
                    eff_max_sub_mb_size
                );
            }
        }
        if cur_mb.len() == eff_max_sub_mb_size {
            continue;
        }
        trace!("\n");
        // CI test is too weak, mine subsets of current node
        for i in 0..to_add.len() {
            let a = to_add[i];
            let mut next = cur_mb.clone();
            next.insert(a);
            stack_added.push(next);
            stack_to_add.push(to_add[0..i].to_vec());
            stack_prob.push(cur_prob.merge(&prob_atoms[a]));
        }
    }
    return InnerMineRes {
        xs: min_res.xs,
        is_in_mb: false,
        num_ci: num_ci,
        badness: -1.0 * (num_dep as f64),
    };
}

/*
 * Function to determine if there is a subset of mb where
 * xs is not cond. indep. to t given the subset
 *
 * Done by mining subsets of mb.
 * Cannot immediately use the entirety of mb due to possibly not having
 * enough samples. (Although this is ideal)
 */
pub fn inner_mb_mine(
    att_mb: &Vec<usize>,
    att_xs: &BTreeSet<usize>,
    att_t: usize,
    init_mb: &BTreeSet<usize>,
    prob_atoms: &Vec<ProbabilityTIDs>,
    prob_xt: &ProbabilityTIDs,
    known_xs_deps: &mut HashMap<BTreeSet<usize>, Vec<BTreeSet<usize>>>,
    max_sub_mb_size: usize,
    alpha: f64,
    use_gtest: bool,
) -> InnerMineRes {
    trace!("\n\t\t\tfind seps with mb: [");
    att_mb.iter().for_each(|a| trace!("{},", a));
    trace!("], with init_mb: [");
    init_mb.iter().for_each(|a| trace!("{},", a));
    trace!("], with prob dom: [");
    prob_xt.get_atts().iter().for_each(|a| trace!("{},", a));
    trace!("]\n");
    let att_mb_set: BTreeSet<usize> = att_mb.iter().cloned().collect();
    assert![init_mb.is_subset(&att_mb_set)];
    let eff_max_sub_mb_size: usize = max_sub_mb_size + init_mb.len();
    // let eff_max_sub_mb_size: usize = max_sub_mb_size;
    // check if init_mb is more specific than known dependent init_mb
    trace!("\t\t\tknown deps for [ ");
    att_xs.iter().for_each(|a| trace!("{},", a));
    trace!("]: \n");
    if let Some(deps) = known_xs_deps.get(att_xs) {
        let mut i = 1;
        for dep in deps {
            trace!("\t\t\t\t{}. [", i);
            dep.iter().for_each(|a| trace!("{},", a));
            trace!("]\n");
            if att_mb_set == *dep {
                trace!("\t\t\t\tFOUND EXISTING DEP!!\n");
                return InnerMineRes {
                    xs: att_xs.clone(),
                    is_in_mb: true,
                    num_ci: 0,
                    badness: -1.0,
                };
            }
            i += 1;
        }
    }
    trace!("\t\t\tmine start...\n");
    let mut num_ci: usize = 0;
    // Mining loop
    let mut atts: Vec<usize> = att_mb.clone();
    atts.retain(|a| !init_mb.contains(a));
    let mut prob_xtc = prob_xt.clone();
    for a in init_mb {
        prob_xtc = prob_xtc.merge(&prob_atoms[*a]);
    }
    let mut max_bad: f64 = -1.0;
    let mut stack_to_add: Vec<Vec<usize>> = vec![atts];
    let mut stack_added: Vec<BTreeSet<usize>> = vec![init_mb.clone()];
    let mut stack_prob: Vec<ProbabilityTIDs> = vec![prob_xtc];
    let mut iter: usize = 0;
    let mut accepted_once: bool = false;
    while !stack_added.is_empty() {
        assert![stack_added.len() == stack_to_add.len()];
        assert![stack_prob.len() == stack_to_add.len()];
        let cur_mb: BTreeSet<usize> = stack_added.pop().unwrap();
        let to_add: Vec<usize> = stack_to_add.pop().unwrap();
        let cur_prob: ProbabilityTIDs = stack_prob.pop().unwrap();
        // Trace traces
        if cur_mb.len() == usize::min(eff_max_sub_mb_size, att_mb.len())
        {
            // Run CI Test
            iter += 1;
            trace!("\t\t\t\t{}. sep: [", iter);
            cur_mb.iter().for_each(|v| trace!("{},", v));
            trace!("], to add: [");
            to_add.iter().for_each(|v| trace!("{},", v));
            trace!("]");
            num_ci += 1;
            let total_vars = cur_mb.len() + att_xs.len() + 1;
            let ci: Box<dyn CITest> = if use_gtest && total_vars <= 5 {
                let tmp = GTest::new(
                    &cur_prob.clone().into(),
                    att_t,
                    att_xs,
                    &cur_mb,
                );
                if tmp.is_too_weak() {
                    let tmp2 = SCI::new(
                        &cur_prob.clone().into(),
                        att_t,
                        att_xs,
                        &cur_mb,
                    );
                    trace!("\tres_stat: {}", tmp2.stat);
                    Box::new(tmp2)
                } else {
                    trace!(
                        "\tres_stat: {}, res_df: {}, res_p: {}",
                        tmp.stat,
                        tmp.df,
                        tmp.pval,
                    );
                    Box::new(tmp)
                }
            } else {
                let tmp = SCI::new(
                    &cur_prob.clone().into(),
                    att_t,
                    att_xs,
                    &cur_mb,
                );
                trace!("\tres_stat: {}", tmp.stat);
                Box::new(tmp)
            };
            // If CI Test is strong enough check result of test
            if !ci.is_too_weak() && cur_mb.len() <= eff_max_sub_mb_size
            {
                max_bad = f64::max(max_bad, ci.get_badness());
                if !ci.is_not_cond_indep(alpha) {
                    trace!("\tFOUND SEP!!!\n");
                    // This subset shows xs is not in mb of t
                    // sep = sep.intersection(&cur_mb).cloned().collect();
                    return InnerMineRes {
                        xs: att_xs.clone(),
                        is_in_mb: false,
                        num_ci: num_ci,
                        badness: max_bad,
                    };
                } else if cur_mb != *init_mb
                    || init_mb.len() == 0
                    || to_add.len() == 0
                {
                    accepted_once = true;
                    trace!("\t ACCEPTED!!!\n");
                    continue;
                }
            } else if cur_mb.len() - 1 == init_mb.len() {
                trace!("\tinit mb too rare!!\n");
                return InnerMineRes {
                    xs: att_xs.clone(),
                    is_in_mb: false,
                    num_ci: num_ci,
                    badness: ci.get_badness(),
                };
            } else {
                trace!(
                    "\tTEST TOO WEAK! {} {}<={}\n",
                    ci.is_too_weak(),
                    cur_mb.len(),
                    eff_max_sub_mb_size
                );
            }
        } else if cur_mb.len() < eff_max_sub_mb_size {
            for i in 0..to_add.len() {
                let a = to_add[i];
                let mut next = cur_mb.clone();
                next.insert(a);
                stack_added.push(next);
                stack_to_add.push(to_add[0..i].to_vec());
                stack_prob.push(cur_prob.merge(&prob_atoms[a]));
            }
        }
    }
    if accepted_once {
        if let Some(deps) = known_xs_deps.get_mut(att_xs) {
            deps.push(att_mb_set.clone());
        } else {
            known_xs_deps
                .insert(att_xs.clone(), vec![att_mb_set.clone()]);
        }
    }
    return InnerMineRes {
        xs: att_xs.clone(),
        is_in_mb: accepted_once,
        num_ci: num_ci,
        badness: if accepted_once { -1.0 } else { max_bad },
    };
}

pub fn prune(
    cmb: &mut BTreeSet<usize>,
    prob_atoms: &Vec<ProbabilityTIDs>,
    att_target: usize,
    known_xs_deps: &mut HashMap<BTreeSet<usize>, Vec<BTreeSet<usize>>>,
    alpha: f64,
    sub_mb_limit: usize,
    use_gtest: bool,
) -> usize {
    trace!("\nPruning: [");
    for a in cmb.iter() {
        trace!("{},", a);
    }
    trace!("]");
    // let mut cmb: BTreeSet<usize> = init_mb.clone();
    let mut num_ci: usize = 0;
    let mut cur_atts: BTreeSet<usize> = cmb.clone();
    cur_atts.insert(att_target);
    let mut converged: bool = false;
    let mut remaining: BTreeSet<usize> = cmb.clone();

    while !converged && cmb.len() > 0 {
        // Sort cmb by statistic
        let mut reses: Vec<InnerMineRes> =
            Vec::with_capacity(cmb.len());
        for x in remaining.clone() {
            trace!("\n\tpruning {}...", x);
            let atom = BTreeSet::from([x]);
            let cur_prob = prob_atoms[att_target].merge(&prob_atoms[x]);
            let res: InnerMineRes = find_deps(
                &cmb.difference(&atom).cloned().collect(),
                &atom,
                att_target,
                &BTreeSet::new(),
                // &mb_var_group[&x].difference(&atom).cloned().collect(),
                prob_atoms,
                &cur_prob,
                known_xs_deps,
                sub_mb_limit,
                alpha,
                use_gtest,
            );
            num_ci += res.num_ci;
            if !res.is_in_mb {
                reses.push(res);
            } else {
                remaining.remove(&x);
            }
        }
        if reses.is_empty() {
            converged = true;
            continue;
        }
        // for res in reses {
        //     let x = res.xs.last().unwrap();
        //     trace!(
        //         "\n\tworse x: {}... badness: {}, in_mb? {}\n",
        //         x, res.badness, res.is_in_mb
        //     );
        //     remaining.remove(&x);
        //     cmb.remove(&x);
        //     pruned_prob = pruned_prob.remove_att(*x);
        // }
        reses
            .sort_by(|a, b| b.badness.partial_cmp(&a.badness).unwrap());
        let worse_res: &InnerMineRes = &reses[0];
        let x = worse_res.xs.last().unwrap();
        trace!(
            "\n\tworse x: {}... badness: {}, in_mb? {}\n",
            x,
            worse_res.badness,
            worse_res.is_in_mb
        );
        if worse_res.is_in_mb {
            converged = true;
        } else {
            remaining.remove(&x);
            cmb.remove(&x);
        }
    }
    return num_ci;
}
