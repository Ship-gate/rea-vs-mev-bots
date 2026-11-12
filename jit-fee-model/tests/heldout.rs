use jit_fee_model::{fixture::CASES, naive_fee, range_aware_fee, rel_err};

fn median(mut v: Vec<f64>) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }

fn errs(held_out: bool, exact_only: bool) -> (Vec<f64>, Vec<f64>) {
    let mut n = vec![]; let mut r = vec![];
    for c in CASES.iter().filter(|c| c.held_out == held_out && (!exact_only || c.l_before_exact)) {
        let lt = c.l_before + c.l_jit;
        n.push(rel_err(c.observed_fee, naive_fee(c.fee_ppm, c.amount_in, c.l_jit, lt)));
        r.push(rel_err(c.observed_fee, range_aware_fee(c.fee_ppm, c.tick_lower, c.tick_upper, c.sqrt_p, c.zero_for_one, c.amount_in, c.l_jit, c.l_before)));
    }
    (n, r)
}

#[test]
fn fixture_has_heldout_split() {
    assert!(CASES.iter().filter(|c| c.held_out).count() >= 20);
    assert!(CASES.iter().filter(|c| !c.held_out).count() >= 20);
}

#[test]
fn range_aware_model_fits_heldout_cases() {
    let (n, r) = errs(true, false);
    let (mn, mr) = (median(n.clone()), median(r.clone()));
    let within_1pct = r.iter().filter(|e| **e < 0.01).count() as f64 / r.len() as f64;
    println!("held-out n={} median|err| naive={:.4} range_aware={:.4} frac<1%={:.2}", r.len(), mn, mr, within_1pct);
    assert!(mr < 0.01, "median held-out error must be <1%");
    assert!(within_1pct >= 0.95, ">=95% of held-out cases within 1%");
    let nw = n.iter().filter(|e| **e < 0.01).count() as f64 / n.len() as f64;
    println!("naive frac<1% = {:.2}", nw);
    assert!(within_1pct > nw, "range-aware model must beat naive on held-out");
}

#[test]
fn range_aware_never_worse_on_aggregate() {
    for split in [false, true] {
        let (n, r) = errs(split, false);
        let sn: f64 = n.iter().map(|e| e.min(10.0)).sum(); let sr: f64 = r.iter().map(|e| e.min(10.0)).sum();
        println!("split held_out={} sum|err| naive={:.3} range_aware={:.3}", split, sn, sr);
        assert!(sr < sn, "range-aware strictly better in aggregate");
    }
}

/// Fresh hold-out: 55 JIT pairs from a DISJOINT sample (days 8/22 of each month, 2023-01..2025-12) that was
/// collected only after the two-segment model was frozen. Never inspected during model development.
#[test]
fn frozen_model_on_fresh_disjoint_holdout() {
    use jit_fee_model::fixture_holdout2::CASES_HOLDOUT2 as H;
    let mut n = vec![]; let mut r = vec![];
    for c in H.iter() {
        let lt = c.l_before + c.l_jit;
        n.push(rel_err(c.observed_fee, naive_fee(c.fee_ppm, c.amount_in, c.l_jit, lt)));
        r.push(rel_err(c.observed_fee, range_aware_fee(c.fee_ppm, c.tick_lower, c.tick_upper, c.sqrt_p, c.zero_for_one, c.amount_in, c.l_jit, c.l_before)));
    }
    let w = |v: &Vec<f64>| v.iter().filter(|e| **e < 0.01).count() as f64 / v.len() as f64;
    println!("fresh holdout n={} frac<1%: naive={:.3} range_aware={:.3}; worst range_aware err={:.4}", r.len(), w(&n), w(&r), r.iter().cloned().fold(0.0, f64::max));
    assert!(H.len() >= 40);
    assert!(w(&r) >= 0.95);
    assert!(w(&r) >= w(&n));
}
