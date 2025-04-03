extern crate alloc;


mod account;
mod common;
// HELPER FUNCTIONS
// ================================================================================================

#[macro_export]
macro_rules! assert_transaction_executor_error {
    ($execution_result:expr, $expected_err_code:expr) => {
        match $execution_result {
            Err(miden_tx::TransactionExecutorError::TransactionProgramExecutionFailed(
                miden_prover::ExecutionError::FailedAssertion { clk: _, err_code, err_msg: _ }
            )) => {
                assert!(
                    err_code == $expected_err_code,
                    "Execution failed on assertion with an unexpected error code (Actual err_code: {}, expected {}).",
                    err_code, $expected_err_code
                );
            },
            Ok(_) => panic!("Execution was unexpectedly successful"),
            Err(other) => panic!("Execution error was not as expected: {}", other),
        }
    };
}

