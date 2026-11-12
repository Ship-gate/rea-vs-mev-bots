# Earlier information: we got faster and it did not matter

*Measured 2026-10-31 on Base, before the Denim upgrade. Read-only: no transaction was built for sending, signed for sending or submitted. No wallet and no capital.*

The question: liquidations on Base are won in the 200 ms block slice after an oracle update, and our pipeline took 334 ms. If we see the update earlier and simulate faster, does the opportunity become worth taking?

The answer is no. We got inside the window. The economics did not move.

## Verdict: **STOP** (for this market and route)

Push detection works and removes most of the 334 ms. With a local simulation it puts us comfortably inside the 200 ms race.
**Neither latency nor information is now the binding constraint.** Once we arrive in time, the cbETH/USDC economics are
unchanged from the earlier 100 ms case and still fail the frozen gates and our outlier rule:

- **Outliers:** without the 3 largest opportunities, EV at 10% capture is $3.27/day.
- **Fees:** it falls to -$1.46/day if we pay twice the winner's priority fee.
- **Capture:** at 5% capture without the top 3 it is -$0.46/day.
- **Coverage:** only 44% of sampled debt could be replayed, against a 50% gate.
- **Visibility:** whether winners see updates before inclusion is still inconclusive.

Push detection gives no information edge, because every competitor can subscribe to the same Flashblocks stream. What
remains is a priority-fee auction inside the next 200 ms window, which is the part that fails.

| Option | Recommendation |
|---|---|
| Upstream oracle detection (predicting Chainlink updates from off-chain prices) | **Do next, as a cheap read-only test.** It is the only route to information others do not already get for free. |
| Screen direct cbBTC/USDC | Defer. Same race, same event dependence ($45k corrected, $13.7k without its 3 largest). Worth it only if the upstream test shows an edge. |
| Switch to Solana keeper markets | Run the screen in parallel; build nothing until it shows positive keeper economics. |

## 1. Working endpoints and observed schemas

| Endpoint | Transport | Working subscriptions | Rejected or silent |
|---|---|---|---|
| `wss://mainnet-preconf.base.org` | WebSocket JSON-RPC | `newFlashblocks`, `pendingLogs` (filtered), `newHeads`, `logs` (filtered) | `newPendingTransactions`: "invalid subscription type" |
| `wss://base-rpc.publicnode.com` | WebSocket JSON-RPC | `newFlashblocks`, `newHeads`, `logs` (filtered), `newPendingTransactions` | `pendingLogs`: accepted, never delivered |
| `https://base-rpc.publicnode.com` | HTTP polling | `eth_getLogs` to `"pending"`; needs an address list, at most 10 | none |
| `https://mainnet-preconf.base.org` | HTTP polling | `eth_getLogs` to `"pending"`: "request limit reached" on every call at 2 req/s | (this machine was also running another collector) |
| `https://mainnet-sequencer.base.org` | HTTP | submission only; any read method gets "rpc method is not allowed" | used only for round-trip timing |

**Message shapes**
- **`newFlashblocks`:** a block-shaped object with a zero `hash`, a `number`, a running `gasUsed` and a `transactionsRoot`.
- **`pendingLogs`:** standard log objects with a zero `blockHash` and a `transactionHash`, delivered as each Flashblock seals.
- **Raw stream** (`wss://mainnet.flashblocks.base.org/ws`): brotli-compressed messages of `{payload_id, index, base, diff, metadata}`
  carrying raw transactions and no receipts. It was probed only.

**Node:** v20.20.0 has `WebSocket` only behind `--experimental-websocket`, so we used a small
dependency-free RFC 6455 client instead.

**Event definitions**, identical for push and poll:
- **oracle:** `AnswerUpdated`, `NewTransmission` or the SVR secondary event `0x8d530b9d…`, from the 6 aggregators behind the
  cohort markets (cbETH, BTC, XRP, ETH, USDC and AERO, all quoted in USD).
- **liquid:** Morpho `Liquidate`.

## 2. Latency breakdown

All figures were measured from a desktop machine on a home connection, through a local proxy. They are upper bounds for a
colocated setup.

