use super::{JSX_COMPONENT_HELPER, helpers};

#[test]
fn shared_preambles_publish_the_jsx_component_and_context_contracts() {
    for preamble in [
        helpers::SHARED_PREAMBLE_DTS,
        helpers::SHARED_PREAMBLE_STRICT_DTS,
    ] {
        assert!(preamble.contains(JSX_COMPONENT_HELPER));
        assert!(preamble.contains(
            "type Ctx<Emits = {}, Slots = {}> = { emit: __EmitFn<Emits>; slots: Slots; attrs: Record<string, unknown>; };"
        ));
    }
}
