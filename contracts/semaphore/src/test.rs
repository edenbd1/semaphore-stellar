use crate::fixture::*;
use crate::{Error, Semaphore, SemaphoreClient, VkData};
use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env};

struct Fix {
    env: Env,
    client_id: Address,
    admin: Address,
    group_id: u32,
}

fn vk(env: &Env) -> VkData {
    VkData {
        alpha: BytesN::from_array(env, &VK_ALPHA),
        beta: BytesN::from_array(env, &VK_BETA),
        gamma: BytesN::from_array(env, &VK_GAMMA),
        delta: BytesN::from_array(env, &VK_DELTA),
        ic: vec![
            env,
            BytesN::from_array(env, &VK_IC_0),
            BytesN::from_array(env, &VK_IC_1),
            BytesN::from_array(env, &VK_IC_2),
            BytesN::from_array(env, &VK_IC_3),
            BytesN::from_array(env, &VK_IC_4),
        ],
    }
}

fn setup() -> (Fix, SemaphoreClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register(Semaphore, (admin.clone(),));
    let client = SemaphoreClient::new(&env, &id);
    client.register_vkey(&DEPTH, &vk(&env));
    let group_id = client.create_group(&admin);
    client.update_group_root(&group_id, &BytesN::from_array(&env, &ROOT), &DEPTH);
    (Fix { env, client_id: id, admin, group_id }, client)
}

fn proof_parts(env: &Env) -> (BytesN<32>, BytesN<32>, BytesN<32>, BytesN<32>, BytesN<64>, BytesN<128>, BytesN<64>) {
    (
        BytesN::from_array(env, &ROOT),
        BytesN::from_array(env, &NULLIFIER),
        BytesN::from_array(env, &MESSAGE),
        BytesN::from_array(env, &SCOPE),
        BytesN::from_array(env, &PROOF_A),
        BytesN::from_array(env, &PROOF_B),
        BytesN::from_array(env, &PROOF_C),
    )
}

/// A real proof against the group's current root validates and consumes its nullifier.
#[test]
fn validates_real_proof_and_consumes_nullifier() {
    let (fx, client) = setup();
    let (root, nul, msg, scope, a, b, c) = proof_parts(&fx.env);
    client.validate_proof(&fx.group_id, &root, &nul, &msg, &scope, &a, &b, &c);

    // Second use of the same nullifier is rejected.
    let r = client.try_validate_proof(&fx.group_id, &root, &nul, &msg, &scope, &a, &b, &c);
    assert_eq!(r, Err(Ok(Error::NullifierAlreadyUsed)));
}

/// A proof carrying a root the group does not have is rejected before verification.
#[test]
fn rejects_unknown_root() {
    let (fx, client) = setup();
    let (_root, nul, msg, scope, a, b, c) = proof_parts(&fx.env);
    let mut bad = ROOT;
    bad[31] ^= 0x01; // still canonical, just not the group's root
    let r = client.try_validate_proof(
        &fx.group_id,
        &BytesN::from_array(&fx.env, &bad),
        &nul,
        &msg,
        &scope,
        &a,
        &b,
        &c,
    );
    assert_eq!(r, Err(Ok(Error::InvalidRoot)));
}

/// Stateless verify_proof: true for the real proof, false when a public input is tampered.
#[test]
fn verify_proof_true_and_false() {
    let (fx, client) = setup();
    let (root, nul, msg, scope, a, b, c) = proof_parts(&fx.env);
    assert!(client.verify_proof(&DEPTH, &root, &nul, &msg, &scope, &a, &b, &c));

    let mut bad_msg = MESSAGE;
    bad_msg[31] ^= 0x01;
    assert!(!client.verify_proof(
        &DEPTH,
        &root,
        &nul,
        &BytesN::from_array(&fx.env, &bad_msg),
        &scope,
        &a,
        &b,
        &c
    ));
}

/// Non-canonical public inputs (>= r) are rejected rather than silently reduced mod r.
#[test]
fn rejects_non_canonical_input() {
    let (fx, client) = setup();
    let (root, _nul, msg, scope, a, b, c) = proof_parts(&fx.env);
    let non_canonical = BytesN::from_array(&fx.env, &[0xffu8; 32]); // > r
    let r = client.try_verify_proof(&DEPTH, &root, &non_canonical, &msg, &scope, &a, &b, &c);
    assert_eq!(r, Err(Ok(Error::NonCanonicalInput)));
}

/// Nullifiers are scoped per group: the same nullifier can be used once in each group.
#[test]
fn nullifier_is_scoped_per_group() {
    let (fx, client) = setup();
    let (root, nul, msg, scope, a, b, c) = proof_parts(&fx.env);
    client.validate_proof(&fx.group_id, &root, &nul, &msg, &scope, &a, &b, &c);

    // A second group with the same root accepts the same nullifier once.
    let g2 = client.create_group(&fx.admin);
    client.update_group_root(&g2, &root, &DEPTH);
    client.validate_proof(&g2, &root, &nul, &msg, &scope, &a, &b, &c);
    let again = client.try_validate_proof(&g2, &root, &nul, &msg, &scope, &a, &b, &c);
    assert_eq!(again, Err(Ok(Error::NullifierAlreadyUsed)));
}
