//! Nuxt lazy hydration macro expansion.
//!
//! Nuxt exposes `defineLazyHydrationComponent` as a compiler macro. Expanding it
//! in the SFC compiler keeps the behavior independent from a specific bundler
//! transform and prevents the empty runtime stub from leaking into output.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, BindingPattern, Declaration, Expression, Statement,
    VariableDeclaration,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{String, ToCompactString};

/// A rewritten script and the module preamble it needs.
#[derive(Debug, Clone)]
pub(crate) struct LazyHydrationTransform {
    pub code: String,
    pub preamble: String,
}

#[derive(Debug)]
struct Edit {
    start: usize,
    end: usize,
    replacement: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LazyHydrationStrategy {
    Visible,
    Idle,
    Interaction,
    MediaQuery,
    If,
    Time,
    Never,
}

impl LazyHydrationStrategy {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "visible" => Some(Self::Visible),
            "idle" => Some(Self::Idle),
            "interaction" => Some(Self::Interaction),
            "mediaQuery" => Some(Self::MediaQuery),
            "if" => Some(Self::If),
            "time" => Some(Self::Time),
            "never" => Some(Self::Never),
            _ => None,
        }
    }

    const fn factory_name(self) -> &'static str {
        match self {
            Self::Visible => "__vizeCreateLazyVisibleComponent",
            Self::Idle => "__vizeCreateLazyIdleComponent",
            Self::Interaction => "__vizeCreateLazyInteractionComponent",
            Self::MediaQuery => "__vizeCreateLazyMediaQueryComponent",
            Self::If => "__vizeCreateLazyIfComponent",
            Self::Time => "__vizeCreateLazyTimeComponent",
            Self::Never => "__vizeCreateLazyNeverComponent",
        }
    }
}

/// Expand Nuxt `defineLazyHydrationComponent(strategy, loader)` calls.
pub(crate) fn transform_lazy_hydration_macros(content: &str) -> Option<LazyHydrationTransform> {
    if !content.contains("defineLazyHydrationComponent") {
        return None;
    }

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, content, source_type).parse();
    if ret.panicked {
        return None;
    }

    let mut edits = Vec::new();
    let mut strategies = Vec::new();

    for stmt in ret.program.body.iter() {
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                collect_lazy_hydration_edits(var_decl, content, &mut edits, &mut strategies)?;
            }
            Statement::ExportNamedDeclaration(export_decl) => {
                if let Some(Declaration::VariableDeclaration(var_decl)) =
                    export_decl.declaration.as_ref()
                {
                    collect_lazy_hydration_edits(var_decl, content, &mut edits, &mut strategies)?;
                }
            }
            _ => {}
        }
    }

    if edits.is_empty() {
        return None;
    }

    edits.sort_by_key(|edit| edit.start);
    let mut code = String::with_capacity(
        content.len() + edits.iter().map(|e| e.replacement.len()).sum::<usize>(),
    );
    let mut cursor = 0usize;
    for edit in edits {
        if edit.start < cursor {
            continue;
        }
        code.push_str(&content[cursor..edit.start]);
        code.push_str(&edit.replacement);
        cursor = edit.end;
    }
    code.push_str(&content[cursor..]);

    Some(LazyHydrationTransform {
        code,
        preamble: build_lazy_hydration_preamble(&strategies),
    })
}

fn collect_lazy_hydration_edits(
    var_decl: &VariableDeclaration<'_>,
    content: &str,
    edits: &mut Vec<Edit>,
    strategies: &mut Vec<LazyHydrationStrategy>,
) -> Option<()> {
    for declarator in var_decl.declarations.iter() {
        let BindingPattern::BindingIdentifier(_) = &declarator.id else {
            continue;
        };
        let Some(Expression::CallExpression(call)) = declarator.init.as_ref() else {
            continue;
        };
        let Expression::Identifier(callee) = &call.callee else {
            continue;
        };
        if callee.name.as_str() != "defineLazyHydrationComponent" {
            continue;
        }
        if call.arguments.len() < 2 {
            continue;
        }

        let Some(strategy) = strategy_from_argument(&call.arguments[0]) else {
            continue;
        };
        let Some(import_id) = import_id_from_loader(&call.arguments[1]) else {
            continue;
        };
        let Some(loader_source) = argument_source(&call.arguments[1], content) else {
            continue;
        };

        let start = call.span.start as usize;
        let end = call.span.end as usize;
        if start > end || end > content.len() {
            continue;
        }

        let import_id = serde_json::to_string(import_id).ok()?;
        let mut replacement =
            String::with_capacity(strategy.factory_name().len() + import_id.len() + 8);
        replacement.push_str(strategy.factory_name());
        replacement.push('(');
        replacement.push_str(&import_id);
        replacement.push_str(", ");
        replacement.push_str(&loader_source);
        replacement.push(')');

        edits.push(Edit {
            start,
            end,
            replacement,
        });
        if !strategies.contains(&strategy) {
            strategies.push(strategy);
        }
    }

    Some(())
}

