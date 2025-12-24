use std::collections::BTreeSet;

use crate::prob_tids::ProbabilityTIDs;
use crate::probability::Probability;
use dataset::DataSet;
use liam::{nested_assoc_mine, prune};
use mb::{find_mb_for_var, forward_select, mine, ResultMB};
use prob_map::ProbabilityMap;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::iter::once;

mod ci_tests;
mod dataset;
mod g2test;
mod liam;
mod mb;
mod metadata;
mod prob_map;
mod prob_tids;
mod probability;
mod sci;

#[pyfunction]
fn learn_mbs(
    data: Vec<Vec<usize>>,
    att_target: usize,
    alpha: f64,
    k: usize,
    use_tid: bool,
) -> PyResult<(BTreeSet<usize>, usize)> {
    let dataset: DataSet = DataSet::new(data);
    let prob: ProbabilityMap = ProbabilityMap::new(&dataset);
    let mut res = ResultMB {
        mb: BTreeSet::new(),
        num_ci_test: 0,
    };
    // println!("Forward Selection...");
    forward_select(&mut res, &prob, att_target, alpha);
    println!("Backward Selection...");
    mine(&mut res, &prob, att_target, k, alpha, use_tid);
    // backward_select(&mut res, prob, att_target, alpha);
    println!("Pruning...");
    // prune(&mut res, &prob, att_target, alpha);
    println!("Done!");
    Ok((res.mb, res.num_ci_test))
}

#[pyfunction]
fn run_nested_assoc_mine(
    data: Vec<Vec<usize>>,
    att_target: usize,
    mb: BTreeSet<usize>,
    max_sub_mb_size: usize,
    alpha: f64,
    use_gtest: bool,
) -> PyResult<(BTreeSet<usize>, usize)> {
    let mut cmb: BTreeSet<usize> = mb.clone();
    let mut cmb_groups: Vec<BTreeSet<usize>> = Vec::new();
    let mut mb_var_to_group: HashMap<usize, BTreeSet<usize>> =
        HashMap::new();
    let dataset: DataSet = DataSet::new(data);
    let all_atts: BTreeSet<usize> = (0..dataset.natts).collect();
    let atom_probs: Vec<ProbabilityTIDs> = (0..dataset.natts).map(|a| ProbabilityTIDs::new_marg(&dataset, BTreeSet::from([a]))).collect();
    println!("Runnig Miner...");
    let mut num_ci: usize = 0;
    let mut known_xs_deps: HashMap<BTreeSet<usize>, Vec<BTreeSet<usize>>> = HashMap::new();
    let mut atts: BTreeSet<usize> =
        all_atts.difference(&mb).cloned().collect();
    atts.remove(&att_target);
    let mut it: usize = 0;
    let mut converged: bool = false;
    while !converged && !atts.is_empty() {
        // num_ci += prune(&mut cmb, &prob, att_target, alpha, df_limit);
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
        let res = nested_assoc_mine(
            &cmb,
            &atom_probs,
            &atts,
            att_target,
            &mut known_xs_deps,
            max_sub_mb_size,
            alpha,
            use_gtest,
        );
        match res {
            Some(r) => {
                num_ci += r.num_ci_test;
                let mut pruned_res_mb = r.mb.clone();
                num_ci += prune(
                    &mut pruned_res_mb,
                    &atom_probs,
                    att_target,
                    &mut known_xs_deps,
                    alpha,
                    max_sub_mb_size,
                    use_gtest,
                );
                if cmb == pruned_res_mb {
                    converged = true;
                }
                for y in pruned_res_mb.difference(&cmb) {
                    atts.remove(&y);
                }
                cmb = pruned_res_mb;
                if r.mb != cmb {
                    num_ci += prune(
                        &mut cmb,
                        &atom_probs,
                        att_target,
                        &mut known_xs_deps,
                        alpha,
                        max_sub_mb_size,
                        use_gtest,
                    );
                }
            }
            None => {
                converged = true;
            }
        }
    }
    // num_ci += prune(&mut cmb, &prob, att_target, alpha, df_limit);
    print!(
        "\nfinal: mb size: {} \t att rem: {}\t mb: [",
        cmb.len(),
        atts.len()
    );
    for a in cmb.iter() {
        print!("{},", a);
    }
    println!("]");
    Ok((cmb, num_ci))
}

#[pyfunction]
fn IAMB(
    data: Vec<Vec<usize>>,
    att_target: usize,
    alpha: f64,
) -> PyResult<(BTreeSet<usize>, usize)> {
    let dataset: DataSet = DataSet::new(data);
    let prob: ProbabilityMap = ProbabilityMap::new(&dataset);
    let mut res = ResultMB {
        mb: BTreeSet::new(),
        num_ci_test: 0,
    };
    println!("Forward Selection...");
    forward_select(&mut res, &prob, att_target, alpha);
    println!("Pruning...");
    // prune(&mut res, &prob, att_target, alpha);
    println!("Done!");
    Ok((res.mb, res.num_ci_test))
}

/// A Python module implemented in Rust.
#[pymodule]
fn LIMMB(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(learn_mbs, m)?)?;
    m.add_function(wrap_pyfunction!(run_nested_assoc_mine, m)?)?;
    m.add_function(wrap_pyfunction!(IAMB, m)?)?;
    Ok(())
}
