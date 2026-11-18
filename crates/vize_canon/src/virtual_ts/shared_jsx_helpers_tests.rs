use super::{JSX_COMPONENT_HELPER, helpers};

#[test]
fn shared_preambles_publish_the_jsx_component_and_context_contracts() {
    const CONTEXT: &str = "type Ctx<Emits = {}, Slots = {}> = { emit: __EmitFn<Emits>; slots: Slots; attrs: Record<string, unknown>; };";
    for preamble in [
        helpers::SHARED_PREAMBLE_DTS,
        helpers::SHARED_PREAMBLE_STRICT_DTS,
    ] {
        assert_eq!(preamble.matches(JSX_COMPONENT_HELPER).count(), 1);
        assert_eq!(preamble.matches(CONTEXT).count(), 1);
    }
}
