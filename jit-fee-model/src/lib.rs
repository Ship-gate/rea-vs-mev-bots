//! Simulation-only JIT fee-capture model (Uniswap V3, one tick-spacing range).
//! H-JIT-1 (hypothesis under test): the fee a JIT LP captures from one exact-input swap is
//!   fee_jit = in_range_fee * L_jit / (L_before + L_jit)
//! where in_range_fee accounts for the swap EXITING the JIT range (naive model ignores this).
//! f64 is used deliberately: this is an evaluator, not an executor. Production would use U256/Q64.96.
pub mod fixture;
pub mod fixture_holdout2;

/// Naive model: whole swap pays fee inside the JIT range.
pub fn naive_fee(fee_ppm: u32, amount_in: f64, l_jit: f64, l_tot: f64) -> f64 {
    amount_in * fee_ppm as f64 / 1e6 * (l_jit / l_tot)
}

/// Range-exit-aware model. sqrt prices are in raw-token units (sqrt(token1/token0)).
pub fn range_aware_fee(fee_ppm: u32, tick_lower: i32, tick_upper: i32, sqrt_p: f64,
                       zero_for_one: bool, amount_in: f64, l_jit: f64, l_before: f64) -> f64 {
    let f = fee_ppm as f64 / 1e6;
    let l_tot = l_before + l_jit;
    let spa = 1.0001f64.powf(tick_lower as f64 / 2.0);
    let spb = 1.0001f64.powf(tick_upper as f64 / 2.0);
    // net (after-fee) input needed to push price to the range boundary
    let mut net_in = amount_in * (1.0 - f);
    // Segment A: pre-swap price sits OUTSIDE the JIT range on the entry side (observed in 7/61 cases:
    // the bot pre-positions one spacing *ahead* of the price). Input spent traversing to the range edge at
    // L_before only earns the JIT nothing. (Approximation: no initialized tick crossed in segment A.)
    let mut sp = sqrt_p;
    if zero_for_one && sp > spb { let need = l_before * (1.0 / spb - 1.0 / sp); if net_in <= need { return 0.0; } net_in -= need; sp = spb; }
    if !zero_for_one && sp < spa { let need = l_before * (spa - sp); if net_in <= need { return 0.0; } net_in -= need; sp = spa; }
    // Segment B: inside the JIT range with L_before + L_jit; stops at the far edge.
    let to_boundary = if zero_for_one { l_tot * (1.0 / spa - 1.0 / sp) } else { l_tot * (spb - sp) };
    let in_range_net = net_in.min(to_boundary.max(0.0));
    let in_range_fee = in_range_net * f / (1.0 - f);
    in_range_fee * l_jit / l_tot
}

/// Scaffold (NOT validated): per-opportunity net in numeraire units, marked at an external reference price.
/// All inputs must be supplied by an off-chain evaluator; unknowns (hedge slippage, inventory carry) are explicit params.
pub struct JitEconomics { pub fee_value: f64, pub inventory_delta_value: f64, pub gas_cost: f64,
                          pub builder_payment: f64, pub hedge_cost: f64, pub inventory_carry_cost: f64 }
impl JitEconomics {
    pub fn net(&self) -> f64 { self.fee_value + self.inventory_delta_value - self.gas_cost - self.builder_payment - self.hedge_cost - self.inventory_carry_cost }
    /// Decision rule: participate only if net >= min_margin (bid the remainder to the builder upstream).
    pub fn should_participate(&self, min_margin: f64) -> bool { self.net() >= min_margin }
}

pub fn rel_err(obs: f64, pred: f64) -> f64 { if obs == 0.0 { if pred == 0.0 { 0.0 } else { f64::INFINITY } } else { ((obs - pred) / obs).abs() } }
