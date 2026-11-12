# JIT fee-capture model

A small Rust library, with no dependencies, that predicts the swap fee a just-in-time (JIT) liquidity position captures on Uniswap V3. It is an evaluator only. It never builds, signs or submits a transaction.

## What it models

A JIT provider adds concentrated liquidity in a narrow range immediately before a large swap and removes it immediately after. The obvious estimate of its fee income is:

```
fee = amount_in × fee_rate × L_jit / (L_before + L_jit)
```

That estimate is wrong whenever the swap does not stay inside the JIT range. This library uses a two-segment model instead:

- **Segment A.** If the pre-swap price sits outside the JIT range on the entry side, the input spent moving the price to the range edge earns the JIT position nothing.
- **Segment B.** Inside the range, fees are shared in proportion to liquidity, and accrual stops when the price exits the far edge.

`naive_fee` implements the first formula. `range_aware_fee` implements the two-segment model.

## Results

Observed fees come from public Ethereum mainnet receipts of one long-running JIT operator, 2023 to 2025.

| Sample | Cases | Naive model within 1% | Two-segment model within 1% |
|---|---|---|---|
| Sample A, held-out split | 30 | 87% | 100% |
| Sample B, fresh and disjoint | 55 | 90.9% | 98.2% (54 of 55) |

Worst two-segment error on sample B: 1.37%.

Sample B is the result to trust. The two-segment fix was found by inspecting misses in sample A, including cases in its held-out split, so that split is not pristine. Sample B was collected only after the model was frozen.

## Run it

```sh
cargo test -- --nocapture
```

## Limits

- Fee capture is not profit. In the study this came from, JIT positions marked at the pool price were break-even to negative after builder payments and gas. `JitEconomics` is an unvalidated scaffold for that accounting; every cost is an input you must supply.
- `f64` arithmetic. Suitable for filtering and estimation, not for exact on-chain amounts.
- Segment A assumes no initialized tick is crossed before reaching the range.
- One operator, one pool family, single tick-spacing ranges. Other setups are untested.
- Fixtures were generated from public receipts and pool state; the generator script is not included.
