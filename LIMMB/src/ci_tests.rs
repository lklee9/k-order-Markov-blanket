use std::collections::BTreeSet;

use crate::g2test::{chi_square_p_val, g2_df, g2_stat_with_probs, g2_df_eff};
use crate::prob_map::ProbabilityMap;
use crate::prob_tids::ProbabilityTIDs;
use crate::probability::Probability;
use crate::sci::cond_sci_stat;

pub trait CITest {
    fn is_too_weak(&self) -> bool;
    fn is_not_cond_indep(&self, alpha: f64) -> bool;
    fn get_statistic(&self) -> f64;
    fn get_mb_tested(&self) -> BTreeSet<usize>;
    fn get_badness(&self) -> f64;
}

// * G Test

pub struct GTest {
    pub vars: BTreeSet<usize>,
    pub samp_size: usize,
    pub stat: f64,
    pub df: usize,
    pub pval: f64,
    pub badness: f64,
}

impl GTest {
    pub fn new(
        prob: &ProbabilityMap,
        t: usize,
        x: &BTreeSet<usize>,
        cond: &BTreeSet<usize>,
    ) -> Self {
        // Get Probs
        let mut xtc: BTreeSet<usize> = cond.union(x).cloned().collect();
        xtc.insert(t);
        let prob_xtc: ProbabilityMap = prob.marginalize(&xtc);
        let prob_c: ProbabilityMap = prob.marginalize(cond);

        let xc: BTreeSet<usize> = cond.union(x).cloned().collect();
        let prob_xc: ProbabilityMap = prob.marginalize(&xc);

        let mut tc: BTreeSet<usize> = cond.clone();
        tc.insert(t);
        let prob_tc: ProbabilityMap = prob.marginalize(&tc);
        // print!("GTest: [");
        // x.iter().for_each(|a| print!("{},", a));
        // print!("], {} | [", t);
        // cond.iter().for_each(|a| print!("{},", a));
        // println!("]");

        let stat =
            g2_stat_with_probs(&prob_xtc, &prob_xc, &prob_tc, &prob_c);
        // let df = g2_df(prob.get_dataset(), t, x, cond);
        let df = g2_df_eff(x, t, &prob_c);

        let pval = if df == 0 {
            1.0
        } else {
            chi_square_p_val(stat, df)
        };
        Self {
            vars: x.clone(),
            samp_size: prob.get_dataset().sample_size,
            stat,
            df,
            pval,
            badness: -1.0 * stat,
        }
    }
    
    pub fn new_from_prob(
        prob_cond: &ProbabilityTIDs,
        prob_x: &ProbabilityTIDs,
        prob_t: &ProbabilityTIDs,
    ) -> Self {
        // Get Probs
        let prob_xc = prob_cond.merge(prob_x);
        let prob_xtc = prob_xc.merge(prob_t);
        let prob_tc = prob_cond.merge(prob_t);
        // print!("GTest: [");
        // x.iter().for_each(|a| print!("{},", a));
        // print!("], {} | [", t);
        // cond.iter().for_each(|a| print!("{},", a));
        // println!("]");

        let stat = g2_stat_with_probs(
            &prob_xtc, &prob_xc, &prob_tc, prob_cond,
        );
        // let df = g2_df(
        //     prob_cond.get_dataset(),
        //     *prob_t.get_atts().first().unwrap(),
        //     prob_x.get_atts(),
        //     prob_cond.get_atts(),
        // );
        let df = g2_df_eff(
            prob_x.get_atts(),
            *prob_t.get_atts().first().unwrap(),
            prob_cond,
        );
        let pval = if df == 0 {
            1.0
        } else {
            chi_square_p_val(stat, df)
        };
        Self {
            vars: prob_x.get_atts().clone(),
            samp_size: prob_cond.get_dataset().sample_size,
            stat,
            df,
            pval,
            badness: -1.0 * stat,
        }
    }
}

impl CITest for GTest {
    fn is_too_weak(&self) -> bool {
        return false;
        // return self.df * 5 > self.samp_size;
    }

    fn is_not_cond_indep(&self, alpha: f64) -> bool {
        return self.pval <= alpha;
    }

    fn get_mb_tested(&self) -> BTreeSet<usize> {
        self.vars.clone()
    }

    fn get_statistic(&self) -> f64 {
        self.stat
    }

    fn get_badness(&self) -> f64 {
        self.badness
    }
}

// * SCI

pub struct SCI {
    pub vars: BTreeSet<usize>,
    pub samp_size: usize,
    pub stat: f64,
    pub badness: f64,
    card_x: usize,
    card_t: usize,
    card_cond: usize,
}

