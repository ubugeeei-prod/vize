//! Compile-time macro artifact extraction.
//!
//! These helpers keep ecosystem macro output independent from any specific
//! bundler hook. The SFC compiler can erase the runtime call while still
//! returning a loadable artifact for tools such as file-based routers.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{String, ToCompactString};
use vize_croquis::macros::{artifact_macro_names, macro_artifact_kind};

use crate::types::SfcMacroArtifact;

pub(crate) fn extract_macro_artifacts(
    content: &str,
    absolute_offset: usize,
) -> Vec<SfcMacroArtifact> {
    if !contains_artifact_macro_candidate(content) {
        return Vec::new();
    }

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, content, source_type).parse();

    if ret.panicked {
        return Vec::new();
    }

    let static_imports = collect_static_imports(ret.program.body.iter(), content);
    let mut artifacts = Vec::new();

    for stmt in ret.program.body.iter() {
        let Some(call) = artifact_call_from_statement(stmt) else {
            continue;
        };
        let Some(name) = call_name(call) else {
            continue;
        };
        let Some(kind) = macro_artifact_kind(name) else {
            continue;
        };

        let start = call.span.start as usize;
        let end = call.span.end as usize;
        if start > end || end > content.len() {
            continue;
        }

        let source = (&content[start..end]).to_compact_string();
        let payload = call
            .arguments
            .first()
            .map(|arg| argument_source(arg, content))
            .filter(|source| !source.trim().is_empty())
            .unwrap_or_else(|| "{}".into());
        let module_code = build_artifact_module(kind, &payload, &static_imports);

        artifacts.push(SfcMacroArtifact {
            kind: kind.into(),
            name: name.into(),
            source,
            content: payload,
            module_code: Some(module_code),
            start: absolute_offset + start,
            end: absolute_offset + end,
        });
    }

    artifacts
}

pub(crate) fn erase_artifact_macro_statements(content: &str) -> Option<String> {
    if !contains_artifact_macro_candidate(content) {
        return None;
    }

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, content, source_type).parse();

    if ret.panicked {
        return None;
    }

    let mut ranges = Vec::new();
    for stmt in ret.program.body.iter() {
        let Some(call) = artifact_call_from_statement(stmt) else {
            continue;
        };
        let Some(name) = call_name(call) else {
            continue;
        };
        if macro_artifact_kind(name).is_none() {
            continue;
        }

        let span = stmt.span();
        let start = span.start as usize;
        let end = span.end as usize;
        if start <= end && end <= content.len() {
            ranges.push((start, end));
        }
    }

    if ranges.is_empty() {
        return None;
    }

    let mut erased = String::with_capacity(content.len());
    let mut cursor = 0usize;
    for (start, end) in ranges {
        if start < cursor {
            continue;
        }
        erased.push_str(&content[cursor..start]);
        cursor = end;
    }
    erased.push_str(&content[cursor..]);
    Some(erased)
}

fn contains_artifact_macro_candidate(content: &str) -> bool {
    artifact_macro_names().any(|name| content.contains(name))
}

fn artifact_call_from_statement<'a>(stmt: &'a Statement<'a>) -> Option<&'a CallExpression<'a>> {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => unwrap_call_expression(&expr_stmt.expression),
        _ => None,
    }
}

fn unwrap_call_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    match expr {
        Expression::CallExpression(call) => Some(call),
        Expression::TSAsExpression(ts_as) => unwrap_call_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            unwrap_call_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            unwrap_call_expression(&ts_non_null.expression)
        }
        Expression::ParenthesizedExpression(paren) => unwrap_call_expression(&paren.expression),
        _ => None,
    }
}

