// Emit stellar-CLI argument values for the Semaphore lib flow, from fixtures/proof-depth2.json,
// in Soroban BN254 encoding. vk is a JSON object; points/signals are hex (no 0x).
import { readFileSync } from "fs"
import { fileURLToPath } from "url"
import { dirname, resolve } from "path"
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..")
const f = JSON.parse(readFileSync(resolve(root, "fixtures/proof-depth2.json"), "utf8"))

const be32 = (d) => BigInt(d).toString(16).padStart(64, "0")
const G1 = (p) => be32(p[0]) + be32(p[1])
const G2 = (p) => be32(p[0][1]) + be32(p[0][0]) + be32(p[1][1]) + be32(p[1][0])
const [rt, nul, msg, scope] = f.publicSignals

console.log(JSON.stringify({
    depth: 2,
    root: be32(rt),
    nullifier: be32(nul),
    message: be32(msg),
    scope: be32(scope),
    a: G1(f.proof.A),
    b: G2(f.proof.B),
    c: G1(f.proof.C),
    vk: {
        alpha: G1(f.vkey.alpha),
        beta: G2(f.vkey.beta),
        gamma: G2(f.vkey.gamma),
        delta: G2(f.vkey.delta),
        ic: f.vkey.IC.map(G1),
    },
}))