pub fn sci_min_sample_size(
    card_x: usize,
    card_t: usize,
    card_cond: usize,
) -> f64 {
    // eps = delta = 0.05
    let bound1 = 35.0
        + 2.0
            * card_x as f64
            * (card_t as f64).powf(2.0 / 3.0)
            * (card_cond as f64 + 1.0);
    let bound2 = 35.0
        + 2.0
            * card_t as f64
            * (card_x as f64).powf(2.0 / 3.0)
            * (card_cond as f64 + 1.0);
    // print!("\tbounds: ({},{})", bound1, bound2);
    f64::min(bound1, bound2)
}

impl SCI {
    pub fn new(
        prob: &ProbabilityMap,
        t: usize,
        x: &BTreeSet<usize>,
        cond: &BTreeSet<usize>,
    ) -> Self {
        // Get Probs
        let mut xtc: BTreeSet<usize> = cond.union(x).cloned().collect();
        xtc.insert(t);
        let prob_xtc: ProbabilityMap = prob.marginalize(&xtc);
        let prob_c: ProbabilityMap = prob.marginalize(cond);

        let xc: BTreeSet<usize> = cond.union(x).cloned().collect();
        let prob_xc: ProbabilityMap = prob.marginalize(&xc);

        let mut tc: BTreeSet<usize> = cond.clone();
        tc.insert(t);
        let prob_tc: ProbabilityMap = prob.marginalize(&tc);

        let prob_x: ProbabilityMap = prob.marginalize(&x);

        // Calculate min sample size:
        let db = prob.get_dataset();
        let mut card_x: usize = 1;
        for v in prob_x.get_atts() {
            card_x = card_x * db.nvals[*v];
        }
        let mut card_cond: usize = 1;
        for v in prob_c.get_atts() {
            card_cond *= db.nvals[*v];
        }
        let card_t: usize = prob.get_dataset().nvals[t];

        let stat: f64 =
            if sci_min_sample_size(card_x, card_t, card_cond)
                > prob.get_dataset().sample_size as f64
            {
                f64::NAN
            } else {
                let s1 = cond_sci_stat(
                    &prob_xtc, &prob_xc, &prob_tc, &prob_c,
                );
                let s2 = cond_sci_stat(
                    &prob_xtc, &prob_tc, &prob_xc, &prob_c,
                );
                f64::max(s1, s2)
            };

        Self {
            vars: x.clone(),
            samp_size: prob.get_dataset().sample_size,
            stat,
            badness: -1.0 * stat,
            card_x: prob_x.get_size(),
            card_t: prob.get_dataset().nvals[t],
            card_cond: prob_c.get_size(),
        }
    }

    pub fn new_from_prob(
        prob_cond: &ProbabilityTIDs,
        prob_x: &ProbabilityTIDs,
        prob_t: &ProbabilityTIDs,
    ) -> Self {
        // Get Probs
        let prob_xc = prob_cond.merge(prob_x);
        let prob_xtc = prob_xc.merge(prob_t);
        let prob_tc = prob_cond.merge(prob_t);

        // Calculate min sample size:
        let db = prob_cond.get_dataset();
        let mut card_x: usize = 1;
        for v in prob_x.get_atts() {
            card_x = card_x * db.nvals[*v];
        }
        let mut card_cond: usize = 1;
        for v in prob_cond.get_atts() {
            card_cond *= db.nvals[*v];
        }
        let t: usize = *prob_t.get_atts().first().unwrap();
        let card_t: usize = prob_t.get_dataset().nvals[t];

        let stat: f64 =
            if sci_min_sample_size(card_x, card_t, card_cond)
                > db.sample_size as f64
            {
                f64::NAN
            } else {
                let s1 = cond_sci_stat(
                    &prob_xtc, &prob_xc, &prob_tc, prob_cond,
                );
                let s2 = cond_sci_stat(
                    &prob_xtc, &prob_tc, &prob_xc, prob_cond,
                );
                f64::max(s1, s2)
            };

        Self {
            vars: prob_x.atts.clone(),
            samp_size: db.sample_size,
            stat,
            badness: -1.0 * stat,
            card_x: card_x,
            card_t: card_t,
            card_cond: card_cond,
        }
    }

}

impl CITest for SCI {
    fn is_too_weak(&self) -> bool {
        // eps = delta = 0.05
        return sci_min_sample_size(
            self.card_x,
            self.card_t,
            self.card_cond,
        ) > self.samp_size as f64;
    }

    fn is_not_cond_indep(&self, _alpha: f64) -> bool {
        return self.stat > 0.0;
    }

    fn get_mb_tested(&self) -> BTreeSet<usize> {
        self.vars.clone()
    }

    fn get_statistic(&self) -> f64 {
        self.stat
    }

    fn get_badness(&self) -> f64 {
        self.badness
    }
}
