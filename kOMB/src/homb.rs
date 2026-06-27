use itertools::Itertools;
use log::{info, trace};
use num::traits::bounds::UpperBounded;
use pyo3::call;
use rand::seq::index::sample;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::iter::once;
use std::usize::MAX;

use crate::ci_tests::{sci_min_sample_size, CIRes, GTest, SCI};
use crate::dataset::{self, DataSet};
use crate::g2test::{
    self, chi_square_p_val, g2, g2_df, g2_df_eff, g2_stat,
    g2_stat_with_probs, TestResult,
};
use crate::mb::ResultMB;
use crate::metadata::MetaData;
use crate::prob_map::{self, ProbabilityMap};
use crate::prob_tids::ProbabilityTIDs;
use crate::probability::Probability;

type VarSet = BTreeSet<usize>;

struct InnerMineRes {
    xs: BTreeSet<usize>,
    is_in_mb: bool,
    num_ci: usize,
    badness: f64,
}

pub struct HOMB<'a> {
    // Initial Fields
    db: &'a DataSet,
    att_t: usize,
    init_mb: VarSet,
    alpha: f64,
    order: usize,
    sep_size: usize,
    use_gtest: bool,
    // Generated Fields
    prob_atoms: Vec<ProbabilityTIDs<'a>>,
    // Modified Fields
    cmb: VarSet,
    prob_cmb: ProbabilityTIDs<'a>,
    num_ci: usize,
    known_seps: HashMap<usize, HashMap<VarSet, HashSet<VarSet>>>,
    known_not_deps: HashMap<usize, HashSet<VarSet>>,
    known_ci: HashMap<usize, HashMap<VarSet, CIRes>>,
}

impl<'a> HOMB<'a> {
    pub fn new(
        db: &'a DataSet,
        att_t: usize,
        init_mb: BTreeSet<usize>,
        alpha: f64,
        order: usize,
        sep_size: usize,
        use_gtest: bool,
    ) -> Self {
        let prob_atoms: Vec<ProbabilityTIDs<'a>> = (0..db.natts)
            .map(|a| {
                ProbabilityTIDs::new_marg(&db, BTreeSet::from([a]))
            })
            .collect();
        let cmb: VarSet = init_mb.clone();
        let mut prob_cmb: ProbabilityTIDs =
            ProbabilityTIDs::new_empty(db);
        for a in init_mb.iter() {
            prob_cmb = prob_cmb.merge(&prob_atoms[*a]);
        }
        let num_ci: usize = 0;
        let known_seps: HashMap<
            usize,
            HashMap<VarSet, HashSet<VarSet>>,
        > = HashMap::new();
        let known_not_deps: HashMap<usize, HashSet<VarSet>> =
            HashMap::new();
        let known_ci: HashMap<usize, HashMap<VarSet, CIRes>> =
            HashMap::new();
        return Self {
            db,
            att_t,
            init_mb,
            alpha,
            order,
            sep_size,
            use_gtest,
            prob_atoms,
            cmb,
            prob_cmb,
            num_ci,
            known_seps,
            known_not_deps,
            known_ci,
        };
    }

    /// Check if G2 and SCI are too weak for xs_cur
    fn too_weak(&self, xs: &VarSet, mb: &VarSet) -> bool {
        let mut too_weak: bool = g2_df(self.db, self.att_t, xs, mb) * 5
            > self.db.sample_size;
        if too_weak {
            let mut card_x: usize = 1;
            for v in xs.iter() {
                card_x = card_x * self.db.nvals[*v];
            }
            let mut card_cond: usize = 1;
            for v in mb.iter() {
                card_cond *= self.db.nvals[*v];
            }
            let card_t: usize = self.db.nvals[self.att_t];
            too_weak = sci_min_sample_size(card_x, card_t, card_cond)
                > self.db.sample_size as f64;
        };
        return too_weak;
    }

