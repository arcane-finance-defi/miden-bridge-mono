use alloc::{sync::Arc, vec::Vec};
use miden_objects::{note::NoteId, transaction::ExecutedTransaction};
use miden_tx::{testing::TransactionContext, TransactionExecutor, TransactionExecutorError};
use miden_tx::auth::{BasicAuthenticator, TransactionAuthenticator};
use rand_chacha::ChaCha20Rng;
use winter_maybe_async::*;

#[maybe_async]
pub fn execute_with_debugger(
    ctx: TransactionContext,
    authenticator: Option<Arc<dyn TransactionAuthenticator>>
) -> Result<ExecutedTransaction, TransactionExecutorError> {
    let account_id = ctx.account().id();
    let block_num = ctx.tx_inputs().block_header().block_num();
    let notes: Vec<NoteId> =
        ctx.tx_inputs().input_notes().into_iter().map(|n| n.id()).collect();

    let tx_executor = TransactionExecutor::new(
        Arc::new(ctx.tx_inputs().clone()),
        authenticator
    ).with_debug_mode();

    maybe_await!(tx_executor.execute_transaction(account_id, block_num, &notes, ctx.tx_args().clone()))
}