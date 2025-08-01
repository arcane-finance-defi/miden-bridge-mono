use miden_objects::{
    note::NoteScript,
    utils::{sync::LazyLock, Deserializable},
    vm::Program,
};

static BRIDGE_SCRIPT: LazyLock<NoteScript> = LazyLock::new(|| {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/assets/events/BRIDGE.masb"));
    let program = Program::read_from_bytes(bytes).expect("Shipped BRIDGE script is well-formed");
    NoteScript::new(program)
});

pub fn bridge() -> NoteScript {
    BRIDGE_SCRIPT.clone()
}

static CROSSCHAIN_SCRIPT: LazyLock<NoteScript> = LazyLock::new(|| {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/assets/note_scripts/CROSSCHAIN.masb"));
    let program =
        Program::read_from_bytes(bytes).expect("Shipped CROSSCHAIN script is well-formed");
    NoteScript::new(program)
});

pub fn croschain() -> NoteScript {
    CROSSCHAIN_SCRIPT.clone()
}
