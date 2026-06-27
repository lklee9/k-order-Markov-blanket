use std::collections::{hash_map::Keys, BTreeSet, HashMap};

use crate::dataset::{DataSet};
use crate::prob_tids::ProbabilityTIDs;
use crate::probability::Probability;

pub struct ProbabilityMap<'a> {
    pub dataset: &'a DataSet,
    pub atts: BTreeSet<usize>,
    pub comb_to_freq: HashMap<Vec<usize>, usize>,
}

impl<'a> Probability<'a> for ProbabilityMap<'a> {
    fn new(dataset: &'a DataSet) -> Self {
        let atts: BTreeSet<usize> = (0..dataset.natts).collect();
        return ProbabilityMap::new_marg(dataset, atts);
    }
    fn get_combs(&self) -> Vec<Vec<usize>> {
        return self.comb_to_freq.keys().cloned().collect();
    }
    fn get_atts(&self) -> &BTreeSet<usize> {
        return &self.atts;
    }
    
    fn get_dataset(&self) -> &DataSet {
        return &self.dataset;
    }
    
    fn get_size(&self) -> usize {
        return self.comb_to_freq.len();
    }
    
    fn p(&self, comb: &Vec<usize>) -> f64 {
        match self.comb_to_freq.get(comb) {
            Some(&v) => (v as f64) / (self.dataset.sample_size as f64),
            None => 0.0,
        }
    }
    
    fn p_map(&self, vals: &HashMap<usize, usize>) -> f64 {
        let comb: Vec<usize> =
            self.atts.iter().map(|a| vals[a]).collect();
        self.p(&comb)
    }

    fn f(&self, comb: &Vec<usize>) -> usize {
        match self.comb_to_freq.get(comb) {
            Some(&v) => v,
            None => 0,
        }
    }

    fn f_map(&self, vals: &HashMap<usize, usize>) -> usize {
        let comb: Vec<usize> =
            self.atts.iter().map(|a| vals[a]).collect();
        self.f(&comb)
    }
    
    fn clone(&self) -> Self {
        Self {
            dataset: self.dataset,
            atts: self.atts.clone(),
            comb_to_freq: self.comb_to_freq.clone(),
        }
    }
}

impl<'a> ProbabilityMap<'a> {
    pub fn new_marg(dataset: &'a DataSet, vars: BTreeSet<usize>) -> Self {
        let mut comb_to_freq: HashMap<Vec<usize>, usize> =
            HashMap::new();

        for row in &dataset.data {
            let sub_vals: Vec<usize> =
                vars.iter().map(|&a| row[a]).collect();
            match comb_to_freq.get(&sub_vals) {
                Some(f) => {
                    comb_to_freq.insert(sub_vals, f + 1);
                }
                None => {
                    comb_to_freq.insert(sub_vals, 1);
                }
            }
        }
        return Self {
            dataset,
            atts: vars.into_iter().collect(),
            comb_to_freq,
        };
    }

    pub fn marginalize(&self, marg: &BTreeSet<usize>) -> Self {
        let mut comb_to_freq_m: HashMap<Vec<usize>, usize> =
            HashMap::new();
        let att_to_id: HashMap<usize, usize> = self
            .atts
            .iter()
            .enumerate()
            .map(|(i, &a)| (a, i))
            .collect();
        let mut sub_vals: Vec<usize> = Vec::with_capacity(marg.len());
        for _ in 0..marg.len() {
            sub_vals.push(0);
        }
        for (all_vals, freq) in self.comb_to_freq.iter() {
            for (i, a) in marg.iter().enumerate() {
                sub_vals[i] = all_vals[att_to_id[a]];
            }
            match comb_to_freq_m.get(&sub_vals) {
                Some(f) => {
                    comb_to_freq_m.insert(sub_vals.clone(), f + *freq);
                }
                None => {
                    comb_to_freq_m.insert(sub_vals.clone(), *freq);
                }
            }
        }
        return Self {
            dataset: self.dataset,
            atts: marg.clone(),
            comb_to_freq: comb_to_freq_m,
        };
    }
    pub fn marginalize_vec(&self, marg: &Vec<usize>) -> Self {
        return self.marginalize(&marg.iter().cloned().collect());
    }

    pub fn remove_att(&self, rem_att: usize) -> Self {
        let mut new_atts = self.atts.clone();
        new_atts.remove(&rem_att);
        return self.marginalize(&new_atts);
    }

    pub fn predict(
        &self,
        condition: &mut HashMap<usize, usize>,
        target_att: usize,
    ) -> usize {
        let mut max_freq: usize = 0;
        let mut max_val: usize = 0;
        for i in 0..self.dataset.nvals[target_att] {
            condition.insert(target_att, i);
            let cur_comb: Vec<usize> =
                self.atts.iter().map(|a| condition[a]).collect();
            match self.comb_to_freq.get(&cur_comb) {
                Some(f) => {
                    if *f > max_freq {
                        max_freq = *f;
                        max_val = i;
                    }
                }
                None => {
                    continue;
                }
            }
        }
        return max_val;
    }
}


impl<'a> From<ProbabilityTIDs<'a>> for ProbabilityMap<'a> {
    fn from(prob_tids: ProbabilityTIDs<'a>) -> Self {
        let mut comb_to_freq: HashMap<Vec<usize>, usize> = HashMap::new();
        for (comb, tid) in prob_tids.comb_to_tid.iter() {
            comb_to_freq.insert(comb.clone(), tid.len());
        }
        ProbabilityMap {
            dataset: prob_tids.dataset,
            atts: prob_tids.atts.clone(),
            comb_to_freq: comb_to_freq,
        }
    }
}
