use crate::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};
use vize_carton::cstr;

const SCRIPT: &str = "<script setup lang=\"ts\">\nimport Comp from './Comp.vue'\nconst state: { value?: number } = {}\nstate.value = 1\nconst noop = () => {}\nconst flag = true\n</script>\n";

#[test]
fn actual_root_checks_keep_creation_between_earlier_and_later_render_mutations() {
    for newline in ["\n", "\r\n"] {
        for prefix in ["", "<!-- 日本 🧭 -->\n"] {
            for callback in [
                "(field: typeof state.value) => !!field",
                "(field: typeof state['value']) => !!field",
                "(field: (typeof state)['value']) => !!field",
                "(field): typeof state.value => field",
                "(field) => field as typeof state.value",
                "(field) => field satisfies typeof state.value",
            ] {
                for directive in ["", "<!-- @vue-expect-error -->\n"] {
                    let source = cstr!("{prefix}{SCRIPT}<template>\n<div>{{{{ state.value = 2 }}}}</div>\n{directive}<Comp :pick=\"{callback}\" @save=\"noop\" />\n<div>{{{{ delete state.value }}}}</div>\n</template>\n").replace('\n', newline);
                    let result = type_check_sfc(
                        &source,
                        &SfcTypeCheckOptions::new("src/Min.vue").with_virtual_ts(),
                    );
                    let generated = result.virtual_ts.as_deref().expect("a complete projection");
                    let owner = generated
                        .find("const __vize_prop_check_0_pick:")
                        .expect("one canonical callback owner");
                    assert_eq!(
                        generated.matches("const __vize_prop_check_0_pick:").count(),
                        1
                    );
                    assert!(
                        generated
                            .find("void (state.value = 2)")
                            .expect("earlier render effect")
                            < owner
                    );
                    assert!(
                        owner
                            < generated
                                .find("void (delete state.value)")
                                .expect("later render effect")
                    );
                    let mapping = result
                        .virtual_ts_mappings
                        .iter()
                        .find(|mapping| {
                            generated
                                .get(mapping.gen_range.clone())
                                .is_some_and(|text| {
                                    text.trim_start()
                                        .starts_with("const __vize_prop_check_0_pick:")
                                })
                        })
                        .expect("the moved owner retains its complete authored mapping");
                    let attribute = cstr!(":pick=\"{callback}\"");
                    assert_eq!(
                        source.get(mapping.src_range.clone()),
                        Some(attribute.as_str())
                    );
                    let value = mapping
                        .sub_spans
                        .iter()
                        .find(|span| source.get(span.src_range.clone()) == Some(callback))
                        .expect("exact callback value geometry");
                    assert_eq!(generated.get(value.gen_range.clone()), Some(callback));
                    for mapping in &result.virtual_ts_mappings {
                        assert!(generated.get(mapping.gen_range.clone()).is_some());
                        assert!(source.get(mapping.src_range.clone()).is_some());
                    }
                }
            }
        }
    }
}