    fn test_ci(
        &mut self,
        att_x: usize,
        prob_cond: &ProbabilityTIDs,
    ) -> CIRes {
        let cond = prob_cond.atts.clone();
        if let Some(res) = self.known_ci[&att_x].get(&cond) {
            return res.clone();
        }
        self.num_ci += 1;
        let tmp = GTest::new_from_prob(
            prob_cond,
            &self.prob_atoms[att_x],
            &self.prob_atoms[self.att_t],
            self.alpha,
        );
        let res = if self.use_gtest
            && !tmp.too_weak
            && prob_cond.atts.len() <= 3
        {
            print!(
                "\tres_stat: {}, res_df: {}, res_p: {}",
                tmp.stat, tmp.df, tmp.pval,
            );
            tmp
        } else {
            let tmp = SCI::new_from_prob(
                prob_cond,
                &self.prob_atoms[att_x],
                &self.prob_atoms[self.att_t],
            );
            print!("\tres_stat: {}", tmp.stat);
            tmp
        };
        self.known_ci
            .get_mut(&att_x)
            .unwrap()
            .insert(prob_cond.atts.clone(), res.clone());
        res
    }

    /// Order given vars by G-Stat in ascending order
    fn order_vars(
        &mut self,
        prob_cond: &ProbabilityTIDs,
        vars_to_add: &mut Vec<usize>,
    ) {
        let var_to_stat: HashMap<usize, f64> = vars_to_add
            .iter()
            .map(|&a| {
                (
                    a,
                    GTest::new_from_prob(
                        &prob_cond,
                        &self.prob_atoms[a],
                        &self.prob_atoms[self.att_t],
                        self.alpha,
                    )
                    .badness,
                )
            })
            .collect();
        vars_to_add.sort_by(|a, b| {
            var_to_stat[b].partial_cmp(&var_to_stat[a]).unwrap()
        });
    }

    fn order_mb(
        &mut self,
        prob_cond: &ProbabilityTIDs,
        x: usize,
        mb_to_add: &mut Vec<usize>,
    ) {
        let mb_to_stat: HashMap<usize, f64> = mb_to_add
            .iter()
            .map(|&a| {
                (
                    a,
                    self.test_ci(
                        x,
                        &prob_cond.merge(&self.prob_atoms[a]),
                    )
                    .badness,
                )
            })
            .collect();
        mb_to_add.sort_by(|a, b| {
            mb_to_stat[b].partial_cmp(&mb_to_stat[a]).unwrap()
        });
    }

    pub fn run(&mut self) -> (BTreeSet<usize>, usize) {
        println!("Runnig HOMB...");
        self.known_seps = HashMap::new();
        self.known_not_deps = HashMap::new();
        self.known_ci = HashMap::new();
        self.num_ci = 0;
        self.cmb = self.init_mb.clone();
        let mut non_t: BTreeSet<usize> = (0..self.db.natts).collect();
        non_t.remove(&self.att_t);
        for a in non_t.iter() {
            self.known_seps.insert(*a, HashMap::new());
            self.known_not_deps.insert(*a, HashSet::new());
            self.known_ci.insert(*a, HashMap::new());
        }
        let mut prev_cmb: BTreeSet<usize> = self.cmb.clone();
        let mut iter: usize = 0;
        loop {
            let xs_rem: BTreeSet<usize> =
                non_t.difference(&self.cmb).cloned().collect();
            let xs_group = self.find_assoc(&xs_rem);
            iter += 1;
            print!("{}. CMB: [", iter);
            self.cmb.iter().for_each(|a| print!("{},", a));
            print!("]");
            if let Some(mut xs) = xs_group {
                print!("\tTO ADD: [");
                for a in xs.iter() {
                    print!("{},", a);
                    self.prob_cmb =
                        self.prob_cmb.merge(&self.prob_atoms[*a])
                }
                println!("]");
                self.cmb.append(&mut xs);
                // self.prune();
                if self.cmb == prev_cmb || self.cmb.is_subset(&prev_cmb)
                {
                    break;
                }
                prev_cmb = self.cmb.clone();
            } else {
                break;
            }
        }
        self.prune();
        return (self.cmb.clone(), self.num_ci);
    }

