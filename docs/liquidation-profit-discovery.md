# Liquidation profit discovery

**Date:** 2026-11-01  
**Scope:** Evidence-driven search for where the proven liquidation execution engine can repeatedly capture profit.  
**Method:** Historical replay for economic feasibility; shadow observation for competitive feasibility. Replay success is never presented as a live win.  
**Verdict (bottom line):** **No candidate qualifies. All candidates are NO_GO.** The largest credible ex-top-3, competition-adjusted, capture-stressed expected net is on Arbitrum Aave V3 at ≈ **$0.22/day** (10% capture, 50% builder payment) — below any sensible operating cost and below the cost of measuring it. Do not deploy capital. No production transactions, signing, or deployment have occurred or are recommended.

---

## 1. Executive summary

| Candidate | Evidence window | Events | Gross bonus (all) | Ex-top-3 gross | Conservative net/day @10% capture, 50% builder | Capture prob. | Verdict |
|---|---|---|---|---|---|---|---|
| **Arbitrum Aave V3** | 7.1 days (Oct 24–31 2026) | 93 (14 priced) | $764.30 (14 rows) | $36.63 | **+$0.22/day** | UNKNOWN | **NO_GO** |
| Sonic Aave V3 | 90 days | 17 | $490.49 | $148.06 | +$0.078/day | UNKNOWN | **NO_GO** |
| Gnosis Aave V3 | 90 days | 9 | $217.55 | $0.17 | ≈ $0/day (negative after gas) | UNKNOWN | **NO_GO** |
| Compound III Base | 90 days | 62 (33 absorb + 29 buy) | $120.78 absorbed value | n/a (single incumbent) | ≈ $0/day realistic | UNKNOWN | **NO_GO** |
| Morpho Base late-tail | (closed) | — | — | ex-top-1% negative | — | — | **NO_GO (closed)** |
| Aave Base | (closed) | — | $46.07 / 30 d | — | — | — | **NO_GO (closed)** |
| cbETH reactive (Base) | (closed) | — | — | $3.27/day @10% cap | — | — | **STOP (closed)** |

**Ranking rule used throughout:** retained net = gross bonus − exit cost − repayment − matched winner priority payment − gas − failed-attempt cost − fixed infra, with the **top three events removed** (outlier dependence test), then stressed at **5–10% capture** and **50–100% builder payment**. Never ranked by gross.

**Why every candidate fails:** each market is dominated by one incumbent (55–85% of events), the non-outlier tail is worth tens of dollars per quarter, and no capture probability has been measured. The only measured competitive evidence (failed attempts in winner blocks) confirms the incumbent is fast and the race is already crowded.

---

## 2. Evidence rules and what was proven vs. not

**Proven by historical replay (economic feasibility):**
- Winner behavior, event frequency, gross bonus at event-block oracle prices, gas paid by winners, flash-premium cost, asset concentration, incumbent dominance, outlier dependence, burstiness.
- Model/replay fidelity on selected EVM math (Lane G baseline: 160 real `LiquidationCall` events, blocks 26,013,421–26,221,679).

**NOT proven (and must not be implied):**
- Any live entrant win by the local stack. No production transaction, signing, or deployment has occurred.
- Any capture probability. Marked **UNKNOWN** for every candidate until a live-shadow race measures it.
- Any advantage from a faster or more exact execution stack. See §8.

**Method discipline followed:**
- Historical replay only for economics. Replay success ≠ live win.
- Shadow observation (read-only, no broadcast, no capital) is the only permitted next step for competition.
- No new strategy built before evidence gates pass.
- No candidate dependent on rare jackpots, unmeasured competition, or optimistic execution is accepted.

---

## 3. Closed candidates (do not revisit)

### 3.1 Morpho Base late-tail — NO_GO (closed per user)
- 963 txs / 2,844 events; winner net $8,974.60; **83% of positive profit in one 24-hour cascade**; ex-top-1% **−$4,098.81**.
- Atlas auction: bids consumed ~92–100% of bonus.
- **Accepted as NO_GO. Not optimized or revisited.**

### 3.2 Aave Base — NO_GO
- $46.07 historical-winner net over 30 days; −$96.95 ex-top-1%.

### 3.3 cbETH reactive route (Base) — STOP
- Ex-top-3 EV $3.27/day at 10% capture; −$1.46/day at 2× winner fee; −$0.46/day at 5% capture.

### 3.4 Solana keeper markets / Uniswap Firepit
- Not evaluated for liquidation; out of scope. Firepit described as "busy threshold race with outlier-driven economics" (Opportunity Radar). No build until positive keeper economics shown.

