use alloc::sync::Arc;
use miden_objects::{transaction::ExecutedTransaction};
use miden_objects::assembly::DefaultSourceManager;
use miden_tx::{TransactionExecutor, TransactionExecutorError};
use miden_testing::TransactionContext;
use miden_tx::auth::TransactionAuthenticator;
use winter_maybe_async::*;

#[maybe_async]
pub fn execute_with_debugger(
    ctx: TransactionContext,
    authenticator: Option<Arc<dyn TransactionAuthenticator>>
) -> Result<ExecutedTransaction, TransactionExecutorError> {
    let account_id = ctx.account().id();
    let block_num = ctx.tx_inputs().block_header().block_num();
    let notes =
        ctx.tx_inputs().input_notes().clone();

    let tx_args = ctx.tx_args().clone();

    let tx_executor = TransactionExecutor::new(
        Arc::new(ctx),
        authenticator
    ).with_debug_mode();

    maybe_await!(tx_executor.execute_transaction(
        account_id,
        block_num,
        notes.clone(),
        tx_args,
        Arc::new(DefaultSourceManager::default())
    ))
}