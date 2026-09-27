use crate::virtual_ts::generate_virtual_ts;

/// Generate the virtual TS of a component without a script block.
fn generate(template: &str) -> vize_carton::String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts(&summary, None, Some(&root), 0).code
}

/// A slot rendered by several same-named outlets is one entry whose payload
/// merges every outlet, in place of one entry per outlet: those intersected
/// into an overloaded slot function whose parameter the parent inferred from
/// the last outlet only, so `#panel="{ viewMode }"` became `{}` when a bare
/// `<slot name="panel" />` followed the bound one.
#[test]
fn same_named_outlets_share_one_merged_entry() {
    let code = generate(
        r#"<div>
  <template v-if="isPC">
    <slot name="panel" :viewMode="viewMode" />
    <slot name="side" viewMode="sp" />
  </template>
  <template v-else>
    <slot name="panel" />
    <slot name="side" :viewMode="viewMode" />
  </template>
  <slot name="footer" />
</div>"#,
    );

    assert_eq!(
        code.matches("\"panel\"?:").count(),
        1,
        "same-named outlets must produce a single slots entry:\n{code}"
    );
    assert!(
        code.contains("\"panel\"?: (props: __VizeSlotOutletUnion<typeof __vize_slot_payload_"),
        "the merged entry must take the payload union:\n{code}"
    );
    assert_eq!(
        code.matches("__VizeSlotOutletUnion<typeof __vize_slot_payload_")
            .count(),
        2,
        "both duplicated names merge, the unique one does not:\n{code}"
    );
    assert!(
        code.contains("\"footer\"?: (props: typeof __vize_slot_payload_"),
        "a single outlet keeps its plain payload type:\n{code}"
    );
    assert_eq!(
        code.matches("type __VizeSlotOutletUnion<__U> =").count(),
        1,
        "the union helper is declared once:\n{code}"
    );
    assert!(
        code.contains("\"viewMode\": \"sp\" as const,"),
        "a static outlet attribute keeps its literal type in the inferred payload:\n{code}"
    );
}

/// The helper is dead code for a template whose outlet names are unique, and
/// `noUnusedLocals` consumers would report it.
#[test]
fn unique_outlets_do_not_declare_the_union_helper() {
    let code = generate(r#"<div><slot name="panel" :viewMode="viewMode" /><slot /></div>"#);
    assert!(
        !code.contains("__VizeSlotOutletUnion"),
        "no merged entry, no helper:\n{code}"
    );
    assert!(
        code.contains("\"panel\"?: (props: typeof __vize_slot_payload_"),
        "{code}"
    );
}
