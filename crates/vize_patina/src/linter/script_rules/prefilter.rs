use memchr::memmem;
use vize_module::{ModuleExpression, ModuleExpressionKind, ModuleOperationKind, ModuleSyntax};

use super::{
    RULE_NO_RESTRICTED_GLOBALS, RULE_NO_RESTRICTED_MEMBERS, RULE_PINIA_PREFER_STORE_TO_REFS,
    RULE_PREFER_COMPUTED, RULE_PREFER_IMPORT_FROM_VUE, RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
};

pub(super) fn script_rule_may_match(rule_name: &str, source: &str) -> bool {
    let bytes = source.as_bytes();
    match rule_name {
        RULE_PINIA_PREFER_STORE_TO_REFS => memmem::find(bytes, b"Store").is_some(),
        RULE_VUE_ROUTER_PREFER_NAMED_PUSH => {
            (memmem::find(bytes, b".push").is_some() || memmem::find(bytes, b".replace").is_some())
                && (memmem::find(bytes, b"'/").is_some() || memmem::find(bytes, b"\"/").is_some())
                && (memmem::find(bytes, b"router").is_some()
                    || memmem::find(bytes, b"Router").is_some())
        }
        RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT => {
            memmem::find(bytes, b"toMatchSnapshot").is_some()
                && memmem::find(bytes, b".html").is_some()
        }
        RULE_PREFER_COMPUTED => memmem::find(bytes, b"watch").is_some(),
        RULE_PREFER_IMPORT_FROM_VUE => memmem::find(bytes, b"@vue/").is_some(),
        RULE_NO_RESTRICTED_GLOBALS => {
            memmem::find(bytes, b"process").is_some()
                || memmem::find(bytes, b"localStorage").is_some()
                || memmem::find(bytes, b"sessionStorage").is_some()
        }
        RULE_NO_RESTRICTED_MEMBERS => false,
        _ => true,
    }
}

pub(super) fn module_rule_may_match(rule_name: &str, module: &ModuleSyntax) -> bool {
    if !script_rule_may_match(rule_name, &module.source) {
        return false;
    }
    match rule_name {
        RULE_PREFER_IMPORT_FROM_VUE => module.imports.iter().any(|import| {
            matches!(
                import.specifier.as_ref(),
                "@vue/runtime-core" | "@vue/runtime-dom" | "@vue/reactivity" | "@vue/shared"
            )
        }),
        RULE_NO_RESTRICTED_GLOBALS => module.references.iter().any(|reference| {
            matches!(
                reference.name.as_ref(),
                "process" | "localStorage" | "sessionStorage"
            )
        }),
        RULE_PINIA_PREFER_STORE_TO_REFS => {
            module.operations.operations.is_empty()
                || module
                    .operations
                    .operations
                    .iter()
                    .any(|operation| match &operation.kind {
                        ModuleOperationKind::Binding { initializer, .. } => {
                            initializer.as_ref().is_some_and(expression_mentions_store)
                        }
                        ModuleOperationKind::Assignment { value, .. }
                        | ModuleOperationKind::Call(value)
                        | ModuleOperationKind::Await(value) => expression_mentions_store(value),
                        ModuleOperationKind::Return(value) => {
                            value.as_ref().is_some_and(expression_mentions_store)
                        }
                    })
        }
        _ => true,
    }
}

fn expression_mentions_store(expression: &ModuleExpression) -> bool {
    match &expression.kind {
        ModuleExpressionKind::Identifier(name) => store_name(name),
        ModuleExpressionKind::Path(path) => path.last().is_some_and(|name| store_name(name)),
        ModuleExpressionKind::Call {
            callee, arguments, ..
        } => expression_mentions_store(callee) || arguments.iter().any(expression_mentions_store),
        ModuleExpressionKind::Object { properties } => properties
            .iter()
            .any(|property| expression_mentions_store(&property.value)),
        ModuleExpressionKind::Array(items) => items.iter().flatten().any(expression_mentions_store),
        ModuleExpressionKind::Await(inner) | ModuleExpressionKind::Spread(inner) => {
            expression_mentions_store(inner)
        }
        ModuleExpressionKind::Unknown(text) => text.contains("Store"),
        ModuleExpressionKind::Literal { .. } | ModuleExpressionKind::Function { .. } => false,
    }
}

fn store_name(name: &str) -> bool {
    name.starts_with("use") && name.ends_with("Store")
}
