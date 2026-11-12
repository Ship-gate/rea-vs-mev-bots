# Reverse Engineer Anything, Honestly

### A deep technical look at REA — and what it recovers (and can't) from a live on-chain MEV contract

REA (`morluto/rea`, v6.7.0, MIT) bills itself in four words: *Reverse Engineer Anything.* The tagline is "see a feature you like, understand how it works, down to the binary level." That is a big promise, and most tools that make it quietly mean "down to a disassembly dump you still have to read yourself." REA is more interesting than that — not because it decompiles better than Ghidra, but because of what it refuses to claim. Its defining design choice is epistemic: every result is tagged as an *observation*, a *derivation*, an *inference*, or an *unknown*, and the tool is built, top to bottom, to never let those categories blur.

This piece takes REA apart along the axis I know best from a week of using it — its offline EVM lane — and uses one real, adversarial target to show where the promise holds and where it hits a wall. The target is the runtime of `0x1f2F10D1C40777AE1Da742455c65828FF36Df387`, the contract the well-known `jaredfromsubway.eth` operator wallet drives. It is a good stress test precisely because it was written to resist exactly this kind of inspection.

## One CLI, one MCP server, twenty-odd targets

REA is a single npm package, `rea-agents`, that exposes the same analysis workflows two ways: a terminal CLI (`rea <command>`) and a Model Context Protocol server your coding agent calls directly. `npx rea-agents setup` registers the MCP server and installs matching agent instructions for Claude Code, Codex, Cursor, Gemini CLI, Grok and others. The two front-ends share one implementation, so a finding you get by asking your agent is the same finding — same evidence, same limitations — you get from a shell pipe.

The breadth is unusual for one tool. REA's capability matrix spans native binaries (via your own Hopper, Ghidra or IDA), Go and Rust build metadata, WebAssembly, EVM bytecode, SQLite databases, recorded Linux crash cores, JavaScript and Electron apps, websites over CDP, saved HAR/SAZ network captures, .NET assemblies, Android APKs and live devices over adb, JEB projects, Flutter snapshots, firmware images, packet captures, logic-analyzer signals, and live process behavior. Two rules hold across all of them: analysis runs locally, and static inspection reads the supplied bytes without executing the target.

Governance is enforced in `AGENTS.md` with a fail-closed rule most projects would consider paranoid: *do not integrate a capability while material scope, authority, side-effect, or verification questions remain unresolved.* Installers must not upgrade unrelated software — not Node, not Homebrew, not Ghidra. Ghidra and IDA are bring-your-own, run in ephemeral databases, and are never allowed to mutate the target's bytes. For a reverse-engineering tool — a category that historically ships "just pip install this kitchen sink" — that restraint is the product.

## Architecture: dependencies flow inward

REA's internals are a clean hexagonal design. Dependencies flow strictly inward: domain semantics and contracts at the core, then provider adapters (the Ghidra bridge, the EVMole binding, the JADX bridge), then shared application workflows, and finally the CLI and MCP adapters at the edge. Engine-specific code is forbidden from leaking into the domain layer. The repository ships `docs/architecture.mermaid` and a module-boundary verifier (`verify:module-boundaries`) in CI to keep the arrows pointing the right way.

The subtler discipline is *ownership*. Every parsed fact has exactly one owner, and downstream consumers must use that owner's interpretation rather than re-parsing raw source. Resources whose cleanup failed are retained, not leaked and forgotten. A PID plus an executable path is explicitly *not* treated as proof of process ownership after exit or reuse. This is the kind of thing you only build after being burned by a decompiler that reported a stale result as live, and it is the mechanism behind REA's evidence guarantees.

## The EVM lane, in detail

The command is `inspect_evm_interface` (`rea inspect-evm-interface ./runtime.hex hex --json`). It infers an interface — dispatch selectors, their byte offsets, inferred argument types, and state mutability — from a single explicitly selected local bytecode carrier. It is backed by a pinned copy of the open-source **EVMole 0.9.3**, runs only on Linux x64, and requires the host to provide `util-linux`'s `prlimit`. REA installs nothing and touches no chain.

What makes this lane worth studying is its evidence contract, spelled out in `docs/evm-bytecode.md`:

- **Selectors, argument strings and mutability are inferences, not a verified ABI.** Function names and signatures are never looked up. Optimized code can yield different mutability candidates.
- **Three distinct hashes, never conflated.** The *carrier* SHA-256 identifies the exact file you fed in, down to casing and whitespace. The *decoded-byte* SHA-256 identifies the actual EVM bytes. Neither is the Ethereum Keccak code hash. The tool will not let you mistake "the file I analyzed" for "the contract deployed on chain."
- **Empty candidates do not prove the absence of a callable interface.** This sentence is in the docs. Hold onto it — the case study turns on it.
- **Offline means offline.** No target execution, no implicit RPC or explorer lookup, no wallet action, no broadcast, no network request of any kind.
- **EOF (EF00) containers** return a typed `unsupported_target` rather than a bluff.

