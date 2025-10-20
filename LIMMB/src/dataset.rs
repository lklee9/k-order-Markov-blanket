use rand::{rng, Rng};
use std::collections::{BTreeSet, HashMap, HashSet};

pub struct DataSet {
    pub data: Vec<Vec<usize>>,
    pub natts: usize,
    pub att_val_to_idx: Vec<HashMap<usize, usize>>,
    pub nvals: Vec<usize>,
    pub sample_size: usize,
}

fn gen_random_sample_indices(n: usize, size: usize) -> Vec<usize> {
    if size > n {
        panic!("Sample size is greater than dataset size!");
    }
    let mut rng = rng();
    let mut indices: Vec<usize> = (0..n).collect();
    // Perform the Fisher-Yates shuffle, but only partially
    for i in 0..size {
        let j = rng.random_range(i..n);
        indices.swap(i, j);
    }
    return indices[0..size].to_vec();
}

impl DataSet {
    pub fn new(data: Vec<Vec<usize>>) -> Self {
        let natts = data[0].len();
        let mut att_val_to_idx: Vec<HashMap<usize, usize>> =
            (0..natts).map(|_| HashMap::new()).collect();
        let sample_size = data.len();
        let mut nvals: Vec<usize> = vec![0; natts];
        for row in &data {
            for i in 0..row.len() {
                if att_val_to_idx[i].contains_key(&row[i]) {
                    continue;
                }
                let cur_idx = att_val_to_idx[i].len();
                att_val_to_idx[i].insert(row[i], cur_idx);
                nvals[i] = cur_idx + 1;
            }
        }
        print!("nvals: [");
        for n in nvals.iter() {
            print!("{},", n);
        }
        print!("]\n");
        Self {
            data,
            natts,
            att_val_to_idx,
            nvals,
            sample_size,
        }
    }

    pub fn get_comb_id(
        &self,
        atts: &Vec<usize>,
        vals: &Vec<usize>,
    ) -> usize {
        let mut base: usize = 1;
        let mut hash: usize = 0;
        let mut i: usize = 0;
        for &att in atts {
            hash += self.att_val_to_idx[att][&vals[i]] * base;
            println!("base: {}, nvals: {}", base, self.nvals[att]);
            base *= self.nvals[att];
            i += 1;
        }
        return hash;
    }

    pub fn get_vals(
        &self,
        atts: &BTreeSet<usize>,
        comb_id: usize,
    ) -> HashMap<usize, usize> {
        let att_ord: Vec<usize> = atts.iter().cloned().collect();
        let mut vals_ord: Vec<usize> = Vec::with_capacity(atts.len());
        for _ in 0..atts.len() {
            vals_ord.push(0);
        }
        let mut rem = comb_id;
        let mut base =
            atts.iter().fold(1, |acc, a| acc * self.nvals[*a]);
        let mut i = atts.len();
        while i > 0 {
            i -= 1;
            let a = att_ord[i];
            // println!("cur_base: {}, nval: {}", base, self.nvals[a]);
            base = base / self.nvals[a];
            // println!("rem: {}, base: {}, i: {}", rem, base, i);
            vals_ord[i] = rem / base;
            // vals.insert(a, rem / base);
            rem = rem % base;
        }
        let mut vals: HashMap<usize, usize> =
            HashMap::with_capacity(atts.len());
        for i in 0..atts.len() {
            vals.insert(att_ord[i], vals_ord[i]);
        }
        // println!("vals size: {}", vals.len());
        // let mut vals: HashMap<usize, usize> = HashMap::new();
        return vals;
    }

    pub fn get_vals_vec(
        &self,
        atts: &Vec<usize>,
        comb_id: usize,
    ) -> Vec<usize> {
        let mut vals: Vec<usize> = Vec::with_capacity(atts.len());
        for _ in 0..atts.len() {
            vals.push(0);
        }
        let mut rem = comb_id;
        let mut base =
            atts.iter().fold(1, |acc, a| acc * self.nvals[*a]);
        let mut i = atts.len();
        while i > 0 {
            i -= 1;
            let a = atts[i];
            // println!("cur_base: {}, nval: {}", base, self.nvals[a]);
            base = base / self.nvals[a];
            // println!("rem: {}, base: {}, i: {}", rem, base, i);
            vals[i] = rem / base;
            // vals.insert(a, rem / base);
            rem = rem % base;
        }
        return vals;
    }
    pub fn get_comb_id_map(
        &self,
        att_subset: &BTreeSet<usize>,
        vals: &HashMap<usize, usize>,
    ) -> usize {
        let atts: Vec<usize> = att_subset.iter().cloned().collect();
        let vals_v: Vec<usize> = atts.iter().map(|a| vals[a]).collect();
        return self.get_comb_id(&atts, &vals_v);
    }

    pub fn get_comb_id_vec(
        &self,
        att_subset: &Vec<usize>,
        vals: &Vec<usize>,
    ) -> usize {
        let atts: Vec<usize> = att_subset.iter().cloned().collect();
        let vals: Vec<usize> = atts.iter().map(|a| vals[*a]).collect();
        return self.get_comb_id(&atts, &vals);
    }

    pub fn gen_random_sample(&self, size: usize) -> Vec<&Vec<usize>> {
        let random_indices =
            gen_random_sample_indices(self.sample_size, size);
        let mut sample: Vec<&Vec<usize>> = Vec::new();
        for i in random_indices {
            sample.push(&self.data[i]);
        }
        return sample;
    }
}