fn strategy_from_argument(arg: &Argument<'_>) -> Option<LazyHydrationStrategy> {
    let Argument::StringLiteral(lit) = arg else {
        return None;
    };
    LazyHydrationStrategy::from_name(lit.value.as_str())
}

fn import_id_from_loader<'a>(arg: &'a Argument<'a>) -> Option<&'a str> {
    let Argument::ArrowFunctionExpression(arrow) = arg else {
        return None;
    };
    import_literal_from_arrow(arrow)
}

fn import_literal_from_arrow<'a>(arrow: &'a ArrowFunctionExpression<'a>) -> Option<&'a str> {
    if arrow.expression {
        let Some(Statement::ExpressionStatement(expr_stmt)) = arrow.body.statements.first() else {
            return None;
        };
        return import_literal_from_expression(&expr_stmt.expression);
    }

    for stmt in arrow.body.statements.iter() {
        let Statement::ReturnStatement(return_stmt) = stmt else {
            continue;
        };
        let Some(argument) = return_stmt.argument.as_ref() else {
            continue;
        };
        if let Some(literal) = import_literal_from_expression(argument) {
            return Some(literal);
        }
    }

    None
}

fn import_literal_from_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::ImportExpression(import_expr) => match &import_expr.source {
            Expression::StringLiteral(lit) => Some(lit.value.as_str()),
            _ => None,
        },
        Expression::ParenthesizedExpression(paren) => {
            import_literal_from_expression(&paren.expression)
        }
        Expression::AwaitExpression(await_expr) => {
            import_literal_from_expression(&await_expr.argument)
        }
        Expression::ConditionalExpression(conditional) => {
            import_literal_from_expression(&conditional.consequent)
                .or_else(|| import_literal_from_expression(&conditional.alternate))
        }
        Expression::StaticMemberExpression(member) => {
            import_literal_from_expression(&member.object)
        }
        Expression::ComputedMemberExpression(member) => {
            import_literal_from_expression(&member.object)
        }
        Expression::PrivateFieldExpression(member) => {
            import_literal_from_expression(&member.object)
        }
        Expression::CallExpression(call) => import_literal_from_expression(&call.callee),
        _ => None,
    }
}

fn argument_source(arg: &Argument<'_>, source: &str) -> Option<String> {
    let span = arg.span();
    let start = span.start as usize;
    let end = span.end as usize;
    if start > end || end > source.len() {
        return None;
    }
    Some((&source[start..end]).to_compact_string())
}

