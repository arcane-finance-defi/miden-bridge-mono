use std::sync::Arc;
use miden_assembly::diagnostics::IntoDiagnostic;
use miden_assembly::Report;
use miden_lib::AuthScheme;
use miden_lib::transaction::TransactionKernel;
use miden_objects::account::{AccountId, AccountIdAnchor, AccountStorageMode, AuthSecretKey};
use miden_objects::asset::{FungibleAsset, TokenSymbol};
use miden_objects::{Felt, FieldElement, Word};
use miden_objects::crypto::dsa::rpo_falcon512::{PublicKey, SecretKey};
use miden_objects::crypto::rand::{FeltRng, RpoRandomCoin};
use miden_objects::note::{Note, NoteAssets, NoteExecutionHint, NoteExecutionMode, NoteId, NoteInputs, NoteMetadata, NoteRecipient, NoteTag, NoteType};
use miden_objects::testing::account_id::ACCOUNT_ID_SENDER;
use miden_objects::transaction::{OutputNote, TransactionScript};
use miden_objects::utils::word_to_masm_push_string;
use miden_tx::auth::BasicAuthenticator;
use miden_tx::testing::{MockChain, TransactionContextBuilder};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use miden_bridge::accounts::token_wrapper::{bridge_note_tag, create_token_wrapper_account};
use miden_bridge::notes::bridge::{bridge, croschain};
use crate::common::executor::execute_with_debugger;

pub fn get_new_pk_and_authenticator(seed: [Felt; 4]) -> (PublicKey, AuthSecretKey) {
    let seed = Word::from(seed);
    let mut rng = RpoRandomCoin::new(seed);
    let sec_key = SecretKey::with_rng(&mut rng);
    (
        sec_key.public_key(),
        AuthSecretKey::RpoFalcon512(sec_key),
    )
}