---

## 4. Candidate: Arbitrum Aave V3 (best-ranked, still NO_GO)

**Window:** 2026-10-24 → 2026-10-31 (7.1-day span, 8 calendar days). Source: `data/arb_analysis.json`.

### 4.1 Historical evidence (measured)
- **93 liquidation events** in **82 transactions** by **15 unique liquidators**, 10 collateral assets.
- **Only 14 of 93 events are priced** (missing archive oracle state for the rest; see §4.5). Those 14 rows carry **$764.30** gross bonus at event-block Aave oracle prices.
  - Top event: **$357.84** (46.8% of priced gross).
  - Top 3 events: **$727.67** (95.2% of priced gross).
  - Ex-top-3 gross: **$36.63**.
- Median priced bonus: $2.41. Mean: $54.59. 4 of 14 rows are same-asset (collateral == debt), $121.10.
- **Bursty, not steady:** 41 events on Oct 30, 26 on Oct 29 = 72% of all events in two days. 8 active days but most have 1–5 events.
- Gas: $4.82 total over the 14 priced rows (≈$0.34/row at $4k ETH). Flash premium (Aave 5 bps) on the 14 priced rows' debt notional: $8.38 total.

### 4.2 Incumbent / competition (measured)
- **28 failed liquidation attempts** in **21 of 82 winner blocks** (25.6% of blocks), from **14 unique senders**.
- Some failed senders also won elsewhere (vertical integration / shared infrastructure).
- Shared executor `0x8ad1ae9d…` routed **38 events from 8 senders and 5 liquidators**. Only **1 of 93 events** called the Aave Pool directly.
- **0 of 93 had a successful same-block competitor** — but losers revert silently, so absence of success is **not** evidence of absence of competition. This is a measurement limitation, not a safety signal.
- Failed-attempt gas cost: **$0.58 total** at $4k ETH (median $0.009/attempt) — competitors are bidding cheaply, which means the race is not expensive to enter but also that margins are thin.

### 4.3 Conservative economics
Using the 14 priced rows only, ex-top-3:

| Builder payment | Flash premium | Net ex-top-3 (7.1 d) | Per day | @10% capture | @5% capture |
|---|---|---|---|---|---|
| 0% | 5 bps | $34.04 | $4.79 | $0.48 | $0.24 |
| 50% | 5 bps | $15.72 | $2.21 | **$0.22** | $0.11 |
| 100% | 5 bps | −$2.59 | −$0.37 | −$0.04 | −$0.02 |

- **Best case (0% builder, 100% capture, no top-3 removal):** $105.79/day. This is the gross-outlier-inclusive ceiling and is **not** a business plan — it is the number that disappears the moment the top events are removed or a builder takes a cut.
- **Conservative (50% builder, 10% capture, ex-top-3): $0.22/day.**

### 4.4 Why NO_GO
1. The entire priced value is concentrated in 3 events (95.2%). Ex-top-3 is $36.63 over 7.1 days.
2. 85% of events (79/93) are unpriced — the full-market economics are **UNKNOWN**. The 14 priced rows are the large-asset ones (WETH, WBTC, USDC), so they may overstate the tail; if anything the unpriced tail is smaller.
3. Burstiness: 72% of events in 2 days. "Average per day" is not a steady income stream.
4. Competition is real and crowded (shared executors, 14 failed senders, cheap failed attempts).
5. Capture probability unmeasured (UNKNOWN). Even an optimistic 50% capture on the conservative tail yields ~$1.1/day — below infra cost.
6. 100% builder payment makes it negative.

### 4.5 Open measurement gap
- Arbitrum free RPC has no archive state (`historical state ... is not available`). 79/93 events cannot be priced at event block. A paid archive RPC (Alchemy/QuickNode) would close this, but the expected value of the information is low given the ex-top-3 tail.

---

## 5. Candidate: Sonic Aave V3 — NO_GO

**Window:** 90 days (2026-08-03 → 2026-11-01). Source: `data/undercontested_priced.json` (chain=sonic).

### 5.1 Historical evidence (measured)
- **17 liquidation events**, 17 txs, **7 unique liquidators**, total gross bonus **$490.49** at event-block Aave oracle prices.
  - Top event: $166.34 (33.9%).
  - Ex-top-1: $324.15. **Ex-top-3: $148.06.**
