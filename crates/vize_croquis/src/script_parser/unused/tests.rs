use crate::script_parser::parse::{parse_script_setup_for_unused, parse_script_setup_with_generic};
use crate::{Drawer, DrawerOptions};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::cstr;

const SOURCE: &str = "import { Kind } from './kind'; const unread = 1;";

#[test]
fn generic_scopes_resolve_free_reads_without_spelling_based_admission() {
    for (generic, expected) in [
        ("T extends Kind.A | Kind.B", vec!["unread"]),
        ("T = { value: Kind }", vec!["unread"]),
        ("T extends keyof typeof Kind", vec!["unread"]),
        ("Kind, T extends Kind", vec!["Kind", "unread"]),
        ("T extends { Kind: string }", vec!["Kind", "unread"]),
        ("T extends import('./kind').Kind", vec!["Kind", "unread"]),
        (
            "T extends (unknown extends infer Kind ? Kind : never)",
            vec!["Kind", "unread"],
        ),
    ] {
        let result =
            parse_script_setup_for_unused::<false>(SOURCE, Some(generic), false, true, false);
        let actual: Vec<_> = result
            .unused_bindings
            .iter()
            .map(|name| name.as_str())
            .collect();
        assert_eq!(actual, expected, "{generic}");
    }
}

#[test]
fn invalid_or_escaped_type_parameters_do_not_prove_an_unread_binding() {
    for generic in ["T extends", "T>() {}); Kind; (function<U", "T, T"] {
        let result =
            parse_script_setup_for_unused::<false>(SOURCE, Some(generic), false, true, false);
        assert_eq!(result.unused_bindings.len(), 0, "{generic}");
    }
}

#[test]
fn unused_demand_preserves_all_other_owned_script_metadata() {
    let generic = Some("T extends Kind.A");
    let ordinary = parse_script_setup_with_generic(SOURCE, generic);
    let mut demanded = parse_script_setup_for_unused::<false>(SOURCE, generic, false, true, false);
    assert_eq!(
        demanded
            .unused_bindings
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        ["unread"]
    );
    demanded.unused_bindings.clear();
    assert_eq!(cstr!("{demanded:?}"), cstr!("{ordinary:?}"));
}

#[test]
fn retained_program_entry_keeps_original_binding_spans_and_generic_reads() {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, SOURCE, SourceType::ts()).parse();
    assert_eq!(parsed.diagnostics.len(), 0);
    let mut drawer = Drawer::with_options(DrawerOptions::for_lint()).with_unused_bindings();
    drawer.draw_script_setup_program(&parsed.program, SOURCE, Some("T extends Kind.A"));
    let result = drawer.finish();
    assert_eq!(
        result
            .unused_bindings
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        ["unread"]
    );
    assert_eq!(result.binding_spans.get("Kind"), Some(&(9, 13)));
    assert_eq!(result.binding_spans.get("unread"), Some(&(37, 43)));
}
