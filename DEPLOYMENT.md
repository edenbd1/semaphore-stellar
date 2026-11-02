# Testnet deployment (library)

The verifier and the generic Semaphore library are deployed and verified live on Stellar
**testnet**. (Multisig and relay deployments live in their own repos.)

## Verifier

| | |
|---|---|
| Contract ID | `CCMZJTOGVVXOAL5C72INJYGK4M4V2MSPGB3NH2GM4PRBNW7AIVQMZK5N` |
| Deploy tx | [`bbfdf070…`](https://stellar.expert/explorer/testnet/tx/bbfdf0700be0400c7999db7dd7ab02642b7e654b41b07e8f85ba2b6759ef3b99) |
| `verify()` → `true` | [`cd21cebd…`](https://stellar.expert/explorer/testnet/tx/cd21cebd99e308916d6b3b7a2cd9ca53ea496447cb8e7073aaf52ec4f28acd71) |

Invoking `verify()` with the real Semaphore ceremony proof returns **`true`** on-chain.

## Semaphore library — full flow live

| | |
|---|---|
| Contract ID | `CBW3YM227O5VHLGVYE46QEMKA5H5ZOMG2EESKLLWX7X7RQ6MG3DQOZWO` |
| `register_vkey(2, vk)` | [`6095e1b7…`](https://stellar.expert/explorer/testnet/tx/6095e1b7e23176f12baea9dce2016f4e29a9101dd47663148ed516a79c9ca667) |
| `create_group` | [`64484969…`](https://stellar.expert/explorer/testnet/tx/64484969d32b7bca1c82d7a22ac2b3f71e77b777bdc4cbe48c650094632904c9) |
| `validate_proof` (real proof → `validated`) | [`d1b9c932…`](https://stellar.expert/explorer/testnet/tx/d1b9c9323b212cff29307e41b904f00bdb6b703bdfe6901963c6196415b9a695) |
| Replaying the same nullifier | rejected `Error(Contract, #7)` = `NullifierAlreadyUsed` |

## Reproduce

```bash
scripts/deploy-testnet.sh                 # build + deploy the verifier
scripts/invoke-verify.sh <CONTRACT_ID>    # read-only simulation -> true
scripts/semaphore-flow.sh                 # full library lifecycle
```

## Trustless on-chain tree folded into the library — verified on testnet

The sdk-29 `semaphore` library can source a group's root from the sdk-28 `SemaphoreGroups`
contract (which computes it on-chain from the real members), instead of an admin-pushed root.
`verify_with_onchain_root` cross-reads that root and verifies the proof against it.

| | |
|---|---|
| `semaphore` (sdk 29) | `CASZTKMIG2WUK53A47D5BUWNZJHEPUQEBTZWUKYAAVP5LOFV33P5ZYYW` |
| `SemaphoreGroups` (sdk 28) | `CBMLCUMADQ6H4SLHJAL7LMXJTAOEL74ZGLB5ZDFVL7NL4OHA5DZQ2YV3` |
| Flow | group of 3 real commitments added to `groups` → on-chain root equals the off-chain Semaphore `Group` root → `verify_with_onchain_root` returns `true` |
| `verify_with_onchain_root` tx | [`cbcdfcc2…`](https://stellar.expert/explorer/testnet/tx/cbcdfcc2ef96fc40af1f70c42c6581821e3556113c5a4dff0fc627fd55d5d3cf) → `true` |

The root is computed on-chain from the real members and read cross-contract, so no admin can
forge it. This folds the trustless tree into the library by composition (no SDK downgrade).
