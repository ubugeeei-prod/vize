use super::{JSX_COMPONENT_HELPER, helpers};

#[test]
fn batch_preambles_leave_the_current_jsx_contracts_per_file() {
    // Until the L4 route atomically stops emitting its local declarations,
    // publishing the same aliases here would make legacy scripts report TS2300.
    const CONTEXT: &str = "type Ctx<Emits = {}, Slots = {}> = { emit: __EmitFn<Emits>; slots: Slots; attrs: Record<string, unknown>; };";
    for preamble in [
        helpers::SHARED_PREAMBLE_DTS,
        helpers::SHARED_PREAMBLE_STRICT_DTS,
    ] {
        assert_eq!(preamble.matches(JSX_COMPONENT_HELPER).count(), 0);
        assert_eq!(preamble.matches(CONTEXT).count(), 0);
    }
}