fn build_lazy_hydration_preamble(strategies: &[LazyHydrationStrategy]) -> String {
    let mut preamble = String::from(
        "import { defineAsyncComponent as __vizeDefineAsyncComponent, defineComponent as __vizeDefineComponent, h as __vizeH, hydrateOnIdle as __vizeHydrateOnIdle, hydrateOnInteraction as __vizeHydrateOnInteraction, hydrateOnMediaQuery as __vizeHydrateOnMediaQuery, hydrateOnVisible as __vizeHydrateOnVisible, mergeProps as __vizeMergeProps } from 'vue'\n",
    );
    preamble.push_str("import { useNuxtApp as __vizeUseNuxtApp } from '#app/nuxt'\n");
    preamble.push_str("const __vizeDefineLazyHydrationComponent = (props, defineStrategy) => (id, loader) => __vizeDefineComponent({\n");
    preamble.push_str("  inheritAttrs: false,\n");
    preamble.push_str("  props,\n");
    preamble.push_str("  emits: ['hydrated'],\n");
    preamble.push_str("  setup(props2, ctx) {\n");
    preamble.push_str("    if (import.meta.server) {\n");
    preamble.push_str("      const nuxtApp = __vizeUseNuxtApp()\n");
    preamble.push_str("      nuxtApp.hook('app:rendered', ({ ssrContext }) => {\n");
    preamble.push_str("        ssrContext['~lazyHydratedModules'] ||= new Set()\n");
    preamble.push_str("        ssrContext['~lazyHydratedModules'].add(id)\n");
    preamble.push_str("      })\n");
    preamble.push_str("    }\n");
    preamble.push_str("    const child = __vizeDefineAsyncComponent({ loader })\n");
    preamble.push_str("    const comp = __vizeDefineAsyncComponent({ hydrate: defineStrategy(props2), loader: () => Promise.resolve(child) })\n");
    preamble.push_str("    const onVnodeMounted = () => { ctx.emit('hydrated') }\n");
    preamble.push_str("    return () => __vizeH(comp, __vizeMergeProps(ctx.attrs, { onVnodeMounted }), ctx.slots)\n");
    preamble.push_str("  },\n");
    preamble.push_str("})\n");

    if strategies.contains(&LazyHydrationStrategy::Visible) {
        preamble.push_str("const __vizeCreateLazyVisibleComponent = __vizeDefineLazyHydrationComponent({ hydrateOnVisible: { type: [Object, Boolean], required: false, default: true } }, (props) => __vizeHydrateOnVisible(props.hydrateOnVisible === true ? void 0 : props.hydrateOnVisible))\n");
    }
    if strategies.contains(&LazyHydrationStrategy::Idle) {
        preamble.push_str("const __vizeCreateLazyIdleComponent = __vizeDefineLazyHydrationComponent({ hydrateOnIdle: { type: [Number, Boolean], required: false, default: true } }, (props) => props.hydrateOnIdle === 0 ? void 0 : __vizeHydrateOnIdle(props.hydrateOnIdle === true ? void 0 : props.hydrateOnIdle))\n");
    }
    if strategies.contains(&LazyHydrationStrategy::Interaction) {
        preamble.push_str(
            "const __vizeDefaultInteractionEvents = ['pointerenter', 'click', 'focus']\n",
        );
        preamble.push_str("const __vizeCreateLazyInteractionComponent = __vizeDefineLazyHydrationComponent({ hydrateOnInteraction: { type: [String, Array], required: false, default: __vizeDefaultInteractionEvents } }, (props) => __vizeHydrateOnInteraction(props.hydrateOnInteraction === true ? __vizeDefaultInteractionEvents : props.hydrateOnInteraction || __vizeDefaultInteractionEvents))\n");
    }
    if strategies.contains(&LazyHydrationStrategy::MediaQuery) {
        preamble.push_str("const __vizeCreateLazyMediaQueryComponent = __vizeDefineLazyHydrationComponent({ hydrateOnMediaQuery: { type: String, required: true } }, (props) => __vizeHydrateOnMediaQuery(props.hydrateOnMediaQuery))\n");
    }
    if strategies.contains(&LazyHydrationStrategy::If) {
        preamble.push_str("const __vizeCreateLazyIfComponent = __vizeDefineLazyHydrationComponent({ hydrateWhen: { type: Boolean, default: true } }, (props) => props.hydrateWhen ? void 0 : () => {})\n");
    }
    if strategies.contains(&LazyHydrationStrategy::Time) {
        preamble.push_str("const __vizeCreateLazyTimeComponent = __vizeDefineLazyHydrationComponent({ hydrateAfter: { type: Number, required: true } }, (props) => props.hydrateAfter === 0 ? void 0 : (hydrate) => { const id = setTimeout(hydrate, props.hydrateAfter); return () => clearTimeout(id) })\n");
    }
    if strategies.contains(&LazyHydrationStrategy::Never) {
        preamble.push_str("const __vizeHydrateNever = () => {}\n");
        preamble.push_str("const __vizeCreateLazyNeverComponent = __vizeDefineLazyHydrationComponent({ hydrateNever: { type: Boolean, required: false, default: true } }, () => __vizeHydrateNever)\n");
    }
    preamble.push('\n');
    preamble
}

#[cfg(test)]
mod tests {
    use super::transform_lazy_hydration_macros;

    #[test]
    fn transforms_define_lazy_hydration_component() {
        let content = r#"const LazyHydrationMyComponent = defineLazyHydrationComponent(
  'visible',
  () => import('./components/MyComponent.vue'),
)
"#;

        let transformed =
            transform_lazy_hydration_macros(content).expect("macro should be transformed");

        assert!(!transformed.code.contains("defineLazyHydrationComponent"));
        assert!(transformed
            .code
            .contains("__vizeCreateLazyVisibleComponent"));
        assert!(transformed
            .code
            .contains("\"./components/MyComponent.vue\", () => import"));
        assert!(transformed
            .preamble
            .contains("hydrateOnVisible as __vizeHydrateOnVisible"));
        assert!(transformed
            .preamble
            .contains("const __vizeCreateLazyVisibleComponent"));
    }

    #[test]
    fn ignores_dynamic_strategy_or_loader() {
        let content = r#"const strategy = 'visible'
const LazyHydrationMyComponent = defineLazyHydrationComponent(
  strategy,
  source,
)
"#;

        assert!(transform_lazy_hydration_macros(content).is_none());
    }

    #[test]
    fn transforms_exported_variable_declarations() {
        let content = r#"export const LazyHydrationMyComponent = defineLazyHydrationComponent(
  'time',
  () => import('./components/MyComponent.vue'),
)
"#;

        let transformed =
            transform_lazy_hydration_macros(content).expect("macro should be transformed");

        assert!(!transformed.code.contains("defineLazyHydrationComponent"));
        assert!(transformed.code.contains("__vizeCreateLazyTimeComponent"));
        assert!(transformed
            .preamble
            .contains("const __vizeCreateLazyTimeComponent"));
    }
}