- **14 active days out of 90** (sparse).
- Top liquidator `0x67981ba6…` took **8 events / $268.50 (54.7%)**.
- Total debt notional: $12,016.60. Flash premium (5 bps): $6.01 over 17 events.
- Gas: ~$1.92 total over 17 txs (native S, ≈$0.11/tx at $1/S assumption; conservative high).

### 5.2 Competition (measured, with caveat)
- Failed-attempt scan found **98 real-selector failed attempts** (56 × `0xd425a35d` + 42 × `0x4dfce92c`) in **17 blocks** (~5.8/block), **14 of 17 blocks had failures**.
- (The scan also returned 197 `0x00000000` matches — these are false positives from fallback-selector matching and are **not** counted as liquidation competition.)
- Real competition is dense: ~6 attempts per winner block. The incumbent (`0x67981ba6…`) holds 54.7% despite this.

### 5.3 Conservative economics (ex-top-3, 90 days)

| Builder payment | Net ex-top-3 | Per day | @10% capture |
|---|---|---|---|
| 0% | $144.13 | $1.60 | $0.16 |
| 50% | $70.10 | $0.78 | **$0.078** |
| 100% | −$3.93 | −$0.04 | −$0.004 |

### 5.4 Why NO_GO
- Ex-top-3 tail is $148.06 over 90 days = $1.64/day gross at 100% capture. At 10% capture and 50% builder = $0.078/day.
- 14 active days / 90 = sparse; no steady state.
- Incumbent holds 54.7% with dense competition (6 attempts/block). Capture probability for a new entrant is likely well below 10%.
- Gas is cheap but the margin is already gone after builder payment at 100%.

---

## 6. Candidate: Gnosis Aave V3 — NO_GO

**Window:** 90 days. Source: `data/undercontested_priced.json` (chain=gnosis).

### 6.1 Historical evidence (measured)
- **9 liquidation events**, 9 txs, **6 unique liquidators**, total gross bonus **$217.55**.
  - Top event: **$163.42 (75.1%)**.
  - Ex-top-1: $54.13. **Ex-top-3: $0.17.**
- **6 active days out of 90.**
- Top liquidator `0x378b159c…` took 2 events including the $163.42 one.
- Gas: ~$46.48 total, of which **one outlier tx paid $46.44** (priority-fee spike on the $163.42 event, 28% of that bonus). Excluding outlier: ~$0.05 total.
- Flash premium (5 bps) negligible.

### 6.2 Competition
- 34 failed attempts in 5 of 9 blocks (19 senders). 4 direct Aave Pool calls same/next block; 2 reverted (dust griefing).

### 6.3 Conservative economics
- Ex-top-3 gross = **$0.17 over 90 days**. Even at 100% capture and 0% builder: **$0.0019/day**. Gas alone (conservative $0.05) exceeds this.
- **Negative after any realistic cost.** No business.

### 6.4 Why NO_GO
- The market is a single jackpot ($163.42, 75.1% of all value) plus dust. Ex-top-3 is $0.17.
- Removing the top event leaves $54.13 over 90 days. Removing top 3 leaves nothing.
- **Outlier-dependent by construction. Rejected.**

---

## 7. Candidate: Compound III Base — NO_GO

**Window:** 90 days (blocks 52,011,727 → 53,350,927). Source: `data/compound_base_events.jsonl` (62 events).

### 7.1 Historical evidence (measured)
- **62 events total: 33 `absorb` + 29 `buy`** across `cUSDCv3` (47) and `cWETHv3` (15). `cUSDbCv3`: **0 events.**
  - cUSDCv3: 33 absorb + 14 buy.
  - cWETHv3: 0 absorb + 15 buy.