The worker that runs the analysis is jailed: a 256 MiB JavaScript heap, a 3 GiB virtual-address-space ceiling, 30 CPU-seconds, a 30-second wall deadline, 4 MiB maximum input, 16 MiB maximum reply, 1 MiB of combined diagnostics. Limits fail hard with no partial success, and the real verification lane asserts that only the declared Node and `prlimit` launchers run and that any attempted network socket is rejected. You are not trusting a README's word for "offline"; there is a socket-trace test enforcing it.

## Running a Linux-x64-only tool on Apple Silicon

The EVM, pwntools and TShark lanes are Linux-x64-only, which is a problem on an M-series Mac. The workaround is clean enough to be its own reproducible recipe, and it is the part of this exercise I would put in front of any skeptic.

Colima can start a Linux VM backed by Apple's Virtualization framework with Rosetta providing x86-64 translation. One dedicated profile — 2 CPUs, 4 GiB RAM, a 20 GiB sparse disk — hosts a `linux/amd64` container running the published REA 6.7.0 package. The container is deliberately over-constrained for an analysis box: it runs as the unprivileged `node` user, with **networking disabled**, a **read-only root filesystem and input mount**, a temporary `/tmp`, all Linux capabilities dropped, `no-new-privileges` set, and explicit CPU, memory and PID limits. The Dockerfile pins both the base-image digest and the REA version. No source change was made to the REA checkout.

The result: a Linux-x64 reverse-engineering tool runs, byte-for-byte as published, on Apple Silicon, inside a sandbox that physically cannot phone home — which is exactly the property you want when the thing you are analyzing is hostile.

## Reproduce it end to end

The whole pipeline is six commands and one file. Create the VM profile once, build the pinned image once (the only step that needs the network, to pull the base image and the package), then run analysis offline as many times as you like:

```sh
# one-time: dedicated VM with Rosetta x86-64 translation
colima start rea-research --vm-type vz --vz-rosetta \
  --cpus 2 --memory 4 --disk 20

# one-time: build the pinned linux/amd64 image (needs network)
docker --context colima-rea-research build --platform linux/amd64 \
  --tag rea-research:6.7.0 ./rea-research

# per run: offline, read-only, no network, dropped caps
docker --context colima-rea-research run --rm --platform linux/amd64 \
  --network none --read-only --cap-drop ALL \
  --security-opt no-new-privileges=true --memory 1g \
  -v "$PWD/rea-research/input:/in:ro" \
  rea-research:6.7.0 \
  rea inspect-evm-interface /in/runtime.hex hex --json
```

To analyze any other contract, drop its runtime bytecode into `input/runtime.hex` as hex text (an optional `0x` prefix is fine) and rerun the last command. Stop the VM with `colima stop rea-research` to reclaim its RAM. Because the analysis container has `--network none`, there is no code path by which "inspect this bytecode" can become "touch the chain" — the isolation is structural, not a promise.

## Why the four-category evidence model earns its keep

It is tempting to read REA's observation / derivation / inference / unknown taxonomy as documentation hygiene. The case study shows it is load-bearing. The empty candidate set is an *observation* (EVMole ran and returned nothing). "This contract has no standard selector table" is a *derivation* from that observation plus the disassembly. "It is a MEV executor" is an *inference* from the call surface and gating. And "what each of its 79 commands actually does off-chain" is an honest *unknown*. A tool that collapses those four into one confident paragraph is worse than useless on an adversarial target, because its most dangerous output looks exactly like its most reliable one. REA keeps them apart by construction, which is why its *negative* result on this contract was more informative than a positive result from a tool willing to guess.

## The case study: a contract written to not be read

The target is 13,835 bytes of runtime, decoded-byte SHA-256 `204a4fde…044708`. I fed the hex to `inspect_evm_interface` in the container. It exited 0 and returned **zero inferred function candidates.**

A lesser tool would have shrugged, or worse, hallucinated a plausible-looking ABI. REA did neither. It returned an empty candidate set *with the documented caveat attached*: this does not establish that the contract has no callable interface. That is the honest answer, and it is also a genuine finding — because the reason the set is empty tells you most of what you need to know about the contract.

EVMole, like every selector-recovery tool, looks for the standard Solidity dispatch prologue: load the first calldata word, shift right by `0xe0` to isolate the four-byte selector, and compare it against a table of `PUSH4` constants with `EQ`/`JUMPI`. This contract has no such table. So interface recovery, correctly, finds nothing to recover. The code is not hidden; it simply does not speak ABI.

## Layering down: a hand static decomposition

With EVMole's negative result in hand, the next layer is a plain static disassembly of bytes I already held locally — no VM, no network, no chain. A linear sweep of the 13,835 bytes, with every `PUSH` operand consumed correctly, yields the real structure. I will mark each claim as REA itself would.

**Observation — the prologue is an operator gate.** The first instructions, instruction-aligned from offset 0 and therefore reliable, are:

