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
