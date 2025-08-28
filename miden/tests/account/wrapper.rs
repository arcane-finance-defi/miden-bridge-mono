use miden_bridge::{
    accounts::{testing::create_token_wrapper_account_builder, token_wrapper::bridge_note_tag},
    notes::bridge::{bridge, croschain},
};
use miden_lib::{
    account::{auth::RpoFalcon512ProcedureAcl, faucets::BasicFungibleFaucet},
    transaction::TransactionKernel,
};
use miden_objects::{
    account::{AccountId, AccountStorageMode, AuthSecretKey},
    asset::{FungibleAsset, TokenSymbol},
    crypto::{
        dsa::rpo_falcon512::{PublicKey, SecretKey},
        rand::{FeltRng, RpoRandomCoin},
    },
    note::{
        Note, NoteAssets, NoteExecutionHint, NoteId, NoteInputs, NoteMetadata, NoteRecipient,
        NoteTag, NoteType,
    },
    testing::account_id::ACCOUNT_ID_SENDER,
    transaction::{OutputNote, TransactionScript},
    utils::word_to_masm_push_string,
    Felt, FieldElement, Word,
};
use miden_testing::{AccountState, Auth, MockChain};
use miden_bridge::accounts::{token_wrapper::bridge_note_tag, testing::create_token_wrapper_account_builder};
use miden_bridge::notes::bridge::{bridge, croschain};
use miden_lib::account::faucets::BasicFungibleFaucet;
use miden_bridge::errors::note_errors::ERR_CROSSCHAIN_TOO_EARLY_EXECUTION;
use crate::assert_transaction_executor_error;

pub fn get_new_pk_and_authenticator(seed: [Felt; 4]) -> (PublicKey, AuthSecretKey) {
    let seed = Word::from(seed);
    let mut rng = RpoRandomCoin::new(seed);
    let sec_key = SecretKey::with_rng(&mut rng);
    (sec_key.public_key(), AuthSecretKey::RpoFalcon512(sec_key))
}

const DAY: u32 = 60 * 60 * 24;

