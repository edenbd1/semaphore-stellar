#![no_std]
//! Phase-2: the generic Semaphore library for Soroban.
//!
//! An `ISemaphore`-aligned contract: anonymous group membership + an epoch-scoped nullifier
//! + an arbitrary signal, verified with Groth16/BN254 against Ethereum's Semaphore ceremony.
//!
//! Architecture (v1): the Merkle tree is maintained **off-chain** (by the group admin / an
//! indexer); the contract stores the current **root on-chain** and the group admin pushes new
//! roots as membership changes. This mirrors Semaphore's public inputs exactly
//! (`[merkleTreeRoot, nullifier, hash(message), hash(scope)]`) without needing an on-chain
//! Poseidon that byte-matches circomlib (deferred to a future on-chain-tree mode).

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, vec, Address, BytesN, Env,
    IntoVal, Symbol, Vec, U256,
};
use semaphore_verifier::verify_encoded;

#[contractevent]
pub struct GroupCreated {
    pub group_id: u32,
    pub admin: Address,
}

#[contractevent]
pub struct RootUpdated {
    pub group_id: u32,
    pub root: BytesN<32>,
    pub depth: u32,
}

#[contractevent]
pub struct AdminTransferred {
    pub group_id: u32,
    pub admin: Address,
}

#[contractevent]
pub struct ProofValidated {
    pub group_id: u32,
    pub nullifier: BytesN<32>,
    pub message: BytesN<32>,
    pub scope: BytesN<32>,
}

/// BN254 scalar field order `r`, big-endian. Public inputs must be canonical (`< r`), else the
/// host silently reduces mod `r` and a non-canonical nullifier could evade the replay guard.
const R_BE: [u8; 32] = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
    0x28, 0x33, 0xe8, 0x48, 0x79, 0xb9, 0x70, 0x91, 0x43, 0xe1, 0xf5, 0x93, 0xf0, 0x00, 0x00, 0x01,
];

fn is_canonical(v: &BytesN<32>) -> bool {
    // Big-endian byte comparison equals numeric comparison.
    v.to_array() < R_BE
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotAdmin = 1,
    GroupNotFound = 2,
    VkNotRegistered = 3,
    InvalidRoot = 4,
    NonCanonicalInput = 5,
    InvalidProof = 6,
    NullifierAlreadyUsed = 7,
}

/// Groth16/BN254 verification key for one tree depth.
#[contracttype]
#[derive(Clone)]
pub struct VkData {
    pub alpha: BytesN<64>,
    pub beta: BytesN<128>,
    pub gamma: BytesN<128>,
    pub delta: BytesN<128>,
    pub ic: Vec<BytesN<64>>,
}

#[contracttype]
#[derive(Clone)]
pub struct Group {
    pub admin: Address,
    pub root: BytesN<32>,
    pub depth: u32,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Vk(u32),
    GroupCount,
    Group(u32),
    Nullifier(u32, BytesN<32>),
    /// A superseded root stays valid until this ledger (root-validity window).
    HistoricalRoot(u32, BytesN<32>),
}

/// How long (in ledgers) a superseded root remains accepted, so a proof built against a root
/// that was current when the proof was made still validates after the tree moves on.
/// ~5s/ledger → ~17280 ledgers ≈ 24h.
const ROOT_VALIDITY_LEDGERS: u32 = 17280;

#[contract]
pub struct Semaphore;

#[contractimpl]
impl Semaphore {
    /// Contract admin owns the per-depth verification-key registry.
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Register the ceremony verification key for a given tree depth (admin only).
    pub fn register_vkey(env: Env, depth: u32, vk: VkData) -> Result<(), Error> {
        Self::admin(&env)?.require_auth();
        env.storage().persistent().set(&DataKey::Vk(depth), &vk);
        Ok(())
    }

    /// Create a group with the given admin; returns the new group id.
    pub fn create_group(env: Env, admin: Address) -> u32 {
        admin.require_auth();
        let id: u32 = env.storage().instance().get(&DataKey::GroupCount).unwrap_or(0);
        let group = Group { admin: admin.clone(), root: BytesN::from_array(&env, &[0u8; 32]), depth: 0 };
        env.storage().persistent().set(&DataKey::Group(id), &group);
        env.storage().instance().set(&DataKey::GroupCount, &(id + 1));
        GroupCreated { group_id: id, admin }.publish(&env);
        id
    }

