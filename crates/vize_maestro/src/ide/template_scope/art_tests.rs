//! Variant-local scopes must not resolve through a sibling or ordinary template.

use super::*;
use crate::server::ServerState;
use tower_lsp::lsp_types::Url;

#[test]
fn art_scopes_select_nearest_loop_and_slot_in_the_requested_variant() {
    for inline in [false, true] {
        for newline in ["\n", "\r\n"] {
            let ordinary = if inline {
                "<template><i v-for=\"item in rows\">{{ item }}</i></template>\n"
            } else {
                ""
            };
            let source = format!(
                "<script setup>const item = 'outer'; const slotValue = 'outer'; const rows = [];</script>\n{ordinary}<art title=\"Scope\" component=\"./Child.vue\">\n<variant name=\"First\" default>\n<div v-for=\"(item, firstOnly) in rows\">{{{{ item }}}}<Child v-slot=\"{{ slotValue }}\">{{{{ slotValue }}}}</Child></div>\n</variant>\n<variant name=\"Second\">\n<section v-for=\"(item, secondOnly) in rows\">🧭{{{{ item }}}}<span v-for=\"item in rows\">{{{{ item }}}}</span><Child v-slot=\"{{ slotValue }}\">{{{{ slotValue }}}}</Child></section>\n</variant>\n</art>\n"
            ).replace('\n', newline);
            let state = ServerState::new();
            let filename = if inline {
                "Inline.vue"
            } else {
                "Scope.art.vue"
            };
            let uri = Url::parse(&format!("file:///tmp/{filename}")).unwrap();
            state
                .documents
                .open(uri.clone(), source.clone(), 1, "vue".into());

            let second_loop = source.rfind("(item, secondOnly)").unwrap() + 1;
            let nested_loop = source.rfind("item in rows").unwrap();
            let second_slot = source.rfind("{ slotValue }").unwrap() + 2;
            for (offset, name, declaration, kind) in [
                (
                    source.find("🧭{{ item }}").unwrap() + "🧭{{ ".len(),
                    "item",
                    second_loop,
                    TemplateScopeBindingKind::Value,
                ),
                (
                    source.rfind("{{ item }}").unwrap() + 3,
                    "item",
                    nested_loop,
                    TemplateScopeBindingKind::Value,
                ),
                (
                    source.rfind("{{ slotValue }}").unwrap() + 3,
                    "slotValue",
                    second_slot,
                    TemplateScopeBindingKind::SlotProp,
                ),
            ] {
                let ctx = IdeContext::new(&state, &uri, offset).unwrap();
                let binding = v_for_binding_at(&ctx, name)
                    .or_else(|| v_slot_binding_at(&ctx, name))
                    .unwrap();
                assert_eq!(
                    binding,
                    TemplateScopeBinding {
                        name: name.into(),
                        start: declaration,
                        end: declaration + name.len(),
                        kind
                    }
                );
                assert_eq!(
                    definition(&ctx, name),
                    Some(GotoDefinitionResponse::Scalar(binding.location(&ctx)))
                );
                let (croquis, start) = analyze(&ctx).unwrap();
                let visible: Vec<_> = bindings_visible_at(&croquis, (offset - start) as u32)
                    .into_iter()
                    .map(|(name, _, _)| name)
                    .collect();
                assert!(visible.contains(&name), "{visible:?}");
                assert!(
                    !visible.contains(&"firstOnly"),
                    "sibling scope leaked: {visible:?}"
                );
            }

            let first_offset = source.find("{{ slotValue }}").unwrap() + 3;
            let ctx = IdeContext::new(&state, &uri, first_offset).unwrap();
            let (croquis, start) = analyze(&ctx).unwrap();
            let visible: Vec<_> = bindings_visible_at(&croquis, (first_offset - start) as u32)
                .into_iter()
                .map(|(name, _, _)| name)
                .collect();
            assert!(visible.contains(&"firstOnly"), "{visible:?}");
            assert!(
                !visible.contains(&"secondOnly"),
                "sibling scope leaked: {visible:?}"
            );
        }
    }
}
