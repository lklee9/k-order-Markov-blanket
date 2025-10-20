use std::collections::{BTreeSet, HashMap, HashSet};

pub struct MetaData {
    pub natts: usize,
    pub nvals: Vec<usize>,
    pub sample_size: usize,
}

impl MetaData {
    pub fn new(data: &Vec<Vec<usize>>) -> Self {
        let natts = data[0].len();
        let sample_size = data.len();
        let mut nvals: Vec<usize> = vec![0; natts];
        for row in data {
            for i in 0..row.len() {
                nvals[i] = std::cmp::max(nvals[i], row[i] + 1);
            }
        }
        Self {
            natts,
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
        for att in atts {
            hash += vals[i] * base;
            base *= self.nvals[*att];
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
        let mut vals: HashMap<usize, usize> = HashMap::new();
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
            vals.insert(a, rem / base);
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

}
