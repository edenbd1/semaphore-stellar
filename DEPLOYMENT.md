# Testnet deployment

The Phase-1 verifier is deployed and verified live on Stellar **testnet**.

| | |
|---|---|
| Network | Testnet (`Test SDF Network ; September 2015`) |
| Contract ID | `CCMZJTOGVVXOAL5C72INJYGK4M4V2MSPGB3NH2GM4PRBNW7AIVQMZK5N` |
| Deploy tx | [`bbfdf070…`](https://stellar.expert/explorer/testnet/tx/bbfdf0700be0400c7999db7dd7ab02642b7e654b41b07e8f85ba2b6759ef3b99) |
| `verify()` tx (returns `true`) | [`cd21cebd…`](https://stellar.expert/explorer/testnet/tx/cd21cebd99e308916d6b3b7a2cd9ca53ea496447cb8e7073aaf52ec4f28acd71) |

Invoking `verify()` with the real Semaphore ceremony proof returns **`true`** on-chain,
confirming that an Ethereum-ceremony Groth16/BN254 proof verifies unchanged on Stellar.

## Reproduce

```bash
scripts/deploy-testnet.sh                 # build + deploy, prints a CONTRACT_ID
scripts/invoke-verify.sh <CONTRACT_ID>    # read-only simulation -> true
scripts/invoke-verify.sh <CONTRACT_ID> semaphore-dev --send   # persisted tx
```

## Semaphore library (Phase 2) — full flow live on testnet

| | |
|---|---|
| Contract ID | `CBW3YM227O5VHLGVYE46QEMKA5H5ZOMG2EESKLLWX7X7RQ6MG3DQOZWO` |
| Deploy tx (with constructor admin) | [`a5b4cb8e…`](https://stellar.expert/explorer/testnet/tx/a5b4cb8ef52ca7d4950bfb87173e13c721cd846c5f2c70f5878db0811914db3b) |
| `register_vkey(2, vk)` | [`6095e1b7…`](https://stellar.expert/explorer/testnet/tx/6095e1b7e23176f12baea9dce2016f4e29a9101dd47663148ed516a79c9ca667) |
| `create_group` | [`64484969…`](https://stellar.expert/explorer/testnet/tx/64484969d32b7bca1c82d7a22ac2b3f71e77b777bdc4cbe48c650094632904c9) |
| `validate_proof` (real proof → `validated` event) | [`d1b9c932…`](https://stellar.expert/explorer/testnet/tx/d1b9c9323b212cff29307e41b904f00bdb6b703bdfe6901963c6196415b9a695) |
| Replaying the same nullifier | rejected on-chain with `Error(Contract, #7)` = `NullifierAlreadyUsed` |

Run it: `scripts/semaphore-flow.sh`.