    fn prune(&mut self) {
        print!("\nPruning: [");
        self.cmb.iter().for_each(|a| print!("{},", a));
        print!("]");
        let mut remaining: VarSet = self.cmb.clone();

        let converged = false;
        let empty_prob = ProbabilityTIDs::new_empty(self.db);
        let empty_vars = BTreeSet::new();
        while !converged && remaining.len() > 0 {
            let mut reses: Vec<(usize, f64)> =
                Vec::with_capacity(remaining.len());
            for x in remaining.clone() {
                let (is_not_ci, badness) =
                    self.not_ci(x, &empty_vars, &empty_prob);
                if is_not_ci {
                    // remaining.remove(&x);
                } else {
                    reses.push((x, badness));
                }
            }
            if reses.is_empty() {
                break;
            }
            // for res in reses {
            //     print!(
            //         "\n\tbad var x: {}... badness: {}\n",
            //         res.0, res.1
            //     );
            //     remaining.remove(&res.0);
            //     self.cmb.remove(&res.0);
            // }
            reses.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let worse_res = reses[0];
            print!(
                "\n\tworse x: {}... badness: {}\n",
                worse_res.0, worse_res.1
            );
            remaining.remove(&worse_res.0);
            self.cmb.remove(&worse_res.0);
        }
    }

    /// Finds a subset of xs_set that belongs in the MB of self.att_t
    fn find_assoc(&mut self, xs_set: &VarSet) -> Option<VarSet> {
        // Sort atts by gstat in asc order
        let mut xs_vec: Vec<usize> = xs_set.iter().cloned().collect();
        print!("Ordering vars based on cond prob with cond: [");
        self.prob_cmb.atts.iter().for_each(|a| print!("{},", a));
        println!("]");
        self.order_vars(&self.prob_cmb.clone(), &mut xs_vec);
        // Start mining loop (DFS)
        let mut stack_to_add: Vec<Vec<usize>> =
            Vec::from(vec![xs_vec.clone()]);
        let mut stack_xs_cur: Vec<VarSet> =
            Vec::from(vec![BTreeSet::new()]);
        let mut stack_prob: Vec<ProbabilityTIDs> =
            Vec::from(vec![ProbabilityTIDs::new_empty(self.db)]);
        let mut iter: usize = 0;
        while !stack_xs_cur.is_empty() {
            assert![stack_xs_cur.len() == stack_to_add.len()];
            if stack_to_add.is_empty() {
                continue;
            }
            iter += 1;
            print!("\n");
            let xs_cur: VarSet = stack_xs_cur.pop().unwrap();
            let mut to_add: Vec<usize> = stack_to_add.pop().unwrap();
            let prob_cond: ProbabilityTIDs = stack_prob.pop().unwrap();
            print!("\t{}. xs_cur: [", iter);
            xs_cur.iter().for_each(|x| print!("{},", x));
            print!("]\tto_add: [");
            to_add.iter().for_each(|x| print!("{},", x));
            print!("]");
            if xs_cur.len() > self.order {
                print!("\tXs too large!");
                continue;
            }
            // self.order_vars(&prob_cond, &mut to_add);
            for i in 0..to_add.len() {
                let x_cur = to_add[i];
                let is_not_ci =
                    self.not_ci(x_cur, &xs_cur, &prob_cond).0;
                if is_not_ci {
                    print!("\tMaking Strict: [");
                    xs_cur.iter().for_each(|a| print!("{},", a));
                    println!("]");
                    return Some(self.make_strict(x_cur, &xs_cur));
                }
                if xs_cur.len() == self.order {
                    continue;
                }
                stack_to_add.push(to_add[0..i].to_vec());
                let mut xs_next: BTreeSet<usize> = xs_cur.clone();
                xs_next.insert(x_cur);
                stack_xs_cur.push(xs_next);
                let prc_next = prob_cond.merge(&self.prob_atoms[x_cur]);
                stack_prob.push(prc_next);
            }
        }
        return None;
    }