#[test]
fn should_issue_public_bridge_note() -> Result<(), Report> {
    let mut mock_chain = MockChain::new();

    let anchor_block_id = mock_chain.latest_block_header().epoch_block_num();
    let anchor_block_commitment = mock_chain.block_header(anchor_block_id.as_usize()).commitment();

    let (pub_key, secret_key) = get_new_pk_and_authenticator([
        Felt::new(1),
        Felt::new(2),
        Felt::new(3),
        Felt::new(4)
    ]);

    let wrapper_authenticator = &BasicAuthenticator::new_with_rng(
        &[(
            Word::from(pub_key),
            secret_key
        )],
        ChaCha20Rng::from_os_rng()
    );

    let (mut wrapper, wrapper_seed) = create_token_wrapper_account(
        [1; 32],
        AccountIdAnchor::new(
            anchor_block_id,
            anchor_block_commitment
        ).into_diagnostic()?,
        TokenSymbol::new("TEST").into_diagnostic()?,
        6,
        Felt::new(1000000),
        1,
        [Felt::new(1),Felt::new(1),Felt::new(1)],
        AccountStorageMode::Private,
        AuthScheme::RpoFalcon512 { pub_key: pub_key.clone() },
    ).into_diagnostic()?;


    let fungible_asset =
        FungibleAsset::new(wrapper.id(), 1000)
            .unwrap()
            .into();

    let mut rng = RpoRandomCoin::new(Word::from([
        Felt::new(456),
        Felt::new(456),
        Felt::new(456),
        Felt::new(456)
    ]));

    let output_serial_num = rng.draw_word();

    let receiver_address = [
        rng.draw_element(),
        rng.draw_element(),
        rng.draw_element(),
    ];

    let call_address = [
        Felt::ZERO,
        Felt::ZERO,
        Felt::ZERO,
    ];

    let chain_id: u64 = 123;

    let note_inputs = NoteInputs::new(vec![
        output_serial_num[0],
        output_serial_num[1],
        output_serial_num[2],
        output_serial_num[3],
        Felt::new(chain_id),
        receiver_address[0],
        receiver_address[1],
        receiver_address[2],
        Felt::ZERO,
        call_address[0],
        call_address[1],
        call_address[2],
    ]).into_diagnostic()?;

    let note = Note::new(
        NoteAssets::new(vec![fungible_asset]).into_diagnostic()?,
        NoteMetadata::new(
            AccountId::try_from(ACCOUNT_ID_SENDER).into_diagnostic()?,
            NoteType::Public,
            NoteTag::from_account_id(wrapper.id(), NoteExecutionMode::Local).into_diagnostic()?,
            NoteExecutionHint::Always,
            Felt::ZERO
        ).into_diagnostic()?,
        NoteRecipient::new(
            [Felt::new(1), Felt::ZERO, Felt::ZERO, Felt::ZERO],
            croschain(),
            note_inputs
        )
    );

    mock_chain.add_pending_note(note.clone());
    mock_chain.add_pending_account(wrapper.clone());

    mock_chain.seal_next_block();

    let mint_tx_inputs = mock_chain.get_transaction_inputs(
        wrapper.clone(),
        Some(wrapper_seed),
        &[],
        &[]
    );

    let mint_tx_script_code = format!(
        "
            begin
                # pad the stack before call
                push.0.0.0 padw

                push.{recipient}
                push.{note_execution_hint}
                push.{note_type}
                push.{aux}
                push.{tag}
                push.{amount}
                # => [amount, tag, aux, note_type, execution_hint, RECIPIENT, pad(7)]

                call.::miden::contracts::faucets::basic_fungible::distribute
                # => [note_idx, pad(15)]

                call.::miden::contracts::auth::basic::auth_tx_rpo_falcon512
                # => [note_idx, pad(15)]

                # truncate the stack
                dropw dropw dropw dropw
            end
            ",
        note_type = Felt::from(NoteType::Private),
        recipient = word_to_masm_push_string(&rng.draw_word()),
        aux = 0,
        tag = Felt::from(NoteTag::for_local_use_case(0,0).into_diagnostic()?),
        note_execution_hint = Felt::from(NoteExecutionHint::Always),
        amount = 10000
    );

    let mint_tx_script =
        TransactionScript::compile(mint_tx_script_code, vec![], TransactionKernel::testing_assembler())
            .into_diagnostic()?;

    let executed_mint_transaction = execute_with_debugger(
        TransactionContextBuilder::new(wrapper.clone())
            .tx_script(mint_tx_script)
            .tx_inputs(mint_tx_inputs)
            .account_seed(Some(wrapper_seed))
            .authenticator(Some(wrapper_authenticator.clone()))
            .build(),
        Some(Arc::new(wrapper_authenticator.clone()))
    ).into_diagnostic()?;

    mock_chain.apply_executed_transaction(&executed_mint_transaction.clone());
    mock_chain.seal_next_block();

    wrapper.apply_delta(&executed_mint_transaction.account_delta().clone()).into_diagnostic()?;

    let asset_word = Word::from(fungible_asset);

    let expected_recipient = NoteRecipient::new(
        output_serial_num,
        bridge(),
        NoteInputs::new(vec![
            asset_word[0],
            asset_word[1],
            asset_word[2],
            asset_word[3],
            Felt::new(chain_id),
            receiver_address[0],
            receiver_address[1],
            receiver_address[2],
            Felt::ZERO,
            call_address[0],
            call_address[1],
            call_address[2],
        ]).into_diagnostic()?
    );

    let expected_note_id = NoteId::new(
        expected_recipient.digest(),
        NoteAssets::new(vec![]).into_diagnostic()?.commitment()
    );

    let expected_note = Note::new(
        NoteAssets::new(vec![]).into_diagnostic()?,
        NoteMetadata::new(
            wrapper.id(),
            NoteType::Public,
            bridge_note_tag(),
            NoteExecutionHint::Always,
            Felt::ZERO
        ).into_diagnostic()?,
        expected_recipient.clone()
    );

    let tx_inputs = mock_chain.
        get_transaction_inputs(
            wrapper.clone(),
            None,
            &[note.clone().id()],
            &[]
        );


    let executed_transaction = execute_with_debugger(
        TransactionContextBuilder::new(wrapper).account_seed(Some(wrapper_seed))
            .tx_inputs(tx_inputs)
            .expected_notes(vec![OutputNote::Full(expected_note)]).build(),
        None
    ).into_diagnostic()?;

    assert_eq!(executed_transaction.output_notes().num_notes(), 1);
    assert_eq!(executed_transaction.output_notes().get_note(0).metadata().tag(), bridge_note_tag());
    assert_eq!(executed_transaction.output_notes().get_note(0).id(), expected_note_id);

    Ok(())
}