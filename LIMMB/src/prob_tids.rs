use std::collections::{hash_map::Keys, BTreeSet, HashMap};

use bit_set::BitSet;

use crate::dataset::DataSet;
use crate::probability::Probability;


pub struct ProbabilityTIDs<'a> {
    dataset: &'a DataSet,
    atts: BTreeSet<usize>,
    comb_to_tid: HashMap<Vec<usize>, BitSet>,
}

impl<'a> Probability<'a> for ProbabilityTIDs<'a> {
    fn new(dataset: &'a DataSet) -> Self {
        let atts: BTreeSet<usize> = (0..dataset.natts).collect();
        return ProbabilityTIDs::new_marg(dataset, atts);
    }
    fn get_combs(&self) -> Vec<Vec<usize>> {
        return self.comb_to_tid.keys().cloned().collect();
    }
    fn get_atts(&self) -> &BTreeSet<usize> {
        return &self.atts;
    }
    fn get_dataset(&self) -> &DataSet {
        return &self.dataset;
    }
    fn get_size(&self) -> usize {
        return self.comb_to_tid.len();
    }

    fn p(&self, comb: &Vec<usize>) -> f64 {
        match self.comb_to_tid.get(comb) {
            Some(v) => {
                (v.len() as f64) / (self.dataset.sample_size as f64)
            }
            None => 0.0,
        }
    }

    fn f(&self, comb: &Vec<usize>) -> usize {
        match self.comb_to_tid.get(comb) {
            Some(v) => v.len(),
            None => 0,
        }
    }

    fn f_map(&self, vals: &HashMap<usize, usize>) -> usize {
        let comb: Vec<usize> =
            self.atts.iter().map(|a| vals[a]).collect();
        self.f(&comb)
    }

}

impl<'a> ProbabilityTIDs<'a> {

    pub fn new_marg(
        dataset: &'a DataSet,
        vars: BTreeSet<usize>,
    ) -> Self {
        let mut comb_to_tid: HashMap<Vec<usize>, BitSet> =
            HashMap::new();
        let n = dataset.data.len();

        for (i, row) in dataset.data.iter().enumerate() {
            let sub_vals: Vec<usize> =
                vars.iter().map(|&a| row[a]).collect();
            match comb_to_tid.get_mut(&sub_vals) {
                Some(tid) => {
                    tid.insert(i);
                }
                None => {
                    let mut bs: BitSet = BitSet::with_capacity(n);
                    bs.insert(i);
                    comb_to_tid.insert(sub_vals, bs);
                }
            }
        }
        return Self {
            dataset,
            atts: vars.into_iter().collect(),
            comb_to_tid,
        };
    }

    pub fn merge(&self, other: &ProbabilityTIDs) -> Self {
        let new_atts: BTreeSet<usize> =
            self.atts.union(&other.atts).cloned().collect();
        let mut new_comb_to_tid: HashMap<Vec<usize>, BitSet> =
            HashMap::with_capacity(
                self.comb_to_tid.len() + other.comb_to_tid.len(),
            );
        let mut tmp_vals: HashMap<usize, usize> =
            HashMap::with_capacity(new_atts.len());
        for (comb1, tid1) in self.comb_to_tid.iter() {
            for (i, a) in self.atts.iter().enumerate() {
                tmp_vals.insert(*a, comb1[i]);
            }
            for (comb2, tid2) in other.comb_to_tid.iter() {
                let mut skip = false;
                for (i, a) in other.atts.iter().enumerate() {
                    let v = comb2[i];
                    if (self.atts.contains(a)) && (tmp_vals[a] != v) {
                        skip = true;
                        break;
                    }
                    tmp_vals.insert(*a, v);
                }
                if skip {
                    continue;
                }
                let tmp_val: Vec<usize> =
                    new_atts.iter().map(|a| tmp_vals[a]).collect();
                let new_tid: BitSet =
                    tid1.intersection(&tid2).collect();
                if new_tid.len() > 0 {
                    new_comb_to_tid.insert(tmp_val, new_tid);
                }
            }
            tmp_vals.clear();
        }
        Self {
            dataset: self.dataset,
            atts: new_atts,
            comb_to_tid: new_comb_to_tid,
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            dataset: self.dataset,
            atts: self.atts.clone(),
            comb_to_tid: self.comb_to_tid.clone(),
        }
    }


}