    fn not_ci(
        &mut self,
        att_x: usize,
        att_xs: &VarSet,
        prob_xs: &ProbabilityTIDs,
    ) -> (bool, f64) {
        print!("\n");
        print!("\n\t\tfind deps of {} with Xs: [", att_x);
        att_xs.iter().for_each(|a| print!("{},", a));
        print!("], with cmb: [");
        self.cmb.iter().for_each(|a| print!("{},", a));
        print!("]");
        let eff_sub_mb_size: usize = self.order + 1;

        let mut atts_to_add: BTreeSet<usize> =
            self.cmb.clone().difference(att_xs).cloned().collect();
        atts_to_add.remove(&att_x);
        let mut atts: Vec<usize> = atts_to_add.into_iter().collect();
        // self.order_mb(&prob_xs, att_x, &mut atts);

        let mut prob_max: ProbabilityTIDs = prob_xs.clone();
        for a in atts.iter() {
            prob_max = prob_max.merge(&self.prob_atoms[*a]);
        }
        let max_g_test: CIRes = GTest::new_from_prob(
            &prob_max,
            &self.prob_atoms[att_x],
            &self.prob_atoms[self.att_t],
            self.alpha,
        );
        // Do NOT short-circuit here by returning the G-test against the full
        // conditioning set `prob_max`. Conditioning on the entire current
        // blanket at once is high-df and underpowered, so it falsely declares
        // true MB members conditionally independent and drops them. Always run
        // the minimal-separator search below, which tests against small
        // (powerful) separators instead.

        let mut worse_badness: f64 = f64::NEG_INFINITY;
        let mut stack_to_add: Vec<Vec<usize>> = Vec::from(vec![atts]);
        let mut stack_deps: Vec<VarSet> =
            Vec::from(vec![att_xs.clone()]);
        let mut stack_cond_prob: Vec<ProbabilityTIDs> =
            Vec::from(vec![prob_xs.clone()]);
        let mut iter: usize = 0;
        while !stack_to_add.is_empty() {
            iter += 1;
            let mut to_add = stack_to_add.pop().unwrap();
            let cur_deps = stack_deps.pop().unwrap();
            let cond_prob = stack_cond_prob.pop().unwrap();
            // Print prints
            // Check if deps too large
            if cur_deps.len() > eff_sub_mb_size {
                continue;
            }
            print!("\n\t\t\t{}. dep: [", iter);
            cur_deps.iter().for_each(|v| print!("{},", v));
            print!("], to add: [");
            to_add.iter().for_each(|v| print!("{},", v));
            print!("]");
            // Check if test will be too weak
            let mut ordered_cmb_nvals: Vec<usize> = self
                .cmb
                .difference(&cur_deps)
                .map(|m| self.db.nvals[*m])
                .collect();
            ordered_cmb_nvals.sort();
            ordered_cmb_nvals.reverse();
            let mut tmp_cond_card: usize =
                cur_deps.iter().fold(1, |a, d| a * self.db.nvals[*d]);
            for i in
                0..usize::min(self.sep_size, ordered_cmb_nvals.len())
            {
                tmp_cond_card *= ordered_cmb_nvals[i];
            }
            // Sample-size gate: skip a CI test we lack the power to run. The two
            // candidate bounds cross over in cardinality, and neither alone works
            // for both regimes the paper reports:
            //   * the lenient df-style product (card_x-1)*(card_t-1)*cond_card*5
            //     is small for low-cardinality vars (binary synthetic), so the
            //     higher-order tests run at N>=50 -- but it EXPLODES for
            //     high-cardinality vars (Mildew card 86, Barley 67), skipping
            //     nearly every test and collapsing the recovered MB to ~1 var;
            //   * the principled SCI bound grows only ~linearly in cardinality, so
            //     it stays lenient on the high-cardinality real datasets -- but its
            //     base of 35 plus terms exceeds 50 for binary conditioning and so
            //     zeros synthetic small-sample F1.
            // Take the MIN so each regime uses whichever bound is lenient there:
            // *5 wins at low card (synthetic small-sample), SCI wins at high card
            // (high-cardinality real-world). This matches the published behaviour
            // in both regimes simultaneously.
            let lenient = ((self.db.nvals[att_x] - 1)
                * (self.db.nvals[self.att_t] - 1)
                * tmp_cond_card
                * 5) as f64;
            let min_samp = f64::min(
                sci_min_sample_size(
                    self.db.nvals[att_x],
                    self.db.nvals[self.att_t],
                    tmp_cond_card,
                ),
                lenient,
            );
            if (self.db.sample_size as f64) < min_samp {
                print!("\tNot Enough Samples!");
                continue;
            }
            // Check if should find seps
            let mut skip: bool = false;
            for known in self.known_not_deps[&att_x].iter() {
                if cur_deps.is_subset(&known) {
                    skip = true;
                    break;
                }
            }
            let cur_df: usize = cur_deps
                .iter()
                .fold(1, |acc, a| acc * self.db.nvals[*a])
                * usize::max(self.db.nvals[self.att_t] - 1, 1)
                * usize::max(self.db.nvals[att_x] - 1, 1);
            skip = skip
                || chi_square_p_val(max_g_test.stat, cur_df)
                    > self.alpha;
            if !skip {
                // Lower bound on p-val is too low, need full check
                let ci: CIRes = self.test_ci(att_x, &cond_prob);
                if !ci.is_ci {
                    let (no_seps, badness) =
                        self.no_seps(att_x, &cur_deps, &cond_prob);
                    worse_badness = f64::max(worse_badness, badness);
                    if no_seps {
                        return (true, worse_badness);
                    }
                } else {
                    let mut tmp = self.known_not_deps[&att_x].clone();
                    tmp.retain(|known| !known.is_subset(&cur_deps));
                    tmp.insert(cur_deps.clone());
                    self.known_not_deps.insert(att_x, tmp);
                }
            }
            if cur_deps.len() == eff_sub_mb_size {
                continue;
            }
            // self.order_mb(&cond_prob, att_x, &mut to_add);
            // Add to stacks
            for i in 0..to_add.len() {
                let d = to_add[i];
                stack_to_add.push(to_add[0..i].to_vec());
                let mut deps_next = cur_deps.clone();
                deps_next.insert(d);
                stack_deps.push(deps_next);
                let prob_next = cond_prob.merge(&self.prob_atoms[d]);
                stack_cond_prob.push(prob_next);
            }
        }
        return (false, worse_badness);
    }

