use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{
    child_list, rule_allow_list_names, rule_option_bool, type_texts_match_names,
};
use crate::utils::{
    TypeTextKind, classify_type_text, is_array_like_type_texts, split_top_level_type_text,
};

/// Type-aware rule that restricts template literal expression types.
#[derive(Clone, Copy, Debug, Default)]
pub struct RestrictTemplateExpressionsRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "invalidType",
    description: "Invalid type used in template literal expression.",
}];
const LISTENERS: &[&str] = &["TemplateLiteral"];

impl RustLintRule for RestrictTemplateExpressionsRule {
    fn name(&self) -> &'static str {
        "restrict-template-expressions"
    }

    fn docs_description(&self) -> &'static str {
        "Require template literal expressions to be string-compatible."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "TemplateLiteral" || is_tagged_template(node) {
            return;
        }

        let options = RestrictTemplateExpressionsOptions::from_node(node);
        for expression in child_list(node, "expressions") {
            if !is_allowed_expression(expression, &options) {
                ctx.report("invalidType", expression.range);
            }
        }
    }
}

fn is_tagged_template(node: &LintNode) -> bool {
    node.field_str("__parentKind") == Some("TaggedTemplateExpression")
}

#[derive(Clone, Debug)]
struct RestrictTemplateExpressionsOptions {
    allow: Vec<String>,
    allow_any: bool,
    allow_array: bool,
    allow_boolean: bool,
    allow_never: bool,
    allow_nullish: bool,
    allow_number: bool,
    allow_regexp: bool,
}

impl RestrictTemplateExpressionsOptions {
    fn from_node(node: &LintNode) -> Self {
        Self {
            allow: {
                let configured = rule_allow_list_names(node, "allow");
                if node
                    .fields
                    .get("__ruleOptions")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|options| options.first())
                    .and_then(|options| options.get("allow"))
                    .is_some()
                {
                    configured
                } else {
                    // Upstream default allow list.
                    ["Error", "URL", "URLSearchParams"]
                        .map(String::from)
                        .to_vec()
                }
            },
            allow_any: rule_option_bool(node, "allowAny").unwrap_or(true),
            allow_array: rule_option_bool(node, "allowArray").unwrap_or(false),
            allow_boolean: rule_option_bool(node, "allowBoolean").unwrap_or(true),
            allow_never: rule_option_bool(node, "allowNever").unwrap_or(false),
            allow_nullish: rule_option_bool(node, "allowNullish").unwrap_or(true),
            allow_number: rule_option_bool(node, "allowNumber").unwrap_or(true),
            allow_regexp: rule_option_bool(node, "allowRegExp").unwrap_or(true),
        }
    }
}

fn is_allowed_expression(
    expression: &LintNode,
    options: &RestrictTemplateExpressionsOptions,
) -> bool {
    if type_texts_match_names(&expression.type_texts, &options.allow) {
        return true;
    }
    if matches!(
        expression.kind.as_str(),
        "ObjectExpression" | "FunctionExpression" | "ArrowFunctionExpression"
    ) {
        return false;
    }
    if expression.kind == "ArrayExpression" {
        return options.allow_array;
    }

    !expression.type_texts.is_empty()
        && expression
            .type_texts
            .iter()
            .all(|text| is_allowed_type_text(text, options))
}

fn is_allowed_type_text(text: &str, options: &RestrictTemplateExpressionsOptions) -> bool {
    split_top_level_type_text(text, '|')
        .iter()
        .all(|part| is_allowed_intersection_type_text(part, options))
}

fn is_allowed_intersection_type_text(
    text: &str,
    options: &RestrictTemplateExpressionsOptions,
) -> bool {
    split_top_level_type_text(text, '&')
        .iter()
        .any(|part| is_allowed_atom_type_text(part, options))
}

fn is_allowed_atom_type_text(text: &str, options: &RestrictTemplateExpressionsOptions) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }
    if options.allow_array && is_array_like_type_texts(&[text]) {
        return true;
    }

    match classify_type_text(Some(text)) {
        TypeTextKind::String => true,
        TypeTextKind::Any => options.allow_any,
        TypeTextKind::Boolean => options.allow_boolean,
        TypeTextKind::Bigint | TypeTextKind::Number => options.allow_number,
        TypeTextKind::Nullish => options.allow_nullish,
        TypeTextKind::Regexp => options.allow_regexp,
        TypeTextKind::Unknown if text == "never" => options.allow_never,
        // Named types are only permitted through the `allow` option (whose
        // default carries the upstream Error/URL/URLSearchParams list) plus
        // the Error-suffix convention shared with the Error-likeness helpers.
        TypeTextKind::Other => {
            let name = &text[..text.find('<').unwrap_or(text.len())];
            options.allow.iter().any(|allowed| allowed == name)
                || (options.allow.iter().any(|allowed| allowed == "Error")
                    && text.ends_with("Error"))
        }
        TypeTextKind::Unknown => false,
    }
}
