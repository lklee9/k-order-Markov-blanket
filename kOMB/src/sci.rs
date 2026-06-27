use std::collections::{BTreeSet, HashMap, HashSet};

use crate::probability::Probability;

pub fn regret(card: usize, samp_size: usize) -> f64 {
    // Algorithm 2 in Mononen, T., & Myllymäki, P. (2008)
    if card < 1 {
        return 0.0;
    }
    let precision: f64 = 10.0;
    // let bound: usize = (
    //     2.0 + (
    //         -2.0
    //             * (samp_size as f64)
    //             * (
    //                 (2.0*(10.0f64).powf(precision) - 1.0).ln() -
    //                     2.0*precision*(10.0f64).ln()
    //             )
    //     ).sqrt()
    // ).ceil() as usize;
    let bound: usize = (
        2.0 + (
            2.0
                * (samp_size as f64) * 10.0 * (10.0f64).ln()
        ).sqrt()
    ).ceil() as usize;
    let mut b: f64 = 1.0;
    let mut sum: f64 = 1.0;
    // println!("\t\t\tbound: {}", bound);
    for k in 1..(bound + 1) {
        // println!("\t\t\t\tb: {}", b);
        b = (samp_size as f64 - k as f64 + 1.0) * (b / samp_size as f64);
        sum += b;
    }

    let mut old_sum: f64 = 1.0;
    let mut new_sum: f64 = 1.0;
    for j in 3..(card + 1) {
        new_sum = sum + (samp_size as f64 * old_sum) / (j - 2) as f64;
        old_sum = sum;
        sum = new_sum;
    }
    if sum.is_nan() {
        panic!["regret is NaN!!!!!"]
    }
    return sum;
}

fn cond_sc<'a>(
    xs_card: usize,
    prob_joint: &impl Probability<'a>,
    prob_cond: &impl Probability<'a>,
) -> f64 {
    let joint_vars: Vec<usize> =
        prob_joint.get_atts().iter().cloned().collect();
    let mut res: f64 = 0.0;
    // Shannon entropy
    for comb in prob_joint.get_combs() {
        let comb_map: HashMap<usize, usize> = (0..joint_vars.len())
            .map(|i| (joint_vars[i], comb[i]))
            .collect();
        let f = prob_cond.f_map(&comb_map) as f64;
        let p = (prob_joint.f(&comb) as f64) / f;
        res -= f * (p * p.log2());
    }
    // regret
    let mut total_regret: f64 = 0.0;
    let card: usize = prob_joint
        .get_atts()
        .difference(prob_cond.get_atts())
        .into_iter()
        .map(|a| prob_joint.get_dataset().nvals[*a])
        .fold(1, |acc, e| acc * e);
    for cond in prob_cond.get_combs() {
        let tmp_regret = regret(card, prob_cond.f(&cond));
        if tmp_regret > 0.0 {
            total_regret += tmp_regret.log2();
        }
    }
    // print!("\tentropy: {}, regret: {}", res, total_regret);
    return res + total_regret;
}

pub fn cond_sci_stat<'a>(
    prob_xtc: &impl Probability<'a>,
    prob_xc: &impl Probability<'a>,
    prob_tc: &impl Probability<'a>,
    prob_c: &impl Probability<'a>,
) -> f64 {
    let mut xs: HashSet<Vec<usize>> = HashSet::new();
    let att_x: BTreeSet<usize> = prob_xc.get_atts().difference(prob_c.get_atts()).cloned().collect();
    let att_xc: Vec<usize>  = prob_xc.get_atts().iter().cloned().collect();
    for comb in prob_xc.get_combs() {
        let comb_map: HashMap<usize, usize> = (0..att_xc.len())
            .map(|i| (att_xc[i], comb[i]))
            .collect();
        let x: Vec<usize> = att_x.iter().map(|x| comb_map[x]).collect();
        xs.insert(x);
    }
    let card = xs.len();
    // test if x and t are cond indep given c
    return cond_sc(card, prob_xc, prob_c) - cond_sc(card, prob_xtc, prob_tc);
}
