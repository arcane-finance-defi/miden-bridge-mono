use miden_assembly::Library;
use miden_objects::utils::Deserializable;
use miden_objects::utils::sync::LazyLock;

static FULL_LIBRARY: LazyLock<Library> = LazyLock::new(|| {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/assets/contracts/full.masl"));
    let lib = Library::read_from_bytes(bytes).expect("Shipped full.masl library isn't well formed");
    lib
});

pub fn full_library() -> Library {
    FULL_LIBRARY.clone()
}