use super::prop_binding_identifier;

#[test]
fn authored_unicode_prop_locals_follow_the_same_oxc_identifier_authority_as_anchors() {
    let names = [
        "café",
        "caf_",
        "résumé",
        "日本語",
        "𐐀value",
        "a\u{200c}b",
        "$value",
        "_private",
        "ascii",
    ];
    assert_eq!(
        names.map(prop_binding_identifier),
        names.map(vize_carton::String::from),
    );
}

#[test]
fn reserved_and_nonidentifier_prop_keys_keep_the_existing_synthetic_alias_policy() {
    assert_eq!(
        [
            "class",
            "default",
            "await",
            "data-id",
            "onUpdate:modelValue",
            "9start",
            "😀"
        ]
        .map(prop_binding_identifier),
        [
            "_class",
            "_default",
            "_await",
            "data_id",
            "onUpdate_modelValue",
            "_9start",
            "_"
        ]
        .map(vize_carton::String::from),
    );
}

fn emit_binding(
    script: &str,
    name: &str,
) -> (
    vize_carton::String,
    Vec<crate::virtual_ts::VizeMapping>,
    Vec<crate::virtual_ts::VizeSemanticLink>,
) {
    let mut drawer = vize_croquis::Drawer::with_options(vize_croquis::DrawerOptions {
        analyze_script: true,
        ..Default::default()
    });
    drawer.analyze_script_setup(script);
    let summary = drawer.finish();
    let mut code = crate::virtual_ts::TextBuf::default();
    let mut mappings = Vec::new();
    let mut links = Vec::new();
    let mut default_keys = Vec::new();
    let offset = |value: usize| value + 17;
    super::PropBindingMappings::new(
        &mut mappings,
        &mut links,
        &mut default_keys,
        &summary,
        Some(script),
        &offset,
    )
    .emit(&mut code, "props", "$ResolvedProps", name, true);
    (code.into_compact(), mappings, links)
}

#[test]
fn explicit_model_template_binding_keeps_its_croquis_owned_authored_name() {
    for name in ["x", "saveItem"] {
        for quote in ['\'', '"'] {
            for newline in ["\n", "\r\n"] {
                let literal = vize_carton::cstr!("{quote}{name}{quote}");
                let script = vize_carton::cstr!(
                    "export interface Props {{ value?: string }}{newline}defineProps<Props>();{newline}defineModel<string>({literal}, {{ default: \"\" }});{newline}const untouched = {literal};{newline}"
                );
                let start = script.find(literal.as_str()).unwrap() + 1;
                let (code, mappings, links) = emit_binding(&script, name);
                assert_eq!(
                    mappings.len(),
                    1,
                    "only the actual model literal owns the local"
                );
                assert_eq!(mappings[0].src_range, start + 17..start + 17 + name.len());
                assert!(mappings[0].sub_spans.is_empty());
                assert_eq!(code.get(mappings[0].gen_range.clone()), Some(name));
                assert_eq!(links.len(), 1, "the existing binding edge must remain");
                assert_eq!(links[0].source_range, mappings[0].gen_range);
                assert_eq!(code.get(links[0].target_range.clone()), Some(name));
                assert_eq!(
                    links[0].kind,
                    crate::virtual_ts::VizeSemanticLinkKind::VueTemplatePropBinding,
                );
                assert_eq!(
                    code.as_str(),
                    vize_carton::cstr!("  const {name} = props[\"{name}\"] as Exclude<$ResolvedProps[\"{name}\"], undefined>;\n  void {name};\n").as_str(),
                );
            }
        }
    }
}

#[test]
fn implicit_computed_escaped_and_unrelated_names_gain_no_model_source_anchor() {
    for (script, name) in [
        ("defineModel<string>();", "modelValue"),
        ("defineModel<string>('class');", "class"),
        ("defineModel<string>('save-item');", "save-item"),
        (
            "const name = 'saveItem'; defineModel<string>(name);",
            "modelValue",
        ),
        (
            "const name = 'saveItem'; defineModel<string>(name);",
            "saveItem",
        ),
        ("defineModel<string>('save\\u0049tem');", "saveItem"),
        (
            "defineModel<string>('other'); const untouched = 'saveItem';",
            "saveItem",
        ),
        ("const untouched = 'saveItem';", "saveItem"),
    ] {
        let (_, mappings, links) = emit_binding(script, name);
        assert!(
            mappings.is_empty(),
            "no explicit copied model name: {script}"
        );
        assert_eq!(
            links.len(),
            1,
            "unknown ownership does not remove the old edge"
        );
    }
}

#[test]
fn an_authored_prop_keeps_precedence_when_a_same_named_model_is_present() {
    let script = "defineProps<{ saveItem: string }>(); defineModel<string>('saveItem');";
    let (_, mappings, _) = emit_binding(script, "saveItem");
    let start = script.find("saveItem").unwrap() + 17;
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].src_range, start..start + "saveItem".len());
}
