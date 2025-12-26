use miden_objects::{
    note::NoteScript,
    utils::{sync::LazyLock, Deserializable},
};

static BRIDGE_SCRIPT: LazyLock<NoteScript> = LazyLock::new(|| {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/assets/events/BRIDGE.masb"));
    NoteScript::read_from_bytes(bytes).expect("Shipped BRIDGE script is malformed")
});

pub fn bridge() -> NoteScript {
    BRIDGE_SCRIPT.clone()
}

static CROSSCHAIN_SCRIPT: LazyLock<NoteScript> = LazyLock::new(|| {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/assets/note_scripts/CROSSCHAIN.masb"));
    NoteScript::read_from_bytes(bytes).expect("Shipped CROSSCHAIN script is malformed")
});

pub fn croschain() -> NoteScript {
    CROSSCHAIN_SCRIPT.clone()
}
