use std::collections::BTreeSet;

use dataset::DataSet;
use mb::{find_mb_for_var, ResultMB, forward_select, mine, prune};
use prob_map::ProbabilityMap;
use pyo3::prelude::*;

mod dataset;
mod g2test;
mod mb;
mod metadata;
mod prob_map;
mod prob_tids;

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
    println!("Forward Selection...");
    forward_select(&mut res, &prob, att_target, alpha);
    println!("Backward Selection...");
    mine(&mut res, &prob, att_target, k, alpha, use_tid);
    // backward_select(&mut res, prob, att_target, alpha);
    println!("Pruning...");
    prune(&mut res, &prob, att_target, alpha);
    println!("Done!");
    Ok((res.mb, res.num_ci_test))
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
    prune(&mut res, &prob, att_target, alpha);
    println!("Done!");
    Ok((res.mb, res.num_ci_test))
}

/// A Python module implemented in Rust.
#[pymodule]
fn LIMMB(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(learn_mbs, m)?)?;
    m.add_function(wrap_pyfunction!(IAMB, m)?)?;
    Ok(())
}