```
0x0000 CALLVALUE  PUSH1 0xff  AND        ; msg.value & 0xff
0x0004 TIMESTAMP  PUSH1 0xff  AND  EQ    ; == (block.timestamp & 0xff) ?
0x0009 PUSH20 0xae2Fc483527B8EF99EB5D9B44875F005ba1FaE13  CALLER  EQ
0x0020 AND        PUSH1 0x28  JUMPI      ; if (both) goto 0x28
0x0024 PUSH2 0x3476  JUMP               ; else goto fallback region
```

Two gates in series. The caller must be the operator wallet `0xae2Fc4…`, and the low byte of `msg.value` must equal the low byte of the block timestamp. The first is ordinary access control. The second is an *inference*: a cheap, per-block keyed check that a naive simulator or replayer is unlikely to reproduce without noticing it must shape `msg.value` to the block it lands in.

**Observation — the dispatcher is a stride-64 jump table, not an ABI.** Past the gate:

```
0x0028 JUMPDEST  PUSH0  CALLDATALOAD  PUSH0  BYTE   ; first calldata byte
0x002d PUSH1 0x06  SHL  JUMP                        ; JUMP(firstByte << 6)
```

It jumps to `firstByte * 64`. The command is a single leading byte, and each handler occupies a 64-byte-aligned slot. Scanning all 256 possible slots, **79 land on a real `JUMPDEST`** — so the contract exposes on the order of 79 distinct operations, none of which carry a four-byte signature. This is the structural fact EVMole's empty result was pointing at.

**Observation — calls are built from embedded templates.** The whole contract contains only two `PUSH4` constants. Handlers do not construct calldata selector-first; they copy prebuilt byte templates out of embedded data regions. One such region encodes the contract's *own* address, `0x1f2f10d1c407…ff36df387`, as a constant alongside what looks like an ERC-20 `transferFrom` calldata skeleton (the exact selector is an *inference* — that region is data, and a linear sweep misaligns inside it).

**Observation — the external-call surface.** The reliably-extracted `PUSH20` address constants are the contract's counterparties: WETH9 (nine occurrences), USDC and USDT (four each), the **Balancer V2 Vault** `0xba12…2c8` (four), the operator wallet (five), and an all-`0xff` max-address sentinel (nine). WETH plus two major stablecoins plus the Balancer Vault is the liquidity footprint of an on-chain arbitrage/MEV executor.

**Inference — what it is.** Taken together: an operator-gated, hand-written command interpreter, dispatched by a single byte, assembling calls to the deepest ETH/stablecoin venues from embedded templates, with per-block keying that frustrates casual replay. That is a MEV executor built for gas economy and for resistance to exactly the automated classification EVMole attempts. I am stopping at that altitude deliberately: the *mechanism* is a fair read of public bytecode; the operational detail — how it chooses what to do with each command — is not something this analysis recovers, and not something worth manufacturing.

**A discipline note, because it is the article's whole point.** Raw opcode tallies from the linear sweep (111 `CALL`, 71 `SELFDESTRUCT`, 72 `REVERT`, 2 `CREATE`) are *over-counts*: the embedded data regions are parsed as if they were code, so any `0xff` or `0xf1` data byte inflates those numbers. I therefore lean only on the instruction-aligned facts — the prologue, the dispatcher, the `PUSH20` operands — for anything load-bearing, and I flag the histogram as unreliable. REA would make exactly that distinction, and being forced to make it by hand is a good argument for the tool making it for you.

## What REA could add, and the ceiling it won't cross

REA's own EVM documentation is candid about the increments it has *not* yet shipped: source/metadata resolution, proxy resolution, storage and control-flow-graph reconstruction, and fork-explicit execution with traces are tracked as separate work under issue #972, each gated behind the fail-closed rule because each carries "materially different result or execution authority." A CFG/storage increment is precisely what would turn my manual sweep — resolving the computed `×64` jumps into a labeled handler map, and reading the `SLOAD`/`SSTORE` slots — into a first-class REA result with evidence attached. The architecture is clearly built to grow in that direction without compromising the offline guarantee.

There is also a hard ceiling, and it is honest to name it. Static analysis of deployed bytecode recovers *structure* and *mechanism*. It does not recover original source, developer intent, or — for a trading contract — the off-chain logic that decides what each command should be. No amount of layering down changes that. The useful and legitimate output here is a structural map and a characterization; the thing static RE cannot hand you is a running strategy, and that is a feature of reality, not a limitation to engineer around.

## Verdict

REA will not out-decompile a dedicated engine, and it does not try to. What it does — and what very few tools in this space do — is treat "I don't know" and "I inferred this" as first-class, non-negotiable outputs, enforced by architecture, by CI boundary checks, by a socket-trace test for its offline claim, and by a governance rule that blocks capabilities until their authority is resolved. On a contract built specifically to defeat automated interface recovery, that discipline is the difference between a confident, wrong ABI and a correct empty result that points you to the real structure one layer down. For reverse engineering, where the expensive mistakes are the confident ones, honesty is the feature. REA is the rare tool that ships it as the core.

---

*Facts in this article were taken from the `morluto/rea` repository at v6.7.0 and from a local, offline static analysis of publicly deployed bytecode. No contract was executed, no chain state was modified, and no network request was made during analysis. Observations, derivations, inferences and unknowns are labeled throughout.*
