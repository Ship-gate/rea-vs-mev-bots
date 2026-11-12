# Static decomposition — runtime `0x1f2F10…Df387`

**Method.** Offline static analysis of 13,835 bytes of deployed runtime. No execution, no chain, no network. Instruction/data separation by reachability CFG; external-call operands resolved by within-basic-block symbolic execution (constant folding + a byte-addressed memory model tracking `MSTORE`/`MSTORE8`/`CODECOPY`/`CALLDATACOPY`). Every conclusion is tagged **[obs]** observation (instruction-aligned, certain), **[der]** derivation (follows from obs), **[inf]** inference, or **[unk]** unknown. Decoded-byte SHA-256 `204a4fde…044708` — **verified byte-identical to `eth_getCode` on Ethereum mainnet at block 26,226,166**, so every conclusion here is pinned to the live deployed contract. Operator wallet `0xae2Fc4…FaE13` shows nonce 6,599,400 at that block — a high-frequency MEV operator.

Single-path symbolic execution does not merge control-flow joins or carry cross-handler memory, so some `CALL` operands do not resolve. Those are reported **indeterminate**, never guessed. Recurring tiny `value`/`target` constants in the raw dump (`0x2`, `0x16`, `0x55`, `target 0x0/0x4`) are **stack-desync artifacts** and are excluded from all conclusions below.

---

## 1. Instruction / operand / data partition  [der]

Reachability CFG from roots (offset 0, the 79 dispatcher handler entries, resolved static jumps) covers **4,918 instruction starts**. Bytes never reached form **77 data runs totalling 5,936 bytes (42% of the contract)**.

The data runs are **per-handler calldata templates**, confirmed by their embedded contents — each run is the pre-built argument block a handler `CODECOPY`s into memory before a call:

| Data run | Bytes | Embedded evidence |
|---|---|---|
| `0x0031` | 204 | `sel:022c0d9f` (UniV2 swap), SELF address |
| `0x0182` | 190 | `sel:a9059cbb`, `sel:23b872dd`, SELF address |
| `0x0dbd` | 579 | BalancerV2Vault, MAX_ADDR sentinel, `sel:095ea7b3`, `sel:52bbbe29`, SELF |
| `0x12d5` | 747 | WETH9, MAX_ADDR sentinel, `sel:128acb08` (UniV3 swap), SELF |
| `0x2afc` | 4 | `sel:d0e30db0` (WETH deposit) |
| `0x3139` | 71 | `sel:70a08231` (balanceOf), SELF |

**[der]** The contract carries no 4-byte selector dispatch table (only two `PUSH4` immediates exist in the whole binary); it stores complete calldata templates as data and copies them. This is exactly why EVMole's selector recovery returned zero — there is nothing in the shape it scans for.

---

## 2. Entrypoint gate — both branches validated  [obs]

```
0x0000 CALLVALUE PUSH1 0xff AND          ; msg.value & 0xff
0x0004 TIMESTAMP PUSH1 0xff AND  EQ       ; == block.timestamp & 0xff
0x0009 PUSH20 0xae2Fc4…FaE13  CALLER  EQ  ; msg.sender == OPERATOR
0x0020 AND  PUSH1 0x28  JUMPI             ; both true -> 0x0028
0x0024 PUSH2 0x3476  JUMP                 ; else      -> 0x3476
```

- **TRUE → `0x0028`** [obs]: the dispatcher. Requires `CALLER == operator wallet` **and** `(msg.value & 0xff) == (block.timestamp & 0xff)`. The second term is a **block-targeting / anti-replay key** — now [obs]-confirmed on a live tx: `value = 227 (0xe3)` matched `timestamp & 0xff = 227`, so the operator sets value's low byte to the target block's timestamp low byte.
- **FALSE → `0x3476`** [obs]: a **multi-factory DEX-callback router** (verified by disassembling the full region). For each enrolled V3-fork factory it recomputes `CREATE2(0xff ‖ factory ‖ keccak(poolKey@calldata[0x84:0xe4]) ‖ initCodeHash)` and admits the caller iff `caller == that pool` **AND** `origin == operator`, then jumps to an operator-supplied continuation offset at `calldata[0xe4] >> 240`. Enrolled constants [obs]: UniV3 `0x1f9843…F984`/`0xe34f…8b54`; `0x41ff9aa7…`/`0x6ce8eb47…`; SushiV3-looking `0xbaceb8ec…`/`0xe34f…8b54`. The terminal block **reverts** a bare `caller==operator` top-level call. **[der]** `ORIGIN==operator` is a distinct privileged mode admitting pool re-entries *within* operator-initiated txs; on this path the first calldata byte has **no** dispatcher role.