#[test]
fn should_issue_public_bridge_note() -> anyhow::Result<()> {
    let mut mock_chain = MockChain::new();

    let (pub_key, _secret_key) =
        get_new_pk_and_authenticator([Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)]);

    let wrapper_builder = create_token_wrapper_account_builder(
        [1; 32],
        TokenSymbol::new("TEST")?,
        6,
        Felt::new(1000000),
        1,
        [Felt::new(1), Felt::new(1), Felt::new(1)],
        AccountStorageMode::Public,
    )?;

    wrapper_builder
        .clone()
        .with_auth_component(RpoFalcon512ProcedureAcl::new(
            pub_key,
            vec![BasicFungibleFaucet::distribute_digest()],
        )?)
        .build()?;

    let mut wrapper = mock_chain.add_pending_account_from_builder(
        Auth::ProcedureAcl {
            auth_trigger_procedures: vec![BasicFungibleFaucet::distribute_digest()],
        },
        wrapper_builder,
        AccountState::Exists,
    )?;

    let fungible_asset = FungibleAsset::new(wrapper.id(), 1000).unwrap().into();

    let mut rng = RpoRandomCoin::new(Word::from([
        Felt::new(456),
        Felt::new(456),
        Felt::new(456),
        Felt::new(456),
    ]));

    let output_serial_num = rng.draw_word();

    let receiver_address = [rng.draw_element(), rng.draw_element(), rng.draw_element()];

    let call_address = [Felt::ZERO, Felt::ZERO, Felt::ZERO];

    let chain_id: u64 = 123;
    let unlock_timestamp = mock_chain.latest_block_header().timestamp() + 7 * DAY;

    let note_inputs = NoteInputs::new(vec![
        output_serial_num[0],
        output_serial_num[1],
        output_serial_num[2],
        output_serial_num[3],
        Felt::new(chain_id),
        receiver_address[0],
        receiver_address[1],
        receiver_address[2],
        Felt::new(unlock_timestamp as u64),
        Felt::ZERO,
        call_address[0],
        call_address[1],
        call_address[2],
    ])?;

    let note = Note::new(
        NoteAssets::new(vec![fungible_asset])?,
        NoteMetadata::new(
            AccountId::try_from(ACCOUNT_ID_SENDER)?,
            NoteType::Public,
            NoteTag::from_account_id(wrapper.id()),
            NoteExecutionHint::Always,
            Felt::ZERO,
        )?,
        NoteRecipient::new(
            [Felt::new(1), Felt::ZERO, Felt::ZERO, Felt::ZERO],
            croschain(),
            note_inputs,
        ),
    );

    mock_chain.add_pending_note(OutputNote::Full(note.clone()));
    mock_chain.prove_next_block().expect("Unable to prove next block");

    let mint_tx_inputs = mock_chain.get_transaction_inputs(wrapper.clone(), None, &[], &[])?;

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

                # truncate the stack
                dropw dropw dropw dropw
            end
            ",
        note_type = Felt::from(NoteType::Private),
        recipient = word_to_masm_push_string(&rng.draw_word()),
        aux = 0,
        tag = Felt::from(NoteTag::for_local_use_case(0, 0)?),
        note_execution_hint = Felt::from(NoteExecutionHint::Always),
        amount = 10000
    );

    let mint_tx_script =
        TransactionScript::compile(mint_tx_script_code, TransactionKernel::testing_assembler())?;

    let executed_mint_transaction = mock_chain.build_tx_context(
            wrapper.clone(),
            &[],
            &[]
        )?
            .tx_script(mint_tx_script)
            .tx_inputs(mint_tx_inputs)
            .build()?
            .execute().expect("Unable to execute mint tx");

    mock_chain.add_pending_executed_transaction(&executed_mint_transaction.clone())?;
    mock_chain.prove_next_block()?;

    wrapper.apply_delta(&executed_mint_transaction.account_delta().clone())?;

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
        ])?,
    );

    let expected_note_id =
        NoteId::new(expected_recipient.digest(), NoteAssets::new(vec![])?.commitment());

    let expected_note = Note::new(
        NoteAssets::new(vec![])?,
        NoteMetadata::new(
            wrapper.id(),
            NoteType::Public,
            bridge_note_tag(),
            NoteExecutionHint::Always,
            Felt::ZERO,
        )?,
        expected_recipient.clone(),
    );

    let tx_inputs =
        mock_chain.get_transaction_inputs(wrapper.clone(), None, &[note.clone().id()], &[])?;


    let failed_executed_transaction = mock_chain.build_tx_context(
            wrapper.clone(),
            &[],
            &[]
        )?
            .tx_inputs(tx_inputs)
            .extend_expected_output_notes(vec![OutputNote::Full(expected_note.clone())])
            .build()?
            .execute();

    assert_transaction_executor_error!(
        failed_executed_transaction,
        ERR_CROSSCHAIN_TOO_EARLY_EXECUTION
    );

    mock_chain.prove_next_block_at(unlock_timestamp + DAY)
        .expect("Unable to generate next block");


    let tx_inputs = mock_chain.
        get_transaction_inputs(
            wrapper.clone(),
            None,
            &[note.clone().id()],
            &[]
        )?;

    let executed_transaction = mock_chain.build_tx_context(
        wrapper.clone(),
        &[],
        &[]
    )?
        .tx_inputs(tx_inputs.clone())
        .extend_expected_output_notes(vec![OutputNote::Full(expected_note.clone())])
        .build()?
        .execute().expect("Unable to execute crosschain consume transaction");

    assert_eq!(executed_transaction.output_notes().num_notes(), 1);
    assert_eq!(
        executed_transaction.output_notes().get_note(0).metadata().tag(),
        bridge_note_tag()
    );
    assert_eq!(executed_transaction.output_notes().get_note(0).id(), expected_note_id);

    Ok(())
}