    /// Push a new off-chain-computed Merkle root + depth for a group (group admin only).
    pub fn update_group_root(
        env: Env,
        group_id: u32,
        new_root: BytesN<32>,
        depth: u32,
    ) -> Result<(), Error> {
        let mut group: Group = env
            .storage()
            .persistent()
            .get(&DataKey::Group(group_id))
            .ok_or(Error::GroupNotFound)?;
        group.admin.require_auth();
        if !is_canonical(&new_root) {
            return Err(Error::NonCanonicalInput);
        }
        // Archive the outgoing root so proofs built against it stay valid for a window.
        let old = group.root.clone();
        if old != BytesN::from_array(&env, &[0u8; 32]) {
            let expiry = env.ledger().sequence() + ROOT_VALIDITY_LEDGERS;
            env.storage().persistent().set(&DataKey::HistoricalRoot(group_id, old), &expiry);
        }
        group.root = new_root.clone();
        group.depth = depth;
        env.storage().persistent().set(&DataKey::Group(group_id), &group);
        RootUpdated { group_id, root: new_root.clone(), depth }.publish(&env);
        Ok(())
    }

    /// Transfer a group's admin (current group admin only).
    pub fn transfer_group_admin(env: Env, group_id: u32, new_admin: Address) -> Result<(), Error> {
        let mut group: Group = env
            .storage()
            .persistent()
            .get(&DataKey::Group(group_id))
            .ok_or(Error::GroupNotFound)?;
        group.admin.require_auth();
        group.admin = new_admin.clone();
        env.storage().persistent().set(&DataKey::Group(group_id), &group);
        AdminTransferred { group_id, admin: new_admin.clone() }.publish(&env);
        Ok(())
    }

    /// Whether `root` is currently accepted for `group_id` (current root, or a superseded root
    /// still inside its validity window).
    pub fn is_valid_root(env: Env, group_id: u32, root: BytesN<32>) -> Result<bool, Error> {
        let group: Group =
            env.storage().persistent().get(&DataKey::Group(group_id)).ok_or(Error::GroupNotFound)?;
        Ok(Self::root_accepted(&env, group_id, &group, &root))
    }

    /// Read a group's current state.
    pub fn group(env: Env, group_id: u32) -> Result<Group, Error> {
        env.storage().persistent().get(&DataKey::Group(group_id)).ok_or(Error::GroupNotFound)
    }

    fn root_accepted(env: &Env, group_id: u32, group: &Group, root: &BytesN<32>) -> bool {
        if root == &group.root {
            return true;
        }
        match env
            .storage()
            .persistent()
            .get::<DataKey, u32>(&DataKey::HistoricalRoot(group_id, root.clone()))
        {
            Some(expiry) => env.ledger().sequence() <= expiry,
            None => false,
        }
    }

    /// Stateless verification of a Semaphore proof against the depth's registered vkey.
    #[allow(clippy::too_many_arguments)]
    pub fn verify_proof(
        env: Env,
        depth: u32,
        merkle_root: BytesN<32>,
        nullifier: BytesN<32>,
        message: BytesN<32>,
        scope: BytesN<32>,
        a: BytesN<64>,
        b: BytesN<128>,
        c: BytesN<64>,
    ) -> Result<bool, Error> {
        let vk: VkData = env
            .storage()
            .persistent()
            .get(&DataKey::Vk(depth))
            .ok_or(Error::VkNotRegistered)?;
        Self::check_canonical(&merkle_root, &nullifier, &message, &scope)?;
        Ok(Self::run_verify(&env, &vk, &merkle_root, &nullifier, &message, &scope, &a, &b, &c))
    }

