use std::collections::{hash_map::Keys, BTreeSet, HashMap, HashSet};

use crate::dataset::{self, DataSet};
use crate::metadata::{self, MetaData};


pub trait Probability<'a> {
    fn new(dataset: &'a DataSet) -> Self;
    fn get_combs(&self) -> Vec<Vec<usize>>;
    fn get_atts(&self) -> &BTreeSet<usize>;
    fn get_size(&self) -> usize;
    fn get_dataset(&self) -> &DataSet;
    fn f(&self, comb: &Vec<usize>) -> usize;
    fn f_map(&self, vals: &HashMap<usize, usize>) -> usize;
    fn p(&self, comb: &Vec<usize>) -> f64;
    fn p_map(&self, vals: &HashMap<usize, usize>) -> f64;
    fn clone(&self) -> Self;
}
