use std::collections::BTreeSet;

use crate::prob_tids::ProbabilityTIDs;
use crate::probability::Probability;
use dataset::DataSet;
use liam::{prune, nested_assoc_mine};
use mb::{find_mb_for_var, forward_select, mine, ResultMB};
use prob_map::ProbabilityMap;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::iter::once;

mod dataset;
mod g2test;
mod liam;
mod mb;
mod metadata;
mod prob_map;
mod prob_tids;
mod probability;

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
    alpha: f64,
) -> PyResult<(BTreeSet<usize>, usize)> {
    let mut cmb: BTreeSet<usize> = mb.clone();
    let dataset: DataSet = DataSet::new(data);
    let df_limit: usize = dataset.sample_size / 5;
    let prob: ProbabilityMap = ProbabilityMap::new(&dataset);
    print!("prob: [");
    for a in prob.get_atts() {
        print!("{},", a);
    }
    println!("]");
    println!("Runnig Miner...");
    let mut atts: BTreeSet<usize> =
        prob.get_atts().difference(&mb).cloned().collect();
    atts.remove(&att_target);
    let mut num_ci: usize = 0;
    let mut atts: BTreeSet<usize> =
        prob.get_atts().difference(&mb).cloned().collect();
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
            &prob,
            &atts,
            att_target,
            3,
            alpha,
        );
        match res {
            Some(r) => {
                num_ci += r.num_ci_test;
                for y in r.mb {
                    cmb.insert(y);
                    atts.remove(&y);
                }
            }
            None => {
                converged = true;
            }
        }
        num_ci += prune(&mut cmb, &prob, att_target, alpha, df_limit);
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