    /// Validate a proof for a group and consume its nullifier (state-changing).
    ///
    /// Enforces: known group root, canonical public inputs, a valid proof, and a
    /// not-yet-used nullifier (scoped to the group). The `scope` public input carries the
    /// epoch, so a fresh scope yields a fresh nullifier set at the application layer.
    #[allow(clippy::too_many_arguments)]
    pub fn validate_proof(
        env: Env,
        group_id: u32,
        merkle_root: BytesN<32>,
        nullifier: BytesN<32>,
        message: BytesN<32>,
        scope: BytesN<32>,
        a: BytesN<64>,
        b: BytesN<128>,
        c: BytesN<64>,
    ) -> Result<(), Error> {
        let group: Group = env
            .storage()
            .persistent()
            .get(&DataKey::Group(group_id))
            .ok_or(Error::GroupNotFound)?;
        if !Self::root_accepted(&env, group_id, &group, &merkle_root) {
            return Err(Error::InvalidRoot);
        }
        Self::check_canonical(&merkle_root, &nullifier, &message, &scope)?;

        let vk: VkData = env
            .storage()
            .persistent()
            .get(&DataKey::Vk(group.depth))
            .ok_or(Error::VkNotRegistered)?;
        if !Self::run_verify(&env, &vk, &merkle_root, &nullifier, &message, &scope, &a, &b, &c) {
            return Err(Error::InvalidProof);
        }

        let key = DataKey::Nullifier(group_id, nullifier.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::NullifierAlreadyUsed);
        }
        env.storage().persistent().set(&key, &());
        ProofValidated { group_id, nullifier: nullifier.clone(), message: message.clone(), scope: scope.clone() }.publish(&env);
        Ok(())
    }

    /// Trustless verification: source the group's Merkle root from an on-chain `SemaphoreGroups`
    /// contract (which computes it from the real members, so no admin can forge it) and verify a
    /// proof against it. Folds the trustless on-chain tree into the library by composition.
    #[allow(clippy::too_many_arguments)]
    pub fn verify_with_onchain_root(
        env: Env,
        groups_contract: Address,
        group_id: u32,
        depth: u32,
        nullifier: BytesN<32>,
        message: BytesN<32>,
        scope: BytesN<32>,
        a: BytesN<64>,
        b: BytesN<128>,
        c: BytesN<64>,
    ) -> Result<bool, Error> {
        // Cross-contract read of the authoritative, on-chain-computed root.
        let root_u256: U256 = env.invoke_contract(
            &groups_contract,
            &Symbol::new(&env, "root"),
            vec![&env, group_id.into_val(&env)],
        );
        let root: BytesN<32> = root_u256.to_be_bytes().try_into().unwrap();
        Self::check_canonical(&root, &nullifier, &message, &scope)?;
        let vk: VkData = env
            .storage()
            .persistent()
            .get(&DataKey::Vk(depth))
            .ok_or(Error::VkNotRegistered)?;
        Ok(Self::run_verify(&env, &vk, &root, &nullifier, &message, &scope, &a, &b, &c))
    }

    fn admin(env: &Env) -> Result<Address, Error> {
        env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotAdmin)
    }

    fn check_canonical(
        root: &BytesN<32>,
        nullifier: &BytesN<32>,
        message: &BytesN<32>,
        scope: &BytesN<32>,
    ) -> Result<(), Error> {
        if is_canonical(root) && is_canonical(nullifier) && is_canonical(message) && is_canonical(scope) {
            Ok(())
        } else {
            Err(Error::NonCanonicalInput)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run_verify(
        env: &Env,
        vk: &VkData,
        root: &BytesN<32>,
        nullifier: &BytesN<32>,
        message: &BytesN<32>,
        scope: &BytesN<32>,
        a: &BytesN<64>,
        b: &BytesN<128>,
        c: &BytesN<64>,
    ) -> bool {
        let pub_signals = soroban_sdk::vec![
            env,
            root.clone(),
            nullifier.clone(),
            message.clone(),
            scope.clone()
        ];
        verify_encoded(
            env, &pub_signals, a, b, c, &vk.alpha, &vk.beta, &vk.gamma, &vk.delta, &vk.ic,
        )
    }
}

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod fixture;
#[cfg(test)]
mod test;