- **Absorbed value (event `usdValue`, 8 decimals): $120,777 over 90 days = $1,342/day** at 100% capture.
- **Absorber concentration: 4 unique absorbers; top `0x6c7c74fd…` took 28/33 (85%).**
- **Buyer concentration: 4 unique buyers; top `0x16e41af5…` took 25/29 (86%).**
- Cross-chain incumbent present: `0xd12810b1…` (Arbitrum's top liquidator) appears in both absorb (2) and buy (2) on Compound Base.
- Frequency: 0.37 absorb/day on cUSDCv3, 0.32 buy/day total. Sparse.

### 7.2 Economics (partially verified)
- Compound III profit comes in two steps: (1) call `absorb` (small absorber reward) on an insolvent position, (2) `buyCollateral` at a governed discount (typically 5–15%).
- Buy-event raw sums (cUSDCv3 WETH collateral): $117,173 USDC paid for 53.2 WETH received over 10 WETH-collateral buys → implied discount depends on WETH price at event time, which requires archive RPC (unavailable). Value is **UNVERIFIED**.
- Even optimistically (5% discount on $1,342/day absorbed value = $67/day gross at 100% capture), the two-step process and 85%/86% incumbent concentration mean a new entrant realistically captures ≈0.

### 7.3 Why NO_GO
- Incumbent-dominated (85% absorb, 86% buy) — same pattern as Aave venues.
- Two-step capital/risk process (absorb then buy) vs. single atomic Aave liquidation.
- Value unverified; even optimistic ceiling ($67/day gross) is marginal and requires beating an incumbent that already holds 85%+.
- Frequency low (0.69 events/day combined).

---

## 8. Technology advantage

**Measured evidence: none.** No live A/B test shows that a faster or more exact execution stack improves liquidation capture versus the baseline.

**What the evidence does show:**
- Lane G baseline used oracle-parity disposal and priority fees only, without coinbase traces; 20 pre-state failures out of 160 events.
- Deterministic math and network latency dominate execution; the binding constraint is **inclusion priority** (builder payment, coinbase relationship), not compute throughput.
- Preconf push (Base Flashblocks) gives p50 319 ms lead over polling, but **no liquidation followed any of 7 observed oracle updates**; push is available to all subscribers, so it confers no information edge.
- Upstream oracle prediction (n=2 BTC, 21 s and 54 s leads, 1 false alarm in 4 crossings) is the only information edge found, and it is **auctioned away** on SVR feeds (bids consume ~100% of bonus).

**Conclusion:** No measured technology advantage. Do not attribute any expected capture improvement to the stack. Treat as unproven until a live-shadow A/B measures it.

---

## 9. Capture probability — all UNKNOWN

No valid live-shadow race evidence exists for any candidate. The Arbitrum failed-attempt data proves competition exists but does not measure our capture rate. **Every capture probability in this report is UNKNOWN and must remain so until measured.**

Assumed stress values used for ranking: **5% and 10% capture** (conservative; consistent with cbETH reactive and prior kill decisions). Even at 50% capture, no candidate reaches $5/day on the ex-top-3 tail.

---

## 10. Smallest live-shadow experiment (best candidate: Arbitrum Aave V3)

**Purpose:** Measure capture probability and inclusion latency. **Read-only. No broadcast, no signing, no capital, no contract deployment.**

### 10.1 Scope
- Chain: Arbitrum One.
- Market: Aave V3 (the 10 collateral assets observed: WETH, WBTC, USDC, USDT, ARB, ...).
- Duration: **14 days** (to span more than one burst window; the 7.1-day sample was dominated by 2 days).
- Budget: **$0 capital, $0 broadcast.** Infra: one existing RPC (free tier acceptable for read-only; paid archive RPC optional for pricing the unpriced 79 events).

### 10.2 What to measure (the only outputs that matter)
1. **Detection latency:** block number / timestamp of `LiquidationCall` log vs. our detection. (Already ~0 on polling; measure Flashblocks push if available on Arbitrum.)
2. **Inclusion race:** for each winner block, record our (simulated) tx submission timestamp, gas price, and whether a same-block competitor existed. **Do not submit.** Reconstruct from mempool observation + block ordering.
3. **Capture probability proxy:** count of winner blocks where our simulated submission would have been included ahead of the winner (by gas price / tx index), divided by total winner blocks. This is an upper bound, not a real win.
4. **Net per captured event:** replay the 14+ priced events through our engine; record gross, gas, flash premium, builder payment (assume 50% and 100% scenarios).
5. **Burstiness:** events per day distribution; confirm or refute the 72%-in-2-days pattern over 14 days.

### 10.3 What this experiment CANNOT prove
- It cannot prove a real live win. No transaction is broadcast.
- It cannot measure builder inclusion (we are not in any builder's bundle).
- It cannot measure coinbase priority (we are not paying real priority fees).
- It cannot validate the smart contract path (no callback, no exit, no repayment executed).
- **It cannot convert an UNKNOWN capture probability into a number.** It can only produce an upper bound. A real capture number requires real submissions, which requires capital and approval.

### 10.4 Success / kill criteria for the experiment
- If the 14-day capture upper bound is **< 10%**: kill. (Consistent with prior kill decisions.)
- If the ex-top-3 tail over 14 days is **< $100 gross**: kill (not worth infra).
- If builder payment at 100% makes net negative on the tail: kill.
- Only if all three pass AND a real-capture A/B is approved by the user: consider a funded live test with a hard cap.

### 10.5 What NOT to do
- Do not sign, deploy, or broadcast anything.
- Do not spend capital.
- Do not build a new strategy before the gates pass.
- Do not revisit Morpho Base late-tail, Aave Base, or cbETH reactive.

---

## 11. What would have to be true to say GO

For any candidate to move from NO_GO to GO, **all** of the following must be measured (not assumed):

1. **Capture probability ≥ 10%** measured via real submissions in a capped live test (not a shadow proxy).
2. **Ex-top-3 net ≥ $5/day** after 50% builder payment, gas, flash premium, and $1/day infra, over ≥ 30 days.
3. **Non-bursty frequency:** ≥ 1 event/day on ≥ 50% of days (no single 24-hour cascade providing > 50% of profit).
4. **Incumbent share ≤ 50%** OR a measurable differentiation (information edge, builder relationship, or cost advantage) that survives auction.
5. **Technology advantage measured:** an A/B showing the execution stack improves capture vs. baseline by a statistically significant margin.
6. **No dependency on a single jackpot:** ex-top-1 ≤ 25% of total value.

**None of the four candidates meet more than two of these.** Arbitrum meets #2 marginally (if the unpriced 79 events are large) but fails #3, #4, and #6. Sonic fails #2, #3, #4. Gnosis fails #2, #3, #6. Compound fails #2, #4, #5.

---

## 12. Final recommendation

- **No deployment.** All candidates NO_GO.
- **No capital.** No signing, no deployment, no broadcast.
- **If the user wants to invest further:** fund the 14-day read-only shadow experiment on Arbitrum Aave V3 (§10) at $0 capital cost. Its only deliverable is a capture upper bound and a burstiness confirmation. It will not produce a GO; it will either confirm the kill or justify a funded test.
- **Do not build new strategies** until a candidate passes the gates in §11.
- **Close the book on:** Morpho Base late-tail (already closed), Aave Base, cbETH reactive, Gnosis Aave V3, Sonic Aave V3 (unless shadow shows ≥ 10% capture upper bound), Compound III Base (unless a 30-day census shows incumbent share < 50%).

---

## Appendix A — Data files

| File | Contents |
|---|---|
| `data/undercontested_enriched.json` | Raw Gnosis (9) + Sonic (17) event logs, oracle addresses, flash premium, direct-call evidence. |
| `data/undercontested_priced.json` | Gnosis/Sonic events priced at event-block Aave oracle prices. |
| `data/arb_analysis.json` | 93 Arbitrum Aave events, 14 priced rows, gas fields, selectors. |
| `data/arb_failed_attempts.json` | 28 failed attempts, 21 blocks, 14 senders, per-block detail. |
| `data/compound_base_events.jsonl` | 62 Compound III Base absorb/buy events (90 days). |

Earlier lane reports and the simulation engine referenced in this document are not part of this repository.

## Appendix B — Key measurements reproduced

- Lane G exact backtest: 92 rows, 12 ex-post "could win," total modeled headroom $4,023.77; ex-top-1 $7.66, ex-top-3 $1.33; one $4,016.11 outlier. Structural zero-flash on same 92 rows: 26 positive, $4,379.12 total; ex-top-1 $44.90, ex-top-3 $16.37.
- Information edge: preconf push p50 319 ms ahead of polling; no liquidation followed any of 7 observed oracle updates; upstream oracle prediction n=2 BTC with 21 s and 54 s leads, 1 false alarm in 4 crossings; SVR bids consume ~100% of bonus.
- Morpho Base: 963 txs / 2,844 events, winner net $8,974.60, ex-top-1% −$4,098.81, 83% of positive profit in one 24-hour cascade.
- Aave Base: $46.07 winner net over 30 days, −$96.95 ex-top-1%.

## Appendix C — Assumptions

- ETH price: $4,000 (gas conversion for Arbitrum and Gnosis xDAI treated as ~$1).
- Sonic native (S) price: $1.00 (conservative high; actual may be lower, which only reduces costs further — does not change verdicts).
- Flash premium: 5 bps (Aave default) on debt notional.
- Builder payment scenarios: 0% / 50% / 100% of gross bonus.
- Capture scenarios: 5% / 10% (consistent with prior kill decisions and cbETH reactive).
- Outlier removal: top 3 events by gross bonus.
- Window lengths: Arbitrum 7.1 days, Sonic/Gnosis/Compound 90 days.

---

*Generated 2026-11-01. All evidence read-only. No production transactions, signing, deployment, or capital deployment performed or recommended.*
