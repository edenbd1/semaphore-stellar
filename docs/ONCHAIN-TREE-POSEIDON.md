# On-chain Merkle tree & circomlib Poseidon — feasibility note

**Goal:** maintain a group's Merkle tree *on-chain* (members added via contract calls, root
computed on-chain) so `validate_proof` needs no trusted off-chain root. This requires a
circomlib-compatible Poseidon (BN254, 2 inputs) on-chain, matching @zk-kit's LeanIMT.

**Crypto feasibility: YES.** CAP-75's `poseidon_permutation` is fully parameterized (t, MDS,
round constants) and reproduces circomlib's Poseidon. `tools/gen-poseidon-params.mjs` extracts
circomlib's exact t=3 constants + test vectors (e.g. `poseidon2([1,2]) =
7853200120776062878684798364095072458815029376092732009249414926327459813530`).

**Toolchain blocker (soroban-sdk 29.0.1): currently NOT callable from a contract.**
`poseidon_permutation` lives on `CryptoHazmat`, which is `pub(crate)` in sdk 29 (so is
`Env::crypto_hazmat()`), and `env::internal` is a private module. The only published wrapper,
`soroban-poseidon`, tops out at 28.0.0 and depends on `soroban-sdk ^28` (it calls the then-public
`crypto_hazmat()`), so it does not build against sdk 29.

**Options to unblock (decide later):**
1. Wait for / request a public Poseidon wrapper in soroban-sdk ≥29 (the SDK docs already point to
   `rs-soroban-poseidon`), then build the on-chain LeanIMT on top.
2. Pin the whole stack to soroban-sdk 28.x + `soroban-poseidon` 28 (backwards step; our deployed
   contracts are on 29).

**Until then:** the library uses the off-chain-tree + on-chain-root model (admin pushes roots),
hardened with a root-validity window (see the `semaphore` contract) so it is safe and usable.
