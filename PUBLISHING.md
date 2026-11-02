# Publishing to crates.io

Two crates are published, the verifier first (the library depends on it):
`semaphore-stellar-verifier`, then `semaphore-stellar`. (The name `semaphore` is taken on
crates.io, so the packages use the `semaphore-stellar*` names; the Rust lib names are unchanged,
so `use semaphore::...` / `use semaphore_verifier::...` still work.)

## One-time
Create an API token at https://crates.io/settings/tokens, then:

```bash
cargo login <TOKEN>
```

## Publish (order matters)

```bash
# from the repo root
cargo publish -p semaphore-stellar-verifier
# wait ~30s for crates.io to index it, then:
cargo publish -p semaphore-stellar
```

Notes:
- Both are prototype / unaudited; bump `version` in each crate's Cargo.toml for later releases.
- The downstream repos (multisig / relay / groups) vendor the verifier under the old package
  name `semaphore-verifier`; if you re-sync them from here, update their dependency to
  `semaphore-stellar-verifier`.
