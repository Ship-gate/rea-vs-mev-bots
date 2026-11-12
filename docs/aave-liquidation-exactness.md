# Matching Aave V3 liquidation math to the wei

*Read-only replay of real Ethereum mainnet liquidations, October 2026. Nothing was signed or submitted.*

We built a simulator for Aave V3 liquidations and checked it against the chain. The first version looked good. It was not good enough, and finding out why was the most useful engineering result of the whole study. Getting the math exact then confirmed the economic result instead of rescuing it.

## Why "within 1%" is not enough

The first version was checked against 160 mainnet `LiquidationCall` events over about 30 days:

| Tolerance on collateral seized | Match rate (140 rows with recoverable pre-state) |
|---|---|
| Exact, within 1 wei | 32.9% |
| Within 1% | 82.9% |
| Within 5% | 100% |

That reads like success. But the median liquidation in this sample repaid about $400 of debt, and the liquidator's bonus is a few percent of that. A 1% error in seized collateral is the same size as the margin being measured. On that version, 102 of 160 events looked profitable for a new entrant. Most of that was model error.

## Six defects

Each was found by comparing against the deployed Pool (revision 11, v3.7) and against upstream source, then pinned with a regression test.

| # | Defect | Correct behaviour |
|---|---|---|
| 1 | Gas cost in wei was subtracted from profit in debt-token units. For USDC debt that is wrong by about 10¹². | Convert gas, and any builder payment, to debt units through the same oracle prices the Pool uses, rounding up. Refuse to answer if WETH has no price. |
| 2 | Efficiency-mode membership was read from reserve configuration bits, which are zero on v3.2 and later. | Read the eMode category's collateral bitmap and test the reserve's index bit. |
| 3 | User configuration bit pairs were swapped. | Bit `2i` is borrowing, bit `2i+1` is collateral. |
| 4 | Health factor was computed from the floored average liquidation threshold. | Compute from the unfloored sum of collateral × threshold. From v3.5, collateral balances round down and debt balances round up. |
| 5 | Liquidation rounding was half-up everywhere. | v3.7 rounds each step in a specific direction: bonus down, debt needed up, protocol fee as the Pool does. Earlier versions differ, so the engine must know the Pool revision. |
| 6 | `debtPrice × debtToCover × collateralUnit` overflowed 128 bits. An ordinary 0.077 ETH liquidation produces a 1.9 × 10⁴⁶ intermediate. | 256-bit numerators, as in the EVM. |

Defect 6 was found by the replay itself: the engine refused a routine liquidation that the chain had executed.

None of these is exotic. Three are about reading the protocol version correctly. That is the practical lesson: a liquidation model is tied to a specific Pool revision, and rounding direction is part of the specification.

## After the fixes

92 mainnet liquidations, blocks 26,013,421 to 26,148,750, all on Pool revision 11. Rows are in [`data/aave_exact_backtest_rows.jsonl`](../data/aave_exact_backtest_rows.jsonl).

| Quantity | Compared against | Exact |
|---|---|---|
| Health factor | `getUserAccountData` at the prior block | 92 of 92 |
| All account data fields | same | 88 of 92 |
| Debt repaid | the liquidation event | 92 of 92 |
| Collateral seized | the liquidation event | 86 of 92 |

Of the six collateral mismatches, three are within 2 wei and three differ by more. Those three are unexplained.

## What exactness bought

With exact amounts, executable exit quotes and every cost in the same unit:

- **12 of 92** events left room for an entrant to beat the winner after the fact, worth $4,023.77 in total.
- **One event was $4,016.11 of that.** Without it the total is $7.66. Without the top three it is $1.33.
- Assuming free capital instead of a 5 basis point flash loan: 26 positive events, $4,379.12 total, $44.90 without the largest, $16.37 without the top three.
- The median winner paid the block builder about **100% of gross**.

Under our cost model, 74 of the 86 winners we could price came out negative, with a median of about −$2. We do not read that as winners losing money. It says our assumptions are worse than theirs: they have their own inventory instead of flash loans, and better exits than a single Uniswap V3 path. That gap is exactly the edge a newcomer does not have.

The inexact model said 64% of events were profitable for an entrant. The exact one says 13% were, and that all of the value was one event.

## Limits

- Exit quotes are a lower bound: the best single Uniswap V3 path at the prior block. No other venues, no split routes.
- Transaction-time state uses prior-block balances with current-block oracle prices and indexes. A second liquidation of the same user in the same block is not supported.
- Entrant gas is assumed equal to the winner's gas used.
- "Could beat the winner" is measured after the fact against the winner's own payment. It is not a forward win rate, and no capture probability was measured.
- 92 events from one market over about three weeks.

## Why the simulator code is not here

The simulator is a port of Aave V3 logic, written by reading the upstream source. That source is licensed under the Business Source License 1.1, with a change to MIT scheduled for 2027-03-06 at the latest. We have not published the port in this MIT-licensed repository. The defects and results above are described in enough detail to check against upstream: `GenericLogic.sol`, `LiquidationLogic.sol`, `UserConfiguration.sol` and `TokenMath.sol` in [aave-dao/aave-v3-origin](https://github.com/aave-dao/aave-v3-origin).
