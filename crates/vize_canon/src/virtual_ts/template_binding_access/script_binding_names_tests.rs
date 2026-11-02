//! Completed semantic facts remain owned by the call plan and borrowed by views.
use super::*;
use crate::virtual_ts::scope::script_top_level_binding_names;

fn names(values: &[&str]) -> FxHashSet<String> {
    values.iter().map(|name| String::from(*name)).collect()
}

fn with_owner<'a>(
    summary: &'a Croquis,
    options: &'a VirtualTsOptions,
    owner: InstanceOwner,
) -> TemplateBindingAccess<'a> {
    TemplateBindingAccess {
        instance: Some(InstanceStorage::Owned(InstanceAccess {
            owner,
            summary,
            options,
            setup_spread_bindings: &[],
            infer_dollar_el: false,
            receiver: String::from("__ctx"),
            cache: RefCell::default(),
            diagnostic_prefixes: RefCell::default(),
        })),
        ..Default::default()
    }
}

fn authored<'a>(
    summary: &'a Croquis,
    options: &'a VirtualTsOptions,
    script_names: FxHashSet<String>,
) -> TemplateBindingAccess<'a> {
    with_owner(
        summary,
        options,
        InstanceOwner::AuthoredCalls {
            eligible: names(&["inherited"]),
            script_names,
        },
    )
}

#[test]
fn complete_semantic_names_are_borrowed_through_nested_scope_views() {
    let script = r#"
import type { Thing as TypeOnly } from './types';
import { action as imported } from './values';
namespace Namespace { export type Inner = string; }
enum Mode { On }
interface Contract { value: string }
type Alias = number;
class Service {}
function plain() {}
const { value: local } = { value: 1 };
const \u0045scapedName = 1;
export default {};
"#;
    let expected = names(&[
        "TypeOnly",
        "imported",
        "Namespace",
        "Mode",
        "Contract",
        "Alias",
        "Service",
        "plain",
        "local",
        "EscapedName",
    ]);
    let parsed = script_top_level_binding_names(script).unwrap();
    assert_eq!(parsed, expected);
    let summary = Croquis::new();
    let options = VirtualTsOptions::default();
    let access = authored(&summary, &options, parsed);
    let borrowed = access.authored_script_binding_names(&summary).unwrap();
    assert_eq!(borrowed, &expected);
    let first = access.for_scope(ScopeId::ROOT);
    let second = first.for_scope(ScopeId::ROOT);
    assert!(std::ptr::eq(
        borrowed,
        first.authored_script_binding_names(&summary).unwrap()
    ));
    assert!(std::ptr::eq(
        borrowed,
        second.authored_script_binding_names(&summary).unwrap()
    ));
    drop(second);
    drop(first);
    assert_eq!(
        access.authored_script_binding_names(&summary),
        Some(&expected)
    );
}

#[test]
fn clean_tsx_and_complete_empty_results_remain_successful_facts() {
    let summary = Croquis::new();
    let options = VirtualTsOptions::default();
    for (script, expected) in [
        (
            "const view = <section />; export default {};",
            names(&["view"]),
        ),
        ("export default {};", FxHashSet::default()),
    ] {
        let parsed = script_top_level_binding_names(script).unwrap();
        assert_eq!(parsed, expected);
        let access = authored(&summary, &options, parsed);
        assert_eq!(
            access.authored_script_binding_names(&summary),
            Some(&expected)
        );
    }
}

#[test]
fn unavailable_public_and_mismatched_owners_leave_the_parser_fallback_available() {
    let summary = Croquis::new();
    let other = Croquis::new();
    let options = VirtualTsOptions::default();
    let missing = TemplateBindingAccess::default();
    let public = with_owner(&summary, &options, InstanceOwner::PublicGlobals);
    let access = authored(&summary, &options, names(&["retained"]));
    assert!(missing.authored_script_binding_names(&summary).is_none());
    assert!(public.authored_script_binding_names(&summary).is_none());
    assert!(access.authored_script_binding_names(&other).is_none());
    assert_eq!(
        access.authored_script_binding_names(&summary),
        Some(&names(&["retained"]))
    );
    assert_eq!(
        script_top_level_binding_names("function warm() { await 1 } export default {};"),
        None
    );
}

#[test]
fn receiver_preparation_retains_the_complete_owned_name_set() {
    use vize_croquis::{Analyzer, AnalyzerOptions};
    let script = "const __ctx = 1; export default {};";
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<button @click=\"inherited()\" />");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let options = VirtualTsOptions::default();
    let parsed = script_top_level_binding_names(script).unwrap();
    assert_eq!(parsed, names(&["__ctx"]));
    let mut access = authored(&summary, &options, parsed);
    let before = std::ptr::from_ref(access.authored_script_binding_names(&summary).unwrap());
    access.prepare_instance_context();
    let after = access.authored_script_binding_names(&summary).unwrap();
    assert_eq!(std::ptr::from_ref(after), before);
    assert_eq!(after, &names(&["__ctx"]));
    assert!(access.has_authored_instance_calls());
}
