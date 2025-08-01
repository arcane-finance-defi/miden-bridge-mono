use miden_objects::{
    assembly::Library,
    utils::{sync::LazyLock, Deserializable},
};

static TOKEN_WRAPPER_ACCOUNT_CODE: LazyLock<Library> = LazyLock::new(|| {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/assets/contracts/fungible_wrapper.masl"));
    Library::read_from_bytes(bytes).expect("Shipped Token wrapper library is well-formed")
});

pub fn token_wrapper_account_library() -> Library {
    TOKEN_WRAPPER_ACCOUNT_CODE.clone()
}