#[test]
fn earlier_vif_chains_keep_their_scope_and_unsupported_owners_keep_the_old_phase() {
    for newline in ["\n", "\r\n"] {
        let supported = cstr!("{SCRIPT}<template><div v-if=\"flag\">{{{{ state.value = 2 }}}}</div><div v-else>{{{{ state.value = 3 }}}}</div><Comp :pick=\"(field: typeof state.value) => !!field\" @save=\"noop\" /><div>{{{{ delete state.value }}}}</div></template>").replace('\n', newline);
        let result = type_check_sfc(
            &supported,
            &SfcTypeCheckOptions::new("src/Min.vue").with_virtual_ts(),
        );
        let generated = result.virtual_ts.as_deref().expect("complete projection");
        let owner = generated
            .find("const __vize_prop_check_0_pick:")
            .expect("root owner");
        assert!(generated.find("state.value = 3").expect("else branch") < owner);
        assert!(
            owner
                < generated
                    .find("delete state.value")
                    .expect("later mutation")
        );

        for component in [
            "<Comp v-if=\"flag\" :pick=\"(field: typeof state.value) => !!field\" @save=\"noop\" />",
            "<Comp v-for=\"item in [state]\" :pick=\"(field: typeof item.value) => !!field\" @save=\"noop\" />",
            "<Comp :pick=\"(field: typeof state.value) => !!field\" @save=\"noop\"><template #default>slot</template></Comp>",
            "<div><Comp :pick=\"(field: typeof state.value) => !!field\" @save=\"noop\" /></div>",
            "<Unknown :pick=\"(field: typeof state.value) => !!field\" @save=\"noop\" />",
        ] {
            let source = cstr!("<!-- 日本 🧭 -->{SCRIPT}<template>{component}<div>{{{{ delete state.value }}}}</div></template>").replace('\n', newline);
            let result = type_check_sfc(
                &source,
                &SfcTypeCheckOptions::new("src/Min.vue").with_virtual_ts(),
            );
            let generated = result
                .virtual_ts
                .as_deref()
                .expect("complete recovery projection");
            let owner = generated
                .find("const __vize_prop_check_0_pick:")
                .expect("unsupported checks remain visible");
            assert!(
                owner
                    > generated
                        .find("// Component props type declarations")
                        .expect("unchanged late check phase"),
                "{component}"
            );
        }
    }
}

#[test]
fn effectful_opening_tags_and_prior_deferred_scopes_keep_their_complete_old_phase() {
    let callback = "(field: typeof state.value) => !!field";
    for newline in ["\n", "\r\n"] {
        for value in [
            "state.value = 2",
            "state.value++",
            "read()",
            "assertState(state)",
            "state.value",
            "({ get value() { delete state.value; return 1 } })",
            "({ [state.value]: true })",
        ] {
            let source = cstr!("<!-- 日本 🧭 -->{SCRIPT}<template><Comp :other=\"{value}\" :pick=\"{callback}\" @save=\"noop\" /><div>{{{{ delete state.value }}}}</div></template>").replace('\n', newline);
            assert_late_owner(&source, "const __vize_prop_check_0_pick:");
        }
        for opening in [
            "@[delete(state.value)]=\"noop\"",
            "v-on=\"(delete state.value, {})\"",
        ] {
            let source = cstr!("<!-- 日本 🧭 -->{SCRIPT}<template><Comp {opening} :pick=\"{callback}\" /><div>{{{{ delete state.value }}}}</div></template>").replace('\n', newline);
            assert_late_owner(&source, "const __vize_prop_check_0_pick:");
        }
        for prefix in [
            "<div v-for=\"entry in [state]\">{{ delete entry.value }}</div>",
            "<slot :value=\"delete state.value\" />",
            "<section><Comp v-bind=\"(delete state.value, {})\" /></section>",
        ] {
            let source = cstr!("<!-- 日本 🧭 -->{SCRIPT}<template>{prefix}<Comp :pick=\"{callback}\" @save=\"noop\" /><div>{{{{ delete state.value }}}}</div></template>").replace('\n', newline);
            let owner = if prefix.starts_with("<section>") {
                "const __vize_prop_check_1_pick:"
            } else {
                "const __vize_prop_check_0_pick:"
            };
            assert_late_owner(&source, owner);
        }
    }
}

fn assert_late_owner(source: &str, declaration: &str) {
    let result = type_check_sfc(
        source,
        &SfcTypeCheckOptions::new("src/Min.vue").with_virtual_ts(),
    );
    let generated = result
        .virtual_ts
        .as_deref()
        .expect("complete visible projection");
    let owner = generated
        .find(declaration)
        .expect("the canonical owner remains visible");
    assert_eq!(generated.matches(declaration).count(), 1);
    assert!(
        owner
            > generated
                .find("// Component props type declarations")
                .expect("original check phase")
    );
}
