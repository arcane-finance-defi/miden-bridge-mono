use miden_assembly::diagnostics::IntoDiagnostic;
use miden_lib::AuthScheme;
use miden_lib::AuthScheme::RpoFalcon512;
use miden_objects::account::{AccountIdAnchor, AccountStorageMode, AuthSecretKey};
use miden_objects::asset::TokenSymbol;
use miden_objects::{Felt, Word};
use miden_tx::testing::MockChain;
use miden_bridge::accounts::token_wrapper::create_token_wrapper_account;


pub fn get_new_pk_and_authenticator() -> (Word, AuthSecretKey) {
    let seed = [0_u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);
    let sec_key = SecretKey::with_rng(&mut rng);
    (
        sec_key.public_key().into(),
        AuthSecretKey::RpoFalcon512(sec_key),
    )
}

#[test]
fn should_bridge_out() {
    let mut mock_chain = MockChain::new();

    let anchorBlockId = mock_chain.latest_block_header().epoch_block_num();
    let anchorBlockCommitment = mock_chain.block_header(anchorBlockId.as_usize()).commitment();




    create_token_wrapper_account(
        [1; 32],
        AccountIdAnchor::new(
            anchorBlockId,
            anchorBlockCommitment
        ).into_diagnostic()?,
        TokenSymbol::new("TEST").into_diagnostic()?
        6,
        Felt::new(u64::MAX)
        AccountStorageMode::Public,
        AuthScheme::RpoFalcon512 {}
    );
}