//! The source-built plugin's `props: true` projection, admitted per project.

use super::{FILE, ROUTE, SourceType, output, plan};
use crate::virtual_ts::VirtualTsOutput;

const PROPS: &str =
    "import('vue-router/experimental')._RoutePropsForFilePath<\"/src/pages/[id].vue\">";
const SELF: &str = "\"./[id].vue\"";

fn admitted(script: &str, javascript: bool) -> (VirtualTsOutput, bool, usize) {
    let language = if javascript {
        SourceType::mjs()
    } else {
        SourceType::ts()
    };
    let plan = plan(script, |at| at, 0..script.len(), language, true).unwrap();
    let mut result = output(script);
    let imported = super::super::apply(
        &mut result,
        &plan,
        script,
        |at| at,
        0..script.len(),
        (FILE, Some(SELF)),
        None,
    );
    (result, imported, plan.props_calls.len())
}

fn page(generic: bool) -> vize_carton::String {
    if generic {
        vize_carton::cstr!("definePage<{FILE}, typeof import({SELF}).default>(")
    } else {
        vize_carton::cstr!("definePage<{FILE}>(")
    }
}

#[test]
fn props_true_adds_the_component_generic_and_wraps_the_authored_contract() {
    for newline in ["\n", "\r\n"] {
        let script = "import { definePage } from 'vue-router/experimental';\nconst props = defineProps<{ extra?: string } /* 雪 */>();\ndefinePage({ props: true });\nconst bad = props.missing;".replace('\n', newline);
        let (result, imported, calls) = admitted(&script, false);
        assert!(imported);
        assert_eq!(calls, 1);
        assert_eq!(
            result.code,
            script.replace("definePage(", &page(true)).replace(
                "<{ extra?: string }",
                &format!("<{PROPS} & ({{ extra?: string }})")
            )
        );
        for token in ["props.missing", "bad", "/* 雪 */"] {
            let authored = script.find(token).unwrap();
            let generated = result.code.find(token).unwrap();
            assert_eq!(result.mapping.to_generated(authored), Some(generated));
            assert_eq!(
                result
                    .mapping
                    .diagnostic_range_to_authored(generated, generated + token.len()),
                Some((authored, authored + token.len()))
            );
        }
    }
}

#[test]
fn untyped_define_props_receives_the_helper_alone_in_either_statement_order() {
    let script = "definePage({ props: true }); const props = withDefaults(defineProps(), {}); defineProps<{ a: 1 }>(); defineProps({ a: String });";
    let (result, imported, calls) = admitted(script, false);
    assert!(imported);
    assert_eq!(calls, 2);
    assert_eq!(
        result.code,
        script
            .replace("definePage(", &page(true))
            .replace("defineProps()", &format!("defineProps<{PROPS}>()"))
            .replace("<{ a: 1 }>", &format!("<{PROPS} & ({{ a: 1 }})>"))
    );
}

#[test]
fn only_a_literal_true_option_passes_params_but_any_other_option_checks_the_component() {
    for (option, generic) in [
        ("props: false", false),
        ("props: (to: unknown) => ({})", true),
        ("props: {}", true),
        ("props: !0", true),
        ("props: (true)", true),
        ("'props': true", false),
        ("['props']: true", false),
        ("props() { return true }", false),
        ("name: 'x'", false),
    ] {
        let script = format!("definePage({{ {option} }}); defineProps<{{ a: 1 }}>();");
        let (result, imported, calls) = admitted(&script, false);
        assert!(!imported, "{option}");
        assert_eq!(calls, 0, "{option}");
        assert_eq!(
            result.code,
            script.replace("definePage(", &page(generic)),
            "{option}"
        );
    }
}

#[test]
fn explicit_generic_and_nested_pages_follow_the_plugin_statement_rule() {
    let explicit = "definePage<'x'>({ props: true }); defineProps<{ a: 1 }>();";
    let (result, imported, _) = admitted(explicit, false);
    assert!(imported);
    assert_eq!(
        result.code,
        explicit.replace("<{ a: 1 }>", &format!("<{PROPS} & ({{ a: 1 }})>"))
    );
    // Only setup statements enable params as props; nested calls do not.
    let nested = "if (1) { definePage({ props: true }) } defineProps<{ a: 1 }>();";
    let (result, imported, calls) = admitted(nested, false);
    assert!(!imported);
    assert_eq!(calls, 0);
    assert_eq!(result.code, nested.replace("definePage(", &page(true)));
}

#[test]
fn javascript_and_the_published_capability_keep_file_only_page_types() {
    let script =
        "const props = defineProps(); definePage({ props: true }); const route = useRoute();";
    let (result, imported, calls) = admitted(script, true);
    assert!(imported);
    assert_eq!(calls, 0);
    assert_eq!(
        result.code,
        script.replace(
            "useRoute()",
            &format!("(useRoute() as ReturnType<typeof useRoute{ROUTE}>)")
        )
    );
    let typed = "const props = defineProps<{ a: 1 }>(); definePage({ props: true });";
    let plan = plan(typed, |at| at, 0..typed.len(), SourceType::ts(), false).unwrap();
    assert!(plan.props_calls.is_empty());
    let mut result = output(typed);
    assert!(!super::super::apply(
        &mut result,
        &plan,
        typed,
        |at| at,
        0..typed.len(),
        (FILE, None),
        None,
    ));
    assert_eq!(result.code, typed.replace("definePage(", &page(false)));
}
