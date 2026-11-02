# Semaphore for Stellar

A faithful port of [Semaphore v4](https://github.com/semaphore-protocol/semaphore) to
Soroban / Stellar: anonymous group membership + an epoch-scoped nullifier + an arbitrary
signal, verified on-chain.

Semaphore v4 uses **BN254** (Ethereum's `alt_bn128`) with Groth16. Soroban's BN254 host
functions are byte-for-byte EVM-compatible, so a proof and verification key produced by
**Ethereum's Semaphore trusted-setup ceremony verify unchanged on Stellar** — no new
ceremony required.

This repository is the **core library**. Confidential applications built on it live in their
own repositories:

- **[semaphore-stellar-multisig](https://github.com/edenbd1/semaphore-stellar-multisig)** —
  confidential custom accounts: single-signer, k-of-n threshold, and a proposal-based DAO.
- **[semaphore-stellar-relay](https://github.com/edenbd1/semaphore-stellar-relay)** — the
  payer-privacy relay: a relayer pays the fee for an anonymous member and is tipped from a pool.
- **[semaphore-stellar-groups](https://github.com/edenbd1/semaphore-stellar-groups)** — a
  *trustless* on-chain group tree: the Merkle root is computed on-chain (LeanIMT + circomlib
  Poseidon), so no admin can forge it.

## Contracts

- **`contracts/verifier`** — a Groth16/BN254 verifier for Semaphore's 4 public inputs
  (`[merkleTreeRoot, nullifier, hash(message), hash(scope)]`, IC has 5 points). Exposes a
  reusable `verify_encoded` plus a `verify` contract entry. Verified against a **real ceremony
  proof** on-chain (`cargo test -p semaphore-verifier`; also live on testnet).
- **`contracts/semaphore`** — the generic `ISemaphore`-aligned library: groups (off-chain tree
  + on-chain root), a per-depth verification-key registry, `verify_proof` (stateless) and
  `validate_proof` (consumes a group-scoped nullifier), with canonical-input (`< r`) checks.

## How a proof is verified

`groth16_verify` checks the pairing identity in product form:

```
e(-A, B) · e(alpha, beta) · e(vk_x, gamma) · e(C, delta) == 1
vk_x = IC[0] + Σ pub_signals[i] · IC[i + 1]
```

via `env.crypto().bn254().pairing_check`. Point encoding is Soroban's uncompressed BN254
format: G1 = `X‖Y` (64 bytes), G2 = `Xc1‖Xc0‖Yc1‖Yc0` (128 bytes, Fp2 imaginary first),
scalars are 32-byte big-endian field elements. Budget: a single verification is ~29M
instructions, well under the 100M mainnet per-tx limit.

## Test & build

```bash
cargo test            # 8 tests (verifier 3, semaphore 5)
stellar contract build
```

The full library flow runs live on testnet (deploy → register_vkey → create_group →
update_root → validate a real proof → replay rejected); see `DEPLOYMENT.md` and `scripts/`.

## Reproducing the test fixture

The fixture is generated, not hand-written — a real proof re-verified with `snarkjs.groth16.verify`
against the depth-2 ceremony vkey, then encoded for Soroban:

```bash
cd tools && npm install && node gen-fixture.mjs
```

## Roadmap

- **Done:** Groth16/BN254 verifier (real ceremony proof, on-chain) + the generic library, live
  on testnet; **root-validity window + admin transfer + `ISemaphore` getters**; confidential
  multisig and payer-privacy relay (incl. a real on-chain `__check_auth` and an explicit
  fee-bump) in their own repos.
- **On-chain trustless tree — done** (in
  [semaphore-stellar-groups](https://github.com/edenbd1/semaphore-stellar-groups), on sdk 28):
  LeanIMT + circomlib Poseidon computed on-chain, verified live on testnet; see
  [docs/ONCHAIN-TREE-POSEIDON.md](./docs/ONCHAIN-TREE-POSEIDON.md).
- **Next:** fold the on-chain tree into the sdk-29 library once the SDK exposes a public
  Poseidon; SEP drafts (membership interface + relay protocol).

## License

MIT — see [LICENSE](./LICENSE). Compatible with Semaphore (MIT, Ethereum Foundation / PSE).