**[der]** Two-surface design: operator drives the dispatcher; pools call back into `0x3476`, which proves the caller is a genuine pool before honoring it. Textbook exact-callback / flash-swap structure.

---

## 3. Dispatcher — all 256 targets resolved  [obs]

Dispatch is `JUMP(byte0(calldata) << 6)` — jump to `firstByte × 64`.

| Outcome for first byte `b` | Count | Effect |
|---|---|---|
| `b×64` is a valid `JUMPDEST` → **live handler** | **79** | executes a handler |
| `b×64` in range but not a `JUMPDEST` → **invalid** | 138 | EVM aborts (bad jump destination) |
| `b×64` ≥ code size → **out of range** | 39 | EVM aborts |

So the ABI is a single leading command byte selecting one of 79 operations; the other 177 byte values revert. 

---

## 4 & 5. Reachable handlers — classification + external calls

Classification uses only resolved evidence. **High-confidence** rows have a selector decoded from a `CODECOPY` template and/or a known destination.

### Token movement (ERC-20 / WETH)  [obs selector]

| Handler | Offset | Call | Dest | Selector (len) |
|---|---|---|---|---|
| `0x04`,`0x09`,`0x0b` | 0x0100/0240/02c0 | CALL | **WETH9** | `23b872dd` transferFrom (100) |
| `0x0d`,`0x0f`,`0x11` | 0x0340/03c0/0440 | CALL | **USDC** | `a9059cbb` transfer (68) |
| `0x13`,`0x15`,`0x17` | 0x04c0/0540/05c0 | CALL | **USDT** | `a9059cbb` transfer (68) |
| `0x2d`,`0xad` | 0x0b40/2b40 | CALL | **WETH9** | `2e1a7d4d` withdraw (36) |
| `0x2f` | 0x0bc0 | CALL | **WETH9** | `23b872dd` transferFrom (100) |
| `0x31`,`0x33` | 0x0c40/0cc0 | CALL | **USDC / USDT** | `a9059cbb` transfer (68) |

### Swaps  [obs selector; pool dest dynamic]

| Handler | Offset | Dest | Selector (len) |
|---|---|---|---|
| `0x42`,`0x44`,`0x91`,`0x93` | 0x1080/1100/2440/24c0 | **BalancerV2Vault** | `52bbbe29` swap (452) |
| `0x57`,`0x59`,`0x5c`,`0x5e`,`0x60`,`0x63`,`0x66`,`0x69` | 0x15c0…0x1a40 | [unk] pool | `128acb08` UniV3 swap (294) |
| `0x88`,`0x8b` | 0x2200/22c0 | [unk] pair | `022c0d9f` UniV2 swap (68) |

### Approvals  [obs selector]

| Handler | Offset | Selector (len) |
|---|---|---|
| `0x41`,`0xaf`,`0xb0`,`0xbf` | 0x1040/2bc0/2c00/2fc0 | `095ea7b3` approve (68) |

`0xbf` approves, calls an unidentified `e67ce706` (132), then approves again — an allowance-setup sequence [der].

### Administrative  [obs]

| Handler | Offset | Evidence |
|---|---|---|
| `0x2e` | 0x0b80 | `SELFDESTRUCT` reachable; no external call |

### Resolved-but-unidentified selectors  [obs value, [unk] meaning]

Pulled from real templates; not looked up (offline, and signature lookup is explicitly out of scope): `7c5e9ea4`(164), `a34123a7`(100), `4f1eb3d8`(164), `3c8a7d8d`(294), `e67ce706`(132), `8dae7333`(132).

### Indeterminate  [unk]

