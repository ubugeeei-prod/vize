use tower_lsp::lsp_types::Url;

use super::template::incomplete_slot_name_completions;
use crate::ide::IdeContext;
use crate::server::ServerState;

#[test]
fn computed_slot_arguments_do_not_claim_the_static_slot_completion_route() {
    for source in [
        "<template><Comp><template #[names.]></template></Comp></template>",
        "<template><Comp><template v-slot:[names.]></template></Comp></template>",
    ] {
        let uri = Url::parse("file:///DynamicSlot.vue").unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        let cursor = source.find("names.").unwrap() + "names.".len();
        let ctx = IdeContext::at_completion(&state, &uri, cursor).unwrap();

        assert_eq!(
            incomplete_slot_name_completions(&ctx)
                .as_ref()
                .map(Vec::len),
            None,
            "{source}"
        );
    }
}

#[test]
fn static_slot_prefix_still_claims_the_closed_slot_completion_route() {
    let source = "<template><Comp><template #></template></Comp></template>";
    let uri = Url::parse("file:///StaticSlot.vue").unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let cursor = source.find("#>").unwrap() + 1;
    let ctx = IdeContext::at_completion(&state, &uri, cursor).unwrap();

    assert_eq!(
        incomplete_slot_name_completions(&ctx)
            .as_ref()
            .map(Vec::len),
        Some(0)
    );
}
