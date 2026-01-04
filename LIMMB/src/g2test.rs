use pyo3::prelude::*;
use rand::seq::SliceRandom;
use rand::{rng, Rng};
use statrs::distribution::{ChiSquared, ContinuousCDF};
use statrs::statistics::Data;
use std::cmp::max;
use std::collections::{BTreeSet, HashMap};
use std::process::Termination;

use crate::dataset::{self, DataSet};
use crate::prob_map::ProbabilityMap;
use crate::probability::Probability;
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

pub fn g2_stat_with_probs<'a>(
    prob_xtc: &impl Probability<'a>,
    prob_xc: &impl Probability<'a>,
    prob_tc: &impl Probability<'a>,
    prob_c: &impl Probability<'a>,
) -> f64 {
    let mut stat: f64 = 0.0;
    let N: usize = prob_xtc.get_dataset().sample_size;
    // print!("tc: [");
    // for a in prob_tc.get_atts() {
    //     print!("{},", a);
    // }
    // println!("]");
    // print!("c: [");
    // for a in prob_c.get_atts() {
    //     print!("{},", a);
    // }
    // println!("]");
    let mut invalid = false;
    let mut num_valid_cells: usize = 0;
    let mut num_cells = 0;
    for comb in prob_xtc.get_combs() {
        let vals: HashMap<usize, usize> = prob_xtc
            .get_atts()
            .iter()
            .enumerate()
            .map(|(i, &a)| (a, comb[i]))
            .collect();
        let s_xtc: f64 = prob_xtc.f(&comb) as f64;
        let s_xc: f64 = prob_xc.f_map(&vals) as f64;
        let s_tc: f64 = prob_tc.f_map(&vals) as f64;
        let s_c: f64 = prob_c.f_map(&vals) as f64;
        // println!(
        //     "stat: {}, s_xtc: {}, s_xc: {}, s_tc: {}, s_c: {}",
        //     stat, s_xtc, s_xc, s_tc, s_c
        // );
        if s_xtc != 0.0 {
            let e: f64 = (N as f64) * (s_xc / s_c) * (s_tc / s_c);
            if e >= 5.0 {
                num_valid_cells += 1;
            }
            num_cells += 1;
            // println!("cur1: {}", (s_xtc * s_c / (s_xc * s_tc)));
            // println!("cur2: {}", (s_xtc * s_c / (s_xc * s_tc)).ln());
            if s_xc == 0.0 || s_tc == 0.0 {
                std::panic!("marginals are less frequent?!");
            }
            stat += s_xtc * (s_xtc * s_c / (s_xc * s_tc)).ln();
        }
    }
    let valid_ratio = (num_valid_cells as f64) / (num_cells as f64);
    print!("\tvalid_ratio={}", valid_ratio);
    if valid_ratio < 0.8 {
        print!("\tWARNING: test might be invalid!!!");
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
    // print!("x: [");
    // for v in x {
    //     print!("{},", v);
    // }
    // println!("]");
    let mut df: usize = 1;
    for v in x {
        // println!("nval for {}: {}", *v, data.nvals[*v]);
        df = df * data.nvals[*v];
    }
    df -= 1;
    df *= data.nvals[t] - 1;
    for v in cond {
        // println!("nval for {}: {}", *v, data.nvals[*v]);
        df = df * data.nvals[*v];
    }
    return df;
}

pub fn g2_df_eff<'a>(
    x: &BTreeSet<usize>,
    t: usize,
    prob_c: &impl Probability<'a>,
) -> usize {
    // Tsamardinos, I., Brown, L. E., & Aliferis, C. F. (2006).
    // Steck, H., & Jaakkola, T. S. (2002).
    let data = prob_c.get_dataset();
    let mut df: usize = 1;
    for v in x {
        // println!("nval for {}: {}", *v, data.nvals[*v]);
        df = df * data.nvals[*v];
    }
    df -= 1;
    df *= data.nvals[t] - 1;
    df *= prob_c.get_size();
    return df;
}

pub fn g2_df_map(
    prob_xtc: &ProbabilityMap,
    prob_xc: &ProbabilityMap,
    prob_tc: &ProbabilityMap,
    prob_c: &ProbabilityMap,
) -> usize {
    return (prob_xc.get_size() - prob_c.get_size())
        + (prob_tc.get_size() - prob_c.get_size());
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
    let mut xtc: BTreeSet<usize> = cond.union(x).cloned().collect();
    xtc.insert(t);
    let prob_xtc: ProbabilityMap = prob.marginalize(&xtc);
    let prob_c: ProbabilityMap = prob.marginalize(cond);

    let xc: BTreeSet<usize> = cond.union(x).cloned().collect();
    let prob_xc: ProbabilityMap = prob.marginalize(&xc);

    let mut tc: BTreeSet<usize> = cond.clone();
    tc.insert(t);
    let prob_tc: ProbabilityMap = prob.marginalize(&tc);
    let stat =
        g2_stat_with_probs(&prob_xtc, &prob_xc, &prob_tc, &prob_c);
    // print!("{}\n", stat);
    // print!("\t computing df...");
    // let df = g2_df_map(&prob_xtc, &prob_xc, &prob_tc, &prob_c);
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