fn call_name<'a>(call: &'a CallExpression<'a>) -> Option<&'a str> {
    match &call.callee {
        Expression::Identifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

fn argument_source(arg: &Argument<'_>, source: &str) -> String {
    let span = arg.span();
    let start = span.start as usize;
    let end = span.end as usize;
    if start > end || end > source.len() {
        return String::default();
    }
    (&source[start..end]).to_compact_string()
}

fn collect_static_imports<'a>(
    statements: impl Iterator<Item = &'a Statement<'a>>,
    content: &str,
) -> String {
    let mut imports = String::default();

    for stmt in statements {
        if !matches!(stmt, Statement::ImportDeclaration(_)) {
            continue;
        }

        let span = stmt.span();
        let start = span.start as usize;
        let end = span.end as usize;
        if start > end || end > content.len() {
            continue;
        }

        imports.push_str(content[start..end].trim());
        imports.push('\n');
    }

    imports
}

fn build_artifact_module(_kind: &str, payload: &str, static_imports: &str) -> String {
    let mut module_code = String::default();
    module_code.push_str(static_imports);
    module_code.push_str("export default ");
    module_code.push_str(payload.trim());
    module_code.push('\n');
    module_code
}

#[cfg(test)]
mod tests {
    use super::{erase_artifact_macro_statements, extract_macro_artifacts};

    #[test]
    fn extracts_define_page_artifact_module() {
        let content = r#"import { routeMeta } from './route'

definePage({
  name: 'home',
  meta: routeMeta,
})

const msg = 'ready'
"#;

        let artifacts = extract_macro_artifacts(content, 10);

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].kind.as_str(), "vue-router.definePage");
        assert_eq!(artifacts[0].name.as_str(), "definePage");
        assert!(artifacts[0].source.contains("definePage"));
        assert!(artifacts[0].content.contains("routeMeta"));
        assert_eq!(artifacts[0].start, 10 + content.find("definePage").unwrap());
        assert!(artifacts[0]
            .module_code
            .as_ref()
            .unwrap()
            .contains("import { routeMeta } from './route'\nexport default {"));
    }

    #[test]
    fn extracts_define_page_meta_artifact_module() {
        let content = r#"import { pageAlias } from './route'

definePageMeta({
  name: 'docs',
  alias: pageAlias,
  meta: {
    scrollMargin: 180,
  },
})

const msg = 'ready'
"#;

        let artifacts = extract_macro_artifacts(content, 4);

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].kind.as_str(), "nuxt.definePageMeta");
        assert_eq!(artifacts[0].name.as_str(), "definePageMeta");
        assert!(artifacts[0].source.contains("definePageMeta"));
        assert!(artifacts[0].content.contains("scrollMargin"));
        assert_eq!(
            artifacts[0].start,
            4 + content.find("definePageMeta").unwrap()
        );
        assert!(artifacts[0]
            .module_code
            .as_ref()
            .unwrap()
            .contains("import { pageAlias } from './route'\nexport default {"));
    }

    #[test]
    fn extracts_define_route_rules_artifact_module() {
        let content = r#"defineRouteRules({
  prerender: true,
  cache: {
    maxAge: 60,
  },
})

const msg = 'ready'
"#;

        let artifacts = extract_macro_artifacts(content, 2);

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].kind.as_str(), "nuxt.defineRouteRules");
        assert_eq!(artifacts[0].name.as_str(), "defineRouteRules");
        assert!(artifacts[0].source.contains("defineRouteRules"));
        assert!(artifacts[0].content.contains("prerender"));
        assert_eq!(
            artifacts[0].start,
            2 + content.find("defineRouteRules").unwrap()
        );
        assert!(artifacts[0]
            .module_code
            .as_ref()
            .unwrap()
            .starts_with("export default {"));
    }

    #[test]
    fn ignores_content_without_artifact_macro_candidates() {
        let content = r#"const msg = 'ready'
const LazyHydrationMyComponent = defineLazyHydrationComponent(
  'visible',
  () => import('./components/MyComponent.vue'),
)
"#;

        assert!(extract_macro_artifacts(content, 0).is_empty());
        assert!(erase_artifact_macro_statements(content).is_none());
    }

    #[test]
    fn erases_define_page_top_level_statement() {
        let content = r#"definePage({ name: 'home' })
const msg = 'ready'
"#;

        let erased = erase_artifact_macro_statements(content).expect("macro should be erased");

        assert!(!erased.contains("definePage"));
        assert!(erased.contains("const msg = 'ready'"));
    }

    #[test]
    fn erases_define_page_meta_top_level_statement() {
        let content = r#"definePageMeta({ name: 'docs' })
const msg = 'ready'
"#;

        let erased = erase_artifact_macro_statements(content).expect("macro should be erased");

        assert!(!erased.contains("definePageMeta"));
        assert!(erased.contains("const msg = 'ready'"));
    }

    #[test]
    fn erases_define_route_rules_top_level_statement() {
        let content = r#"defineRouteRules({ prerender: true })
const msg = 'ready'
"#;

        let erased = erase_artifact_macro_statements(content).expect("macro should be erased");

        assert!(!erased.contains("defineRouteRules"));
        assert!(erased.contains("const msg = 'ready'"));
    }
}
