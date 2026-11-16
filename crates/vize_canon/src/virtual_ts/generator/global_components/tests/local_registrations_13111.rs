//! #13111: keys of the SFC's own `components` option are local components.
use crate::virtual_ts::VirtualTsOptions;
use crate::virtual_ts::component_reference::component_binding_reference;
use vize_carton::{FxHashSet, String};
use vize_croquis::{Analyzer, AnalyzerOptions};

const SCRIPT: &str = r#"
import { defineAsyncComponent, defineComponent } from "vue";
import Child from "./Child.vue";
import * as ui from "./ui";

export default defineComponent({
  components: {
    Child,
    Aliased: Child,
    MemberCard: ui.Card,
    Lazy: defineAsyncComponent(() => import("./Child.vue")),
  },
});
"#;

const TEMPLATE: &str = concat!(
    "<Child label=\"a\" /><Aliased label=\"b\" /><aliased label=\"c\" />",
    "<member-card label=\"d\" /><Lazy /><NotRegistered />",
);

fn analyze(template: &vize_relief::RootNode<'_>) -> vize_croquis::Croquis {
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(SCRIPT);
    analyzer.analyze_template(template);
    analyzer.finish()
}

#[test]
fn registered_keys_resolve_to_their_values_and_skip_the_unknown_probe() {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse_with_options(
        &allocator,
        TEMPLATE,
        vize_relief::options::ParserOptions {
            is_native_tag: Some(vize_carton::is_native_tag),
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{errors:?}");
    let summary = analyze(&root);
    let options = VirtualTsOptions::default();
    let type_only = FxHashSet::default();
    let references = ["Aliased", "aliased", "member-card"]
        .map(|tag| component_binding_reference(&summary, &options, &type_only, tag));
    assert_eq!(
        references,
        [
            String::from("Child"),
            String::from("Child"),
            String::from("ui.Card")
        ]
    );

    let mut code = crate::virtual_ts::TextBuf::default();
    let mut mappings = Vec::new();
    let mut plan =
        super::super::GlobalComponentPlan::new((&summary, Some(&root)), false, true, None);
    plan.emit(
        &mut code,
        &summary,
        &options,
        &FxHashSet::from_iter(["Child", "ui"]),
        &type_only,
        Some(super::super::GlobalComponentDiagnostics {
            mappings: &mut mappings,
            template_offset: 0,
        }),
    );
    // The opaque `Lazy` value keeps the registry stub, but only the
    // unregistered tag is probed as unknown.
    const EXPECTED: &str = concat!(
        "\n// Global component stubs (vue module augmentations)\n",
        "type $GlobalComponent<__K extends string | number | symbol, __F extends string | number | symbol = __K, __L extends string | number | symbol = __F, __G = import(\"vue\").GlobalComponents, __C = __K extends keyof __G ? __G[__K] : __F extends keyof __G ? __G[__F] : __L extends keyof __G ? __G[__L] : any> = unknown extends __C ? any : __C;\n",
        "declare const Lazy: $GlobalComponent<\"Lazy\">;\n",
        "const { \"NotRegistered\": __vize_global_component_5 } = {} as (",
        "\"NotRegistered\" extends keyof import('vue').GlobalComponents ? { \"NotRegistered\": unknown } : ",
        "\"NotRegistered\" extends keyof import('vue').GlobalComponents ? { \"NotRegistered\": unknown } : ",
        "\"NotRegistered\" extends keyof import('vue').GlobalComponents ? { \"NotRegistered\": unknown } : {});\n",
        "void __vize_global_component_5;\n",
        "declare const NotRegistered: $GlobalComponent<\"NotRegistered\">;\n",
    );
    assert_eq!(code.into_compact(), EXPECTED);
    assert_eq!(mappings.len(), 1);
}
