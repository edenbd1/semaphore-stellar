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

## Custom account (`contracts/account`)

`SemaphoreAccount` is a Soroban custom account whose authorization *is* an anonymous
Semaphore membership proof. `__check_auth` enforces:

1. the proof verifies against the account's configured group root + verification key;
2. the **signal is bound to `signature_payload`** — the circuit's `message` must equal
   `keccak256(signature_payload) >> 8` (Semaphore's field hash), so a proof authorizes
   exactly this transaction and cannot be replayed on another;
3. the **nullifier is unused** (persistent registry) — each membership authorizes at most
   once per scope.

So whoever controls a group member identity can authorize the account without revealing
*which* member. Tested against a real proof: valid proof authorizes, replayed nullifier
rejected, wrong payload rejected (`cargo test -p semaphore-account`).

## Confidential multisig (`contracts/multisig`)

`SemaphoreMultisig` is a confidential **k-of-n threshold** custom account. To authorize a
transaction, `k` *distinct* members each submit a Semaphore proof; `__check_auth`:

1. binds every proof's signal **and** scope to `signature_payload`
   (`message = scope = keccak256(payload) >> 8`) — the proofs authorize exactly this
   transaction, and not a weaker rule (anti rule-downgrade);
2. verifies each proof against the group root + verification key;
3. requires the nullifiers to be **pairwise distinct** — a nullifier is deterministic in
   `(scope, member)`, so distinct nullifiers for one payload prove `k` distinct members
   signed *without revealing which*;
4. requires `k >= threshold`.

Tested (real 2-of-3 proofs): threshold met authorizes, below-threshold rejected, duplicate
member rejected, wrong payload rejected. **Budget:** each proof is one pairing check
(~29M instructions), a 2-of-n check ~58M — so a single transaction fits ~3 proofs under the
100M per-tx limit; larger thresholds need batching (future work).

## Layout

```
contracts/verifier   Groth16/BN254 verifier (verify_encoded + verify contract)
contracts/semaphore  generic ISemaphore-aligned library (groups, nullifiers, validate_proof)
contracts/account    Semaphore-authorized custom account (__check_auth)
contracts/multisig   confidential k-of-n threshold account (__check_auth, single-tx)
contracts/dao        proposal-based confidential multisig (async approvals, no tx ceiling)
fixtures             ground-truth proof + ceremony verification key (depth 2)
tools                fixture generators (reproduce fixtures from real proofs)
scripts              testnet deploy + invoke + full lib flow
```

## Roadmap

- **Phase 1 — done:**
  - real ceremony proof verifies on Soroban BN254 (unit test + **live on testnet**);
  - `verify()` fits the budget (~29.1M instructions, under the 100M per-tx limit);
  - `__check_auth` custom account: signal binding + nullifier replay guard.
- **Phase 2 — done:** generic `Semaphore` library (`contracts/semaphore`) — groups
  (off-chain tree + on-chain root), per-depth vkey registry, `verify_proof` /
  `validate_proof`, canonical-input (`< r`) checks, group-scoped nullifier registry.
  **Full flow live on testnet** (deploy → register_vkey → create_group → update_root →
  validate a real proof → replay rejected with `NullifierAlreadyUsed`); see `DEPLOYMENT.md`.
- **Phase 3 — done:** confidential **k-of-n threshold** account (`contracts/multisig`,
  single-tx) + **proposal-based confidential multisig** (`contracts/dao`, async approvals —
  one proof per tx, no ~3-proof ceiling). Both live on testnet (see `DEPLOYMENT.md`).
- **Next:** epoch-scoped root-validity window + `ISemaphore` event parity; full on-chain
  `__check_auth` auth-entry flow; the **confidential relay/signing layer** (payer-privacy edge).

## License

MIT — see [LICENSE](./LICENSE). Compatible with Semaphore (MIT, Ethereum Foundation / PSE).
