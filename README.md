# Semaphore for Stellar

A faithful port of [Semaphore v4](https://github.com/semaphore-protocol/semaphore) to
Soroban / Stellar: anonymous group membership + an epoch-scoped nullifier + an arbitrary
signal, verified on-chain.

Semaphore v4 uses **BN254** (Ethereum's `alt_bn128`) with Groth16. Soroban's BN254 host
functions are byte-for-byte EVM-compatible, so a proof and verification key produced by
**Ethereum's Semaphore trusted-setup ceremony verify unchanged on Stellar** — no new
ceremony required.

## Status — Phase 1 (risk-killer) ✅

The first milestone is to prove the core assumption end to end: *a real Semaphore ceremony
proof verifies on-chain via Soroban's BN254 pairing check.*

`contracts/verifier` implements a Groth16/BN254 verifier for Semaphore's 4 public inputs
(`[merkleTreeRoot, nullifier, hash(message), hash(scope)]`, so the IC vector has 5 points)
and is tested against a **real proof generated with Ethereum's ceremony artifacts**:

```
test test::verifies_real_semaphore_proof ... ok   # real ceremony proof verifies
test test::rejects_tampered_public_signal ... ok  # tampering breaks verification
```

```bash
cd contracts/verifier && cargo test       # run the tests
stellar contract build                    # build the deployable WASM (exports `verify`)
```

## How a proof is verified

`groth16_verify` checks the pairing identity in product form:

```
e(-A, B) · e(alpha, beta) · e(vk_x, gamma) · e(C, delta) == 1
vk_x = IC[0] + Σ pub_signals[i] · IC[i + 1]
```

via `env.crypto().bn254().pairing_check`. Point encoding is Soroban's uncompressed
BN254 format: G1 = `X‖Y` (64 bytes), G2 = `Xc1‖Xc0‖Yc1‖Yc0` (128 bytes, Fp2 imaginary
component first), scalars are 32-byte big-endian field elements.

## Reproducing the test fixture

The fixture in `contracts/verifier/src/fixture.rs` is generated, not hand-written. It is
produced by generating a real proof with Semaphore's TS libraries, re-verifying it with
`snarkjs.groth16.verify` against the depth-2 ceremony vkey (ground truth), then encoding
everything for Soroban:

```bash
cd tools && npm install && node gen-fixture.mjs
```

## Layout

```
contracts/verifier   Groth16/BN254 verifier (Phase 1 prototype)
fixtures             ground-truth proof + ceremony verification key (depth 2)
tools                fixture generator (reproduces fixtures + the Rust fixture module)
```

## Roadmap

- **Phase 1 — done:** real ceremony proof verifies on Soroban BN254.
- **Phase 2:** generic library — group management, nullifier registry (Persistent,
  epoch-scoped), arbitrary signal bound to the proof, `ISemaphore`-aligned API.
- **Phase 3+:** `__check_auth` custom-account integration; confidential multisig and
  confidential relay/signing layers on top.

## License

MIT — see [LICENSE](./LICENSE). Compatible with Semaphore (MIT, Ethereum Foundation / PSE).
