// Emit the stellar-CLI argument values for verify(), from fixtures/proof-depth2.json,
// in Soroban BN254 encoding (hex, no 0x). Vectors are JSON arrays of hex strings.
import { readFileSync } from "fs"
import { fileURLToPath } from "url"
import { dirname, resolve } from "path"
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..")
const f = JSON.parse(readFileSync(resolve(root, "fixtures/proof-depth2.json"), "utf8"))

const be32 = (d) => BigInt(d).toString(16).padStart(64, "0")
const G1 = (p) => be32(p[0]) + be32(p[1])
const G2 = (p) => be32(p[0][1]) + be32(p[0][0]) + be32(p[1][1]) + be32(p[1][0])

const out = {
    pub_signals: JSON.stringify(f.publicSignals.map(be32)),
    a: G1(f.proof.A),
    b: G2(f.proof.B),
    c: G1(f.proof.C),
    vk_alpha: G1(f.vkey.alpha),
    vk_beta: G2(f.vkey.beta),
    vk_gamma: G2(f.vkey.gamma),
    vk_delta: G2(f.vkey.delta),
    vk_ic: JSON.stringify(f.vkey.IC.map(G1)),
}
console.log(JSON.stringify(out))