| Stage | Before: original 334 ms baseline | After: push, remote simulation | After: push, local warm simulation |
|---|---:|---:|---:|
| Waiting to observe the update | 100 (half a poll interval) | 42 (one-way delivery, estimated) | 42 |
| Reading pending state | 81 | 0 (logs arrive pushed) | 0 |
| Simulation | 100 | 97 (p50, n=30) | 7.9 (p50, n=30) |
| Building the calldata | — | 0.002 (n=10,000) | 0.002 |
| Signing (random key, random digest) | — | 0.23 (n=5,000) | 0.23 |
| Sending (half a round trip) | 50 | 39 (n=60) | 39 |
| Own computation | 3 | included | included |
| **Total, p50** | **334** | **178** | **89** |
| **Total, p90** | — | **223** | **122** |

Sample counts and spread:

| Measurement | n | p50 ms | p90 ms | p99 ms | Failures |
|---|---:|---:|---:|---:|---:|
| State read, new connection per call (original method) | 60 | 98.7 | 109.1 | 130.6 | 0 |
| State read, persistent connection (preconf) | 60 | 100.6 | 142.6 | 199.9 | 0 |
| Pending block read, persistent (preconf) | 60 | 83.8 | 111.7 | 190.7 | 0 |
| Submission round trip, persistent (sequencer) | 60 | 78.3 | 92.2 | 108.0 | 0 |
| Remote simulation of the reference replay (publicnode) | 30 | 96.8 | 120.8 | 179.2 | 0 |
| Local `anvil` fork, same replay, warm | 30 | 7.9 | 19.6 | 18,772 (the first, cold call) | 0 |

- **Accuracy.** Remote and local simulations of the reference replay (cbETH winner `0xfebf9f98`) returned byte-identical results
  (sha256 `94a1b153…` for both). That replay matches the chain to 0.0 bps.
- **Persistent connections don't help.** Each remote step costs one network round trip of 80–100 ms, so the only real savings
  are removing round trips (push) and making simulation local.
- **The local result is conditional.** The 7.9 ms is a warm fork. The first call took 18.8 s while it fetched state, and live
  state changes every 200 ms, so production needs a node that follows the chain head, not a fork. None was synced: the
  economics below are the same at 89 ms or 178 ms, so a node would not change the decision.
- **Delivery delay is estimated** as half the pending-read round trip. A competitor's own delivery delay cannot be observed.

## 3. Detection coverage and missing data

The recorder ran for 29.4 minutes. Afterwards it was checked against the final chain for the same blocks. In that time 7
watched oracle updates arrived (14 logs, because each update emits both `AnswerUpdated` and `NewTransmission`): BTC 3, ETH 2,
cbETH 1, AERO 1. No Morpho liquidations happened in the five cohort markets, so the `liquid` definition was subscribed but
never exercised.

| Source | Logs on final chain | Seen | Missing | Extra | Duplicates |
|---|---:|---:|---:|---:|---:|
| Push, preconf `pendingLogs` | 14 | 14 | 0% | 0 | 0 |
| Push, preconf `logs` | 14 | 14 | 0% | 0 | 0 |
| Push, publicnode `logs` | 14 | 14 | 0% | 0 | 0 |
| Poll, publicnode, every 500 ms | 14 | 14 | 0% | 0 | 294 (overlapping ranges, deduplicated by key) |

Feed health: no disconnects, no provider or subscription errors, and no poll errors in 3,515 cycles (p50 192 ms per cycle).
publicnode `pendingLogs` was accepted but delivered nothing.

**How long each source trailed the earliest one**, in ms. Same 14 logs, same definition:

| Source | p10 | p50 | p90 |
|---|---:|---:|---:|
| Push, preconf `pendingLogs` | 0 | 0 | 0 (first every time) |
| **Poll, publicnode, 500 ms (existing method)** | 24 | **319** | 553 |
| Push, preconf `logs` (canonical block) | 366 | 1,534 | 1,853 |
| Push, publicnode `logs` | 466 | 1,745 | 1,958 |

**Skew between the two providers** (publicnode minus preconf, same item):

| Stream | n | p10 | p50 | p90 | p99 |
|---|---:|---:|---:|---:|---:|
| `newFlashblocks` | 9,660 | 7 | 52 | 277 | 850 |
| `newHeads` | 882 | 10 | 39 | 97 | 303 |

Flashblocks arrived from preconf every p50 173 ms (p10 144, p90 244). publicnode's spacing is irregular (p10 42, p90 319),
consistent with batching.