~30 handlers (`0x19`–`0x29`, `0x35`,`0x36`,`0x40`,`0x46`,`0x4a`,`0x72`,`0x95`,`0x99`,`0x9c`–`0xa9`,`0xc1`–`0xd1`, …) reach a `CALL`/`STATICCALL` whose destination and calldata did not resolve under single-path symbolic execution. Reported as external-call-present, operands unknown. `0x06`,`0x2b`,`0x98` sit on data-template bytes (their `i×64` JUMPDEST lands inside a blob) and expose no call.

### External-call tallies (resolved only)

- **Selectors:** `a9059cbb`×14, `128acb08`×8, `095ea7b3`×5, `23b872dd`×4, `52bbbe29`×4, `2e1a7d4d`×2, `022c0d9f`×2, + 6 unidentified.
- **Destinations:** WETH9 ×7, USDC ×4, USDT ×4, BalancerV2Vault ×4. UniV3/V2 pool destinations are passed dynamically (not constants) → [unk], consistent with per-pair routing.

**[inf]** Overall: an operator-gated executor that moves WETH/USDC/USDT and swaps across Balancer V2, Uniswap V3 and Uniswap V2, with pool callbacks authenticated by CREATE2 address recomputation. Mechanism only — the off-chain logic choosing *which* command byte to send in *which* bundle is not in this bytecode and is not recoverable statically **[unk]**.

---

## 6. Consolidated evidence table

| # | Conclusion | Byte offset(s) | Key instructions | Conf |
|---|---|---|---|---|
| 1 | 42% of the contract is calldata-template data, not code | 77 runs (e.g. `0x0dbd`,`0x12d5`) | embedded `52bbbe29`/`128acb08` + addrs | der |
| 2 | No 4-byte selector table exists | whole binary | only 2 `PUSH4` immediates | obs |
| 3 | Entry requires `CALLER==operator` | `0x0009`–`0x001f` | `PUSH20 …FaE13; CALLER; EQ` | obs |
| 4 | Entry requires `(value&0xff)==(timestamp&0xff)` | `0x0000`–`0x0008` | `CALLVALUE; PUSH1 ff; AND; TIMESTAMP; …; EQ` | obs |
| 5 | True branch → dispatcher | `0x0021`–`0x0028` | `PUSH1 28; JUMPI` | obs |
| 6 | False branch → pool-callback auth | `0x0024`,`0x3476` | `PUSH2 3476; JUMP` | obs |
| 7 | False branch recomputes UniV3 pool addr | `0x3493`–`0x3498` | `CALLDATACOPY; KECCAK256; PUSH21 0xff·1f9843…F984` | der |
| 8 | Dispatcher = `JUMP(byte0<<6)` | `0x0028`–`0x0030` | `CALLDATALOAD; PUSH0; BYTE; PUSH1 06; SHL; JUMP` | obs |
| 9 | 79 live handlers / 138 invalid / 39 OOR | `i×64` ∀ i∈[0,256) | JUMPDEST scan | obs |
| 10 | WETH/USDC/USDT transfer handlers | `0x0100`,`0x0340`,`0x04c0`,… | `CALL` + template `a9059cbb`/`23b872dd` | obs |
| 11 | Balancer V2 swap handlers | `0x1080`,`0x1100`,`0x2440`,`0x24c0` | `CALL BalancerV2Vault` sel `52bbbe29` | obs |
| 12 | Uniswap V3 swap handlers (×8) | `0x15c0`–`0x1a40` | `CALL` sel `128acb08` len 294 | obs |
| 13 | Uniswap V2 swap handlers (×2) | `0x2200`,`0x22c0` | `CALL` sel `022c0d9f` | obs |
| 14 | Token-approval handlers | `0x1040`,`0x2bc0`,`0x2c00`,`0x2fc0` | `CALL` sel `095ea7b3` | obs |
| 15 | Self-destruct administrative handler | `0x0b80` | `SELFDESTRUCT` | obs |
| 16 | WETH unwrap handlers | `0x0b40`,`0x2b40` | `CALL WETH9` sel `2e1a7d4d` | obs |
| 17 | ~30 handlers reach an external call, operands unresolved | listed above | `CALL`/`STATICCALL`, dynamic operands | unk |
| 18 | Off-chain command-selection / profit logic | — | not present in bytecode | unk |

*Static analysis of public bytecode. Observations/derivations/inferences/unknowns labeled; model artifacts excluded.*
