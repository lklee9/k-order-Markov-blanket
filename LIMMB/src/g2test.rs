use pyo3::prelude::*;
use rand::seq::SliceRandom;
use rand::{rng, Rng};
use statrs::distribution::{ChiSquared, ContinuousCDF};
use statrs::statistics::Data;
use std::collections::{BTreeSet, HashMap};
use std::process::Termination;
use std::cmp::max;

use crate::dataset::DataSet;
use crate::prob_map::ProbabilityMap;
use crate::prob_tids::ProbabilityTIDs;

pub struct TestResult {
    pub vars: BTreeSet<usize>,
    pub stat: f64,
    pub df: usize,
    pub pval: f64,
}

pub fn chi_square_p_val(stat: f64, df: usize) -> f64 {
    // println!("Testing C2({}) with stat: {}", df, stat);
    let chi2: ChiSquared = ChiSquared::new(df as f64).unwrap();
    1.0 - chi2.cdf(stat)
}

pub fn g2_stat_with_tids(
    prob_xtc: &ProbabilityTIDs,
    prob_xc: &ProbabilityTIDs,
    prob_tc: &ProbabilityTIDs,
    prob_c: &ProbabilityTIDs,
) -> f64 {
    let mut stat: f64 = 0.0;
    for comb in prob_xtc.get_combs() {
        let vals: HashMap<usize, usize> = prob_xtc
            .get_atts()
            .iter()
            .enumerate()
            .map(|(i, &a)| (a, comb[i]))
            .collect();
        let s_xtc: f64 = prob_xtc.f(comb) as f64;
        let s_xc: f64 = prob_xc.f_map(&vals) as f64;
        let s_tc: f64 = prob_tc.f_map(&vals) as f64;
        let s_c: f64 = prob_c.f_map(&vals) as f64;
        // println!(
        //     "stat: {}, s_xtc: {}, s_xc: {}, s_tc: {}, s_c: {}",
        //     stat, s_xtc, s_xc, s_tc, s_c
        // );
        if s_xtc != 0.0 {
            // println!("cur1: {}", (s_xtc * s_c / (s_xc * s_tc)));
            // println!("cur2: {}", (s_xtc * s_c / (s_xc * s_tc)).ln());
            if s_xc == 0.0 || s_tc == 0.0 {
                std::panic!("marginals are less frequent?!");
            }
            stat += s_xtc * (s_xtc * s_c / (s_xc * s_tc)).ln();
        }
    }
    stat * 2.0
}


pub fn g2_stat_with_probs(
    prob_xtc: &ProbabilityMap,
    prob_xc: &ProbabilityMap,
    prob_tc: &ProbabilityMap,
    prob_c: &ProbabilityMap,
) -> f64 {
    let mut stat: f64 = 0.0;
    for comb in prob_xtc.get_combs() {
        let vals: HashMap<usize, usize> = prob_xtc
            .get_atts()
            .iter()
            .enumerate()
            .map(|(i, &a)| (a, comb[i]))
            .collect();
        let s_xtc: f64 = prob_xtc.f(comb) as f64;
        let s_xc: f64 = prob_xc.f_map(&vals) as f64;
        let s_tc: f64 = prob_tc.f_map(&vals) as f64;
        let s_c: f64 = prob_c.f_map(&vals) as f64;
        // println!(
        //     "stat: {}, s_xtc: {}, s_xc: {}, s_tc: {}, s_c: {}",
        //     stat, s_xtc, s_xc, s_tc, s_c
        // );
        if s_xtc != 0.0 {
            // println!("cur1: {}", (s_xtc * s_c / (s_xc * s_tc)));
            // println!("cur2: {}", (s_xtc * s_c / (s_xc * s_tc)).ln());
            if s_xc == 0.0 || s_tc == 0.0 {
                std::panic!("marginals are less frequent?!");
            }
            stat += s_xtc * (s_xtc * s_c / (s_xc * s_tc)).ln();
        }
    }
    stat * 2.0
}

pub fn g2_stat(
    prob: &ProbabilityMap,
    t: usize,
    x: &BTreeSet<usize>,
    cond: &BTreeSet<usize>,
) -> f64 {
    // https://link.springer.com/article/10.1007/s10994-006-6889-7
    // Section 4
    // print!("x: {}, t: {}, cond: [", x.first().unwrap(), t);
    // for c in cond {
    //     print!("{} ", c);
    // }
    // print!("]\n");
    let mut xtc: BTreeSet<usize> = cond.union(x).cloned().collect();
    xtc.insert(t);
    let prob_xtc: ProbabilityMap = prob.marginalize(&xtc);
    let prob_c: ProbabilityMap = prob.marginalize(cond);

    let xc: BTreeSet<usize> = cond.union(x).cloned().collect();
    let prob_xc: ProbabilityMap = prob.marginalize(&xc);

    let mut tc: BTreeSet<usize> = cond.clone();
    tc.insert(t);
    let prob_tc: ProbabilityMap = prob.marginalize(&tc);
    return g2_stat_with_probs(&prob_xtc, &prob_xc, &prob_tc, &prob_c);
}

pub fn g2_df(
    data: &DataSet,
    t: usize,
    x: &BTreeSet<usize>,
    cond: &BTreeSet<usize>,
) -> usize {
    let mut df: usize = 1;
    // print!("x: [");
    // for v in x {
    //     print!("{},", v);
    // }
    // println!("]");
    for v in x {
        // println!("nval for {}: {}", *v, data.nvals[*v]);
        df = df * data.nvals[*v];
    }
    df = if df > 1 {df - 1} else {1};
    df = df * (max(data.nvals[t] - 1, 1));
    for v in cond {
        // println!("nval for {}: {}", *v, data.nvals[*v]);
        df = df * data.nvals[*v];
    }
    return df;
}

pub fn g2(
    prob: &ProbabilityMap,
    t: usize,
    x: &BTreeSet<usize>,
    cond: &BTreeSet<usize>,
) -> TestResult {
    // print!("x: [");
    // for v in x {
    //     print!("{},", v);
    // }
    // println!("]");
    // print!("cond: [");
    // for v in cond {
    //     print!("{},", v);
    // }
    // println!("]");
    // print!("\t computing statistic...");
    let stat = g2_stat(prob, t, x, cond);
    // print!("{}\n", stat);
    // print!("\t computing df...");
    let df = g2_df(prob.get_dataset(), t, x, cond);
    // print!("{}\n", df);
    // print!("\t computing pval...");
    let pval: f64 = chi_square_p_val(stat, df);
    // print!("{}\n", pval);
    return TestResult {
        vars: x.clone(),
        stat,
        df,
        pval,
    };
}