Push beats polling by p50 319 ms on identical events, and that saving is the same size as the race window, so it is the biggest
single gain. The canonical `logs` subscription runs about a whole block behind before Denim; after Denim it should catch up,
because blocks themselves become 200 ms.

## 4. Shadow race

**Live, verified (29.4 minutes).** For each of the 7 oracle updates the recorder logged when the oracle event happened (its
on-chain block and position), our first detection (preconf `pendingLogs`), and a hypothetical ready-to-submit time. Ready is
first detection plus the measured simulation (p50 97 ms) plus signing (0.23 ms), which comes to 97 ms after detection. Estimated
arrival at the sequencer is 136 ms after detection, adding half the measured submission round trip.

**No liquidation followed any of these updates**, so there was no winner to observe and **no live race outcome to report.**
Nothing here is claimed as a win.

- These are observation times at our machine, not sequencer arrival times.
- A competitor's private submission time is unknowable; only when its transaction first became visible to us can be recorded.
- The updates landed early in their blocks (transaction index 11 to 113), at the low-fee tail of a Flashblock, as in the cbETH
  history.

**Historical cohort, modelled (not verified).** The 15 replayable cbETH liquidations from our earlier cbETH/USDC economics study were rerun
through the frozen report at the new latencies. The thresholds were not changed.

| Pipeline | Latency | Reachable | 10% capture, matching fee: all / without largest / without top 3 ($/day) | Twice the fee, without top 3 | 5% capture, without top 3 | Frozen verdict |
|---|---:|---:|---|---:|---:|---|
| Original poll baseline | 334 ms | 0 | 0 / 0 / 0 | 0 | 0 | NO_GO |
| Push, remote simulation, p50 | 178 ms | 12 | 50.00 / 24.30 / **3.27** | **-1.46** | **-0.46** | NO_GO |
| Push, remote simulation, p90 | 223 ms | 0 | 0 / 0 / 0 | 0 | 0 | NO_GO |
| Push, local warm simulation, p50 / p90 | 89 / 122 ms | 12 | 50.00 / 24.30 / **3.27** | **-1.46** | **-0.46** | NO_GO |

At 178 ms and below the failing gates are replay coverage (44%, needs 50%) and pre-inclusion visibility (not established). Our outlier rule
fails too: the value depends on the top 3 events, a doubled fee makes it negative, and so does 5% capture. Reachable only means
arriving within the 200 ms window. Winning inside that window is a priority-fee auction that is not simulated, and failed
attempts are charged at 150,000 gas.

## 5. Flashblocks now, and after Denim

Denim (2026-11-05 18:00 UTC) replaces Flashblocks with canonical 200 ms blocks.

- **Recorder:** it subscribes to the Flashblock types (`newFlashblocks`, `pendingLogs`) and the standard types (`newHeads`,
  `logs`) at the same time, and records every subscription success or error. After Denim it keeps working on `newHeads` and
  `logs`; if the Flashblock types are removed, their errors are recorded rather than crashing it. Only pre-Denim behaviour was
  observed.
- **Analysis:** today a "next Flashblock" boundary is inferred from jumps in priority fee. After Denim it becomes "the next
  block", read directly from block numbers. The race keeps the same 200 ms shape, so the conclusions should carry over.
  Re-verify by running the recorder across the upgrade.
- **Polling:** `eth_getLogs` against `"pending"` may change meaning after Denim. The canonical `logs` subscription should then
  arrive about as early as `pendingLogs` does now.

## 6. Dominant bottleneck

**The economics are the bottleneck; better information and faster execution are not.**

- **Information:** push gives the update at the same moment everyone else can have it. No source we tested shows anything
  earlier than the Flashblock that contains the update.
- **Execution:** push plus local simulation (89 ms p50) is well inside the 200 ms window.
- **Economics:** once inside the window, winning needs the top priority fee. The value left after fees, failed attempts and
  outliers is close to zero (section 4).

## 7. Verdict and evidence

**STOP** pursuing faster detection or execution for Morpho Base liquidations. The evidence is in sections 2 to 4, and
the frozen gates from the cbETH/USDC study are unchanged. **RESEARCH** remains open for one hypothesis: predicting oracle updates
before they land on-chain, the one route to information competitors do not get from the same stream.

---

The recorder and benchmark code are not part of this repository. Endpoint behaviour is as observed on the measurement date and may have changed, particularly after Denim.