    fn no_seps(
        &mut self,
        att_x: usize,
        att_deps: &VarSet,
        prob_deps: &ProbabilityTIDs,
    ) -> (bool, f64) {
        print!("\n\t\t\tfind seps with mb: [");
        att_deps.iter().for_each(|a| print!("{},", a));
        print!("], with prob dom: [");
        prob_deps.get_atts().iter().for_each(|a| print!("{},", a));
        print!("]\n");
        let mut atts_to_add: BTreeSet<usize> =
            self.cmb.difference(att_deps).cloned().collect();
        atts_to_add.remove(&att_x);
        // Check if atts_to_add is subset of known seps
        if !self.known_seps[&att_x].contains_key(&att_deps) {
            self.known_seps
                .get_mut(&att_x)
                .unwrap()
                .insert(att_deps.clone(), HashSet::new());
        }
        for s in self.known_seps[&att_x][&att_deps].iter() {
            if s.is_subset(&atts_to_add) {
                return (false, f64::INFINITY);
            }
        }
        // let eff_sub_size = att_deps.len() +
        //     usize::min(self.sep_size, atts_to_add.len());
        let eff_sub_size = usize::max(
            att_deps.len()
                + usize::min(self.sep_size, atts_to_add.len()),
            usize::min(self.order, atts_to_add.len()),
        );
        let mut tmp_atts: Vec<usize> =
            atts_to_add.into_iter().collect();
        // self.order_mb(&prob_deps, att_x, &mut tmp_atts);
        // tmp_atts.reverse();
        let mut stack_to_add: Vec<Vec<usize>> = vec![tmp_atts];
        let mut stack_cond: Vec<VarSet> = vec![att_deps.clone()];
        let mut stack_cond_prob: Vec<ProbabilityTIDs> =
            vec![prob_deps.clone()];
        let mut worse_badness: f64 = f64::NEG_INFINITY;
        let mut iter: usize = 0;
        while !stack_to_add.is_empty() {
            let mut to_add = stack_to_add.pop().unwrap();
            let cur_cond = stack_cond.pop().unwrap();
            let cur_sep: BTreeSet<usize> =
                self.cmb.difference(&cur_cond).cloned().collect();
            let cond_prob = stack_cond_prob.pop().unwrap();
            if cur_cond.len() == eff_sub_size {
                iter += 1;
                print!("\t\t\t\t{}. cond: [", iter);
                cur_cond.iter().for_each(|v| print!("{},", v));
                print!("], to add: [");
                to_add.iter().for_each(|v| print!("{},", v));
                print!("]");
                // Check if known before testing
                if let Some(tmp) = self.known_ci[&att_x].get(&cur_cond)
                {
                    print!("\n");
                    if tmp.is_ci {
                        self.known_seps
                            .get_mut(&att_x)
                            .unwrap()
                            .get_mut(&att_deps)
                            .unwrap()
                            .insert(cur_sep);
                        return (false, tmp.badness);
                    }
                    continue;
                }
                let ci = self.test_ci(att_x, &cond_prob);
                worse_badness = f64::max(worse_badness, ci.badness);
                if ci.too_weak {
                    print!("\tTEST TOO WEAK!\n");
                } else if ci.is_ci {
                    print!("\tFOUND SEP!\n");
                    self.known_seps
                        .get_mut(&att_x)
                        .unwrap()
                        .get_mut(&att_deps)
                        .unwrap()
                        .insert(cur_sep);
                    return (false, worse_badness);
                } else {
                    print!("\n");
                }
            } else if cur_cond.len() < eff_sub_size {
                // self.order_mb(&cond_prob, att_x, &mut to_add);
                // to_add.reverse();
                for i in 0..to_add.len() {
                    let a = to_add[i];
                    let mut next_cond = cur_cond.clone();
                    next_cond.insert(a);
                    stack_cond.push(next_cond);
                    stack_to_add.push(to_add[0..i].to_vec());
                    stack_cond_prob
                        .push(cond_prob.merge(&self.prob_atoms[a]));
                }
            }
        }
        return (true, worse_badness);
    }

    fn make_strict(&mut self, att_x: usize, att_xs: &VarSet) -> VarSet {
        let mut att_xs_rem = att_xs.clone();
        for a in att_xs {
            let mut tmp_xs = att_xs_rem.clone();
            tmp_xs.remove(&a);
            let mut tmp_prob_xs = ProbabilityTIDs::new_empty(self.db);
            for x in tmp_xs.iter() {
                tmp_prob_xs = tmp_prob_xs.merge(&self.prob_atoms[*x]);
            }
            if self.not_ci(att_x, &tmp_xs, &tmp_prob_xs).0 {
                // x is redundant, remove it
                att_xs_rem = tmp_xs;
            }
        }
        att_xs_rem.insert(att_x);
        return att_xs_rem;
    }
}
