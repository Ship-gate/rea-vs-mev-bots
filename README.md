# Reverse engineering is powerful. It does not win against MEV bots.

*A read-only study, October–November 2026. No transaction was signed, broadcast or funded at any point.*

We spent about two weeks pointing [REA](https://github.com/morluto/rea) ("Reverse Engineer Anything", MIT, v6.7.0) at live MEV contracts on Ethereum, then tried to turn what we learned into a profitable strategy on paper. The first half worked better than expected. The second half failed everywhere we looked.

This repository is the write-up, the supporting documents, the on-chain data behind the tables and a kit to reproduce the static analysis.

## The short version

- REA and a careful hand analysis recovered the full structure of two well-known MEV contracts: entry gates, dispatch design, external-call surface, callback authentication and gas techniques.
- That knowledge let us rebuild the on-chain math accurately. Our liquidation model matched real collateral seizures within 1% on 82.9% of the 140 rows with recoverable pre-state, and within 5% on all of them. A fee model for just-in-time liquidity predicted observed fees within 1% on 54 of 55 fresh cases.
- None of it produced a strategy worth running. Across seven liquidation markets on five chains, the best case after removing the three largest events was about **$0.22 per day**. Every candidate was rejected.
- The reason is simple. A bot's contract is the part anyone can read. Its edge is in what the contract does not contain: the off-chain decision logic, the inclusion relationships, the latency budget, the capital, and auctions that hand nearly all of the surplus to someone else.

## What REA is

REA is a single package that exposes reverse-engineering workflows as a CLI and as an MCP server for coding agents. It covers native binaries, WebAssembly, EVM bytecode, mobile apps, network captures and more. Two rules hold across all of it: analysis runs locally, and static inspection reads bytes without executing the target.

Its defining choice is about honesty. Every result is tagged as an observation, a derivation, an inference or an unknown, and the tool is built so those categories do not blur. On an adversarial target that matters more than decompiler quality, because a confident wrong answer looks exactly like a right one. A longer review is in [docs/rea-deep-dive.md](docs/rea-deep-dive.md).

## Case 1: a contract written to not be read

Target: `0x1f2F10D1C40777AE1Da742455c65828FF36Df387`, labeled by explorers as a jaredfromsubway MEV bot. 13,835 bytes of runtime, verified byte-identical to mainnet code at block 26,226,166.

REA's EVM interface command returned **zero inferred functions**, with its documented caveat that an empty result does not prove there is no callable interface. That empty result was the finding. Selector-recovery tools look for the standard Solidity dispatch table, and this contract does not have one.

One layer down, by static disassembly of the same bytes:

- **Entry gate.** The caller must be the operator wallet, and the low byte of `msg.value` must equal the low byte of the block timestamp. The second check ties each call to its target block.
- **Dispatcher.** `JUMP(firstCalldataByte × 64)`. One leading command byte selects a handler. Of 256 possible values, 79 land on a live handler and 177 abort.
- **Calldata templates.** 42% of the contract (5,936 bytes in 77 runs) is data, not code: pre-built argument blocks that handlers copy into memory before a call. The whole binary contains only two `PUSH4` constants.
- **Callback authentication.** Calls that fail the entry gate fall through to a router that recomputes the expected pool address with `CREATE2` and admits the caller only if it is that pool and the transaction origin is the operator.
- **Call surface.** WETH, USDC, USDT, the Balancer V2 vault, Uniswap V3 and V2 swaps.

About 30 handlers reach an external call whose operands did not resolve statically. They are reported as unknown. The full map with byte offsets and confidence tags is in [docs/static-decomposition-0x1f2f10.md](docs/static-decomposition-0x1f2f10.md).

What this recovers is mechanism. Which command byte to send, in which bundle, at what bid, is decided off-chain and is not in the bytecode.

## Case 2: the bot behind a "$37 million profit" claim

Target: `0xA69babEF1cA67A37Ffaf7a485DfFF3382056e78C`, tagged "MEV Bot" by Etherscan, with about 6.96 million transactions.

- It is a proxy vault whose single business entry delegates into one of hundreds of per-strategy logic contracts: 384 distinct targets in 6,040 sampled transactions.
- About 94% of successful sampled transactions are one-leg trades against a single pool from held inventory, with zero priority fee and a direct payment to the block builder. In 210 of 236 tested trades the bot was the first to touch that pool in the block. That is not sandwiching. It fits arbitrage against a centralized exchange, where the other leg is off-chain.
- Just-in-time liquidity was reconstructed end to end and replayed on a fork with exact balance deltas. Across 116 sampled pairs the bot earned about 3.5 ETH in fees and paid about 2.85 ETH to builders and 0.88 ETH in gas. Marked at the pool price, that is break-even to negative.
- A widely viewed 2023 video put the bot's profit at $37 million. That figure is the token balance shown on the explorer, not profit.

The largest activity class has no observable profit at all, because the hedge happens on an exchange we cannot see. We could describe exactly how the bot trades and still not know whether a given trade made money.

## Then we tried to make money with it, on paper

Everything below is historical replay or passive observation. Replay success was never counted as a live win.

### Liquidations

We rebuilt Aave V3 liquidation math and checked it against 160 real mainnet liquidations over about 30 days. The math held. The economics did not.

In the exact backtest, 92 events produced $4,023.77 of modeled headroom, of which **$4,016.11 was one event**. Without the top three events the total was $1.33.

The same pattern appeared in every market screened. Ranking rule: net after costs, three largest events removed, then stressed at 10% capture and a 50% payment to the builder.

| Market | Window | Events | Gross bonus | Without top 3 | Net per day, stressed | Verdict |
|---|---|---|---|---|---|---|
| Arbitrum Aave V3 | 7.1 days | 93 (14 priced) | $764.30 | $36.63 | $0.22 | Rejected |
| Sonic Aave V3 | 90 days | 17 | $490.49 | $148.06 | $0.078 | Rejected |
| Gnosis Aave V3 | 90 days | 9 | $217.55 | $0.17 | about $0 | Rejected |
| Compound III Base | 90 days | 62 | incumbent-dominated | n/a | about $0 | Rejected |
| Morpho Base late tail | closed | 2,844 | 83% of profit in one 24-hour cascade | negative without top 1% | n/a | Rejected |
| Aave Base | 30 days | n/a | $46.07 winner net | negative without top 1% | n/a | Rejected |
| cbETH reactive, Base | closed | n/a | n/a | n/a | $3.27 at best | Stopped |

Each market is dominated by one incumbent holding 55% to 85% of events. The rest is a tail worth tens of dollars per quarter. Full detail is in [docs/liquidation-profit-discovery.md](docs/liquidation-profit-discovery.md), with the event data in [data/](data).

### Speed

On Base, 98.9% of cbETH liquidation value went to transactions landing in the next 200-millisecond block slice after the oracle update. Our pipeline took 334 ms. We cut it to 178 ms with push subscriptions and to 89 ms with a warm local fork.

It changed nothing. Expected value without the top three events stayed at $3.27 per day at 10% capture, and went negative at double the priority fee. Push delivery beat polling by a median 319 ms, but push is available to every subscriber, so it is not an edge.

### Auctions

Where liquidations run through an order-flow auction, replay showed winners bidding a median **100% of surplus and keeping 0.78%**. Whatever information advantage exists is sold back to the protocol by design.

### Arbitrage

No repeatable, newcomer-capturable path was found across the routes screened.

- Apparent 20–24 ms detection leads turned out to be delivery skew between two subscriptions to transactions that were already committed.
- An apparent 11 basis point ETH price premium on one exchange shrank to under 1 basis point once the stablecoin conversion was priced in, before fees.
- Sampled cross-venue round trips on Base were negative before gas.

Sandwiching was excluded from implementation from the start.

## Why reading the bot is not enough

1. **The strategy is off-chain.** The contract is an executor. The decision of what to execute lives on a server.
2. **Inclusion is a relationship.** Winners pay builders directly, sometimes 50–76% of gross, and some appear builder-affiliated. A newcomer with identical code does not get identical placement.
3. **The latency budget is physical.** When the window is one block slice, colocation and pre-inclusion visibility decide the race.
4. **Capital is part of the edge.** One target pre-funded an estimated $20–30 million of inventory and used no flash loans.
5. **Surplus is auctioned away.** Where an auction exists, the winner's margin is under 1%.
6. **Profit is jackpots.** Remove the three largest events and most markets are worth less than their gas.
7. **Incumbents are already near optimal.** Reconstructed public patterns are almost fully captured.

Reverse engineering closes the knowledge gap. It does not close any of these.

## What reverse engineering is good for

- **Understanding and review.** Recovering the structure of an unverified contract, including unsafe patterns you should not copy.
- **Checking public claims.** Holdings are not profit. "Front-running" was not what the sample showed.
- **Specific techniques, measured.** Callback authentication with a calldata-supplied salt saved 621 gas per callback in our benchmark. A fee model that predicts within 1% is a usable filter.
- **Knowing when to stop.** The most valuable output of this study is a list of things not to build.

## Method

- Read-only throughout: public bytecode, public transactions, public RPC and explorer data.
- Where kill criteria were pre-registered, they were frozen before the test rows were read and not moved afterwards.
- Never ranked by gross. Always net, with the largest events removed.
- Unknown stays unknown. It is never treated as zero or filled with a guess.

## Limits

- Capture probability was never measured for any candidate, because measuring it requires real submissions. The 5–10% figures are stress assumptions.
- Windows are short and samples are small: 7 to 90 days, sometimes 9 events.
- 79 of 93 Arbitrum events could not be priced without archive state.
- Builder payments were under-counted where transaction traces were unavailable.
- Operator identities and builder affiliations are inferences from public labels and flows, not established facts.
- These are results from one small team at one point in time. Markets change, and a well-capitalized, colocated operator faces different numbers.

## Reproduce the static analysis

Requires Docker with an x86-64 Linux runtime. On Apple Silicon we used Colima with Rosetta.

```sh
colima start rea-research --vm-type vz --vz-rosetta --cpus 2 --memory 4 --disk 20
docker --context colima-rea-research build --platform linux/amd64 --tag rea-research:6.7.0 kit
bash kit/run-analysis.sh
```

The container runs with networking disabled, a read-only filesystem and all capabilities dropped. To inspect a different contract, replace `kit/input/runtime.hex` with its runtime bytecode as hex text. Our recorded output is in [kit/output/interface.json](kit/output/interface.json).

## What is not here

No trading engine, executor contract, bundle submission code or private infrastructure detail is published. This repository is for understanding, not for running a bot.

## License

Code and scripts: MIT, see [LICENSE](LICENSE). Text and data: [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

REA is a separate project by its own authors. This study is independent and not affiliated with it.
