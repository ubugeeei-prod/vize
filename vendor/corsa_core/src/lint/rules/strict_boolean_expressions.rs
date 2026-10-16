use serde_json::Value;

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule, TextRange};
use crate::lint::helpers::rule_option_bool;
use crate::utils::{TypeTextKind, classify_type_text, split_type_text};

/// Type-aware rule that disallows non-boolean values in boolean expression
/// positions (`if`, `while`, `do`, `for`, `?:`, `&&`/`||`, and `!`).
///
/// This is a faithful port of the tsgolint `strict-boolean-expressions` rule.
/// The full decision logic (the Go `analyzeTypeParts` / `checkCondition`
/// functions) lives here in Rust and operates on per-union-member type facts.
#[derive(Clone, Copy, Debug, Default)]
pub struct StrictBooleanExpressionsRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "conditionErrorNumber",
        description: "Unexpected number value in conditional. A number can be falsy (0, NaN) or truthy.",
    },
    RuleMessage {
        id: "conditionErrorString",
        description: "Unexpected string value in conditional. A string can be falsy (empty string) or truthy.",
    },
    RuleMessage {
        id: "conditionErrorObject",
        description: "Unexpected object value in conditional. An object is always truthy.",
    },
    RuleMessage {
        id: "conditionErrorNullish",
        description: "Unexpected nullish value in conditional. The expression is always falsy.",
    },
    RuleMessage {
        id: "conditionErrorOther",
        description: "Unexpected value in conditional. A union of different types has inconsistent truthiness.",
    },
    RuleMessage {
        id: "conditionErrorNullableBoolean",
        description: "Unexpected nullable boolean value in conditional. Please handle the nullish case explicitly.",
    },
    RuleMessage {
        id: "conditionErrorNullableObject",
        description: "Unexpected nullable object value in conditional. Please handle the nullish case explicitly.",
    },
    RuleMessage {
        id: "conditionErrorNullableString",
        description: "Unexpected nullable string value in conditional. Please handle the nullish and empty string cases explicitly.",
    },
    RuleMessage {
        id: "conditionErrorNullableNumber",
        description: "Unexpected nullable number value in conditional. Please handle the nullish and zero cases explicitly.",
    },
    RuleMessage {
        id: "conditionErrorNullableEnum",
        description: "Unexpected nullable enum value in conditional. Please handle the nullish and falsy enum cases explicitly.",
    },
    RuleMessage {
        id: "conditionErrorAny",
        description: "Unexpected any value in conditional. Use a more specific type to ensure type safety.",
    },
    RuleMessage {
        id: "noStrictNullCheck",
        description: "This rule requires the `strictNullChecks` compiler option to be turned on to function correctly.",
    },
    RuleMessage {
        id: "predicateCannotBeAsync",
        description: "Predicate function should not be 'async'; expected a boolean return type.",
    },
];

const LISTENERS: &[&str] = &[
    "IfStatement",
    "WhileStatement",
    "DoWhileStatement",
    "ForStatement",
    "ConditionalExpression",
    "LogicalExpression",
    "UnaryExpression",
    "CallExpression",
];

impl RustLintRule for StrictBooleanExpressionsRule {
    fn name(&self) -> &'static str {
        "strict-boolean-expressions"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow non-boolean values in boolean expression positions."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        let opts = Options::from_node(node);

        // strictNullChecks gate: the host sets `__strictNullChecksDisabled`
        // (a raw compiler-option fact) on the listened node when the program is
        // compiled without strictNullChecks. Mirror Go's single report at range
        // (0, 0). We only emit it from a single deterministic listener kind to
        // avoid duplicate reports across the many listeners.
        if node.kind == "IfStatement" && node.field_bool("__strictNullChecksDisabled") == Some(true)
        {
            ctx.report("noStrictNullCheck", TextRange::new(0, 0));
        }

        match node.kind.as_str() {
            "IfStatement" | "WhileStatement" | "DoWhileStatement" => {
                if let Some(test) = node.child("test").or_else(|| node.child("expression")) {
                    traverse_node(ctx, test, &opts, true);
                }
            }
            "ForStatement" => {
                if let Some(test) = node.child("test") {
                    traverse_node(ctx, test, &opts, true);
                }
            }
            "ConditionalExpression" => {
                if let Some(test) = node.child("test") {
                    traverse_node(ctx, test, &opts, true);
                }
            }
            "LogicalExpression"
                // The `??` operator is not a boolean position.
                if node.field_str("operator") != Some("??") => {
                    traverse_logical_expression(ctx, node, &opts, false);
                }
            "UnaryExpression"
                if node.field_str("operator") == Some("!") => {
                    if let Some(argument) = node.child("argument") {
                        traverse_node(ctx, argument, &opts, true);
                    }
                }
            "CallExpression" => {
                check_call_expression(ctx, node, &opts);
            }
            _ => {}
        }
    }
}

/// Resolved option set. Note the upstream defaults: `allowString`,
/// `allowNumber`, and `allowNullableObject` default to `true`; everything else
/// defaults to `false`.
#[derive(Clone, Copy, Debug)]
struct Options {
    allow_any: bool,
    allow_nullable_boolean: bool,
    allow_nullable_enum: bool,
    allow_nullable_number: bool,
    allow_nullable_object: bool,
    allow_nullable_string: bool,
    allow_number: bool,
    allow_string: bool,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        Self {
            allow_any: rule_option_bool(node, "allowAny").unwrap_or(false),
            allow_nullable_boolean: rule_option_bool(node, "allowNullableBoolean").unwrap_or(false),
            allow_nullable_enum: rule_option_bool(node, "allowNullableEnum").unwrap_or(false),
            allow_nullable_number: rule_option_bool(node, "allowNullableNumber").unwrap_or(false),
            allow_nullable_object: rule_option_bool(node, "allowNullableObject").unwrap_or(true),
            allow_nullable_string: rule_option_bool(node, "allowNullableString").unwrap_or(false),
            allow_number: rule_option_bool(node, "allowNumber").unwrap_or(true),
            allow_string: rule_option_bool(node, "allowString").unwrap_or(true),
        }
    }
}

fn traverse_logical_expression(
    ctx: &mut RuleContext<'_>,
    bin_expr: &LintNode,
    opts: &Options,
    is_condition: bool,
) {
    if let Some(left) = bin_expr.child("left") {
        traverse_node(ctx, left, opts, true);
    }
    if let Some(right) = bin_expr.child("right") {
        traverse_node(ctx, right, opts, is_condition);
    }
}

fn traverse_node(ctx: &mut RuleContext<'_>, node: &LintNode, opts: &Options, is_condition: bool) {
    // Unwrap parenthesized expressions (the TS-ESTree parser usually elides
    // these, but handle the explicit-node case defensively).
    if node.kind == "ParenthesizedExpression" || node.kind == "TSParenthesizedType" {
        if let Some(expression) = node.child("expression") {
            traverse_node(ctx, expression, opts, is_condition);
        }
        return;
    }

    if node.kind == "LogicalExpression" && node.field_str("operator") != Some("??") {
        traverse_logical_expression(ctx, node, opts, is_condition);
        return;
    }

    if !is_condition {
        return;
    }

    check_node(ctx, node, opts);
}

fn check_node(ctx: &mut RuleContext<'_>, node: &LintNode, opts: &Options) {
    let info = analyze_type_info(node);
    check_condition(ctx, node, &info, opts);
}

/// Handles `CallExpression` cases: truthiness-asserted arguments and array
/// predicate methods. Both are checker-dependent, so we act only when the host
/// supplies the relevant raw facts.
fn check_call_expression(ctx: &mut RuleContext<'_>, node: &LintNode, opts: &Options) {
    // `findTruthinessAssertedArgument`: the host computes the index of the
    // argument asserted truthy via an `asserts x` predicate signature.
    if let Some(index) = node.field_f64("__truthinessAssertedArgumentIndex") {
        let index = index as usize;
        if let Some(args) = node.child_list("arguments") {
            if let Some(arg) = args.get(index) {
                traverse_node(ctx, arg, opts, true);
            }
        }
    }

    // `IsArrayMethodCallWithPredicate`: the host marks calls like
    // `[].some(fn)` / `.filter(fn)` etc. whose first argument is a predicate.
    if node.field_bool("__arrayMethodPredicateCall") == Some(true) {
        if let Some(args) = node.child_list("arguments") {
            if let Some(arg) = args.first() {
                let is_function = matches!(
                    arg.kind.as_str(),
                    "ArrowFunctionExpression" | "FunctionExpression" | "FunctionDeclaration"
                );
                if is_function && arg.field_bool("async") == Some(true) {
                    ctx.report("predicateCannotBeAsync", arg.range);
                    return;
                }
                // The predicate's return type is analyzed exactly like any
                // other condition; the host renders it onto the argument as
                // `__predicateReturnTypeParts`.
                let info = analyze_from_parts_field(arg, "__predicateReturnTypeParts")
                    .unwrap_or_else(|| analyze_type_info(arg));
                check_condition(ctx, node, &info, opts);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Type analysis (port of Go `analyzeTypeParts` / `analyzeTypePart`)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Variant {
    Nullish,
    Boolean,
    String,
    Number,
    BigInt,
    Object,
    Any,
    Unknown,
    Never,
    Mixed,
    Generic,
}

#[derive(Clone, Debug)]
struct TypeInfo {
    variant: Variant,
    is_nullable: bool,
    is_truthy: bool,
    is_enum: bool,
}

/// Raw per-union-member descriptor as supplied by the host fact
/// `__conditionTypeParts`. Each entry mirrors the checker primitives the Go
/// rule reads off `checker.Type_flags` plus literal value inspection.
#[derive(Clone, Debug)]
struct PartInfo {
    variant: Variant,
    is_truthy: bool,
    is_enum: bool,
}

/// Analyzes the condition node's type. Prefers the raw `__conditionTypeParts`
/// host fact (faithful, checker-derived); falls back to classifying the
/// rendered `type_texts` when the fact is absent.
fn analyze_type_info(node: &LintNode) -> TypeInfo {
    analyze_from_parts_field(node, "__conditionTypeParts")
        .unwrap_or_else(|| analyze_from_type_texts(node))
}

fn analyze_from_parts_field(node: &LintNode, key: &str) -> Option<TypeInfo> {
    let parts = node.fields.get(key)?.as_array()?;
    let parts: Vec<PartInfo> = parts.iter().filter_map(part_info_from_value).collect();
    if parts.is_empty() {
        return None;
    }
    Some(analyze_type_parts(&parts))
}

fn part_info_from_value(value: &Value) -> Option<PartInfo> {
    let object = value.as_object()?;
    let variant = match object.get("variant").and_then(Value::as_str)? {
        "nullish" => Variant::Nullish,
        "boolean" => Variant::Boolean,
        "string" => Variant::String,
        "number" => Variant::Number,
        "bigint" => Variant::BigInt,
        "object" => Variant::Object,
        "any" => Variant::Any,
        "unknown" => Variant::Unknown,
        "never" => Variant::Never,
        "generic" => Variant::Generic,
        _ => Variant::Mixed,
    };
    Some(PartInfo {
        variant,
        is_truthy: object
            .get("isTruthy")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        is_enum: object
            .get("isEnum")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

/// Port of Go `analyzeTypeParts`.
fn analyze_type_parts(parts: &[PartInfo]) -> TypeInfo {
    let mut info = TypeInfo {
        variant: Variant::Mixed,
        is_nullable: false,
        is_truthy: false,
        is_enum: false,
    };

    // Distinct set of variants encountered, preserving discovery order so the
    // "two variants + nullable" case can pick the non-nullish one.
    let mut variants: Vec<Variant> = Vec::new();
    let mut met_not_truthy = false;

    for part in parts {
        if !variants.contains(&part.variant) {
            variants.push(part.variant);
        }
        if part.variant == Variant::Nullish {
            info.is_nullable = true;
        }
        if part.is_enum {
            info.is_enum = true;
        }
        if matches!(
            part.variant,
            Variant::Boolean | Variant::Number | Variant::String | Variant::BigInt
        ) {
            if met_not_truthy {
                continue;
            }
            info.is_truthy = part.is_truthy;
            met_not_truthy = !part.is_truthy;
        }
    }

    if variants.len() == 1 {
        info.variant = variants[0];
    } else if variants.len() == 2 && info.is_nullable {
        info.variant = variants
            .iter()
            .copied()
            .find(|variant| *variant != Variant::Nullish)
            .unwrap_or(Variant::Mixed);
    } else {
        info.variant = Variant::Mixed;
    }

    info
}

/// Fallback analysis derived from rendered type texts. The primary path runs
/// `analyze_from_condition_type_parts` against the host-supplied
/// `__conditionTypeParts` fact, which is sourced directly from the TSGO
/// checker. This text-based path stays as a stable secondary so unit tests
/// (and any third-party host that has not been updated to ship the full fact
/// payload) keep producing correct results for the common cases.
fn analyze_from_type_texts(node: &LintNode) -> TypeInfo {
    let mut parts: Vec<PartInfo> = Vec::new();

    // Literal AST shortcuts (boolean / numeric / string literals carry a value).
    if node.kind == "Literal" {
        if let Some(value) = node.fields.get("value") {
            if let Some(boolean) = value.as_bool() {
                parts.push(PartInfo {
                    variant: Variant::Boolean,
                    is_truthy: boolean,
                    is_enum: false,
                });
            } else if let Some(number) = value.as_f64() {
                parts.push(PartInfo {
                    variant: Variant::Number,
                    is_truthy: number != 0.0,
                    is_enum: false,
                });
            } else if let Some(text) = value.as_str() {
                parts.push(PartInfo {
                    variant: Variant::String,
                    is_truthy: !text.is_empty(),
                    is_enum: false,
                });
            }
        }
    }

    if parts.is_empty() {
        for text in &node.type_texts {
            for atom in split_type_text(text) {
                parts.push(part_from_type_text(atom.trim()));
            }
        }
    }

    if parts.is_empty() {
        // No type information at all: treat as never so we do not over-report.
        return TypeInfo {
            variant: Variant::Never,
            is_nullable: false,
            is_truthy: false,
            is_enum: false,
        };
    }

    analyze_type_parts(&parts)
}

fn part_from_type_text(atom: &str) -> PartInfo {
    let truthy_literal_boolean = atom == "true";
    let kind = classify_type_text(Some(atom));
    let variant = match kind {
        TypeTextKind::Any => Variant::Any,
        TypeTextKind::Unknown if atom == "never" => Variant::Never,
        TypeTextKind::Unknown => Variant::Unknown,
        TypeTextKind::Nullish => Variant::Nullish,
        TypeTextKind::Boolean => Variant::Boolean,
        TypeTextKind::String => Variant::String,
        TypeTextKind::Number => Variant::Number,
        TypeTextKind::Bigint => Variant::BigInt,
        TypeTextKind::Regexp => Variant::Object,
        TypeTextKind::Other => classify_other_atom(atom),
    };

    let is_truthy = match variant {
        Variant::Boolean => truthy_literal_boolean,
        Variant::String => string_literal_value(atom).is_some_and(|value| !value.is_empty()),
        Variant::Number => number_literal_value(atom).is_some_and(|value| value != 0.0),
        Variant::BigInt => bigint_literal_nonzero(atom),
        _ => false,
    };

    PartInfo {
        variant,
        is_truthy,
        is_enum: false,
    }
}

/// Coarse classification for type texts that did not match a primitive bucket.
/// Object-like shapes (`{...}`, arrays, tuples, function types, named refs) are
/// treated as objects, matching the Go `typeVariantObject` fallthrough.
fn classify_other_atom(atom: &str) -> Variant {
    let atom = atom.trim();
    if atom.is_empty() {
        return Variant::Mixed;
    }
    if atom == "void" {
        return Variant::Nullish;
    }
    if atom == "object" {
        return Variant::Object;
    }
    let looks_like_object = atom.starts_with('{')
        || atom.starts_with('[')
        || atom.starts_with('(')
        || atom.ends_with("[]")
        || atom.contains("=>")
        || atom.starts_with("Array<")
        || atom.starts_with("ReadonlyArray<")
        || atom
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_uppercase());
    if looks_like_object {
        Variant::Object
    } else {
        Variant::Mixed
    }
}

fn string_literal_value(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"')
            || (first == b'\'' && last == b'\'')
            || (first == b'`' && last == b'`')
        {
            return Some(&text[1..text.len() - 1]);
        }
    }
    None
}

fn number_literal_value(text: &str) -> Option<f64> {
    text.parse::<f64>().ok()
}

fn bigint_literal_nonzero(text: &str) -> bool {
    if let Some(stripped) = text.strip_suffix('n') {
        if let Ok(value) = stripped.parse::<i128>() {
            return value != 0;
        }
    }
    false
}

/// Port of Go `checkCondition`.
fn check_condition(ctx: &mut RuleContext<'_>, node: &LintNode, info: &TypeInfo, opts: &Options) {
    match info.variant {
        Variant::Any | Variant::Unknown | Variant::Generic => {
            if !opts.allow_any {
                ctx.report("conditionErrorAny", node.range);
            }
        }
        Variant::Never => {}
        Variant::Nullish => {
            ctx.report("conditionErrorNullish", node.range);
        }
        Variant::String => {
            if opts.allow_string && info.is_truthy {
                return;
            }
            if info.is_nullable {
                if info.is_enum {
                    if !opts.allow_nullable_enum {
                        ctx.report("conditionErrorNullableEnum", node.range);
                    }
                } else if !opts.allow_nullable_string {
                    ctx.report("conditionErrorNullableString", node.range);
                }
            } else if !opts.allow_string {
                ctx.report("conditionErrorString", node.range);
            }
        }
        Variant::Number => {
            if opts.allow_number && info.is_truthy {
                return;
            }
            if info.is_nullable {
                if info.is_enum {
                    if !opts.allow_nullable_enum {
                        ctx.report("conditionErrorNullableEnum", node.range);
                    }
                } else if !opts.allow_nullable_number {
                    ctx.report("conditionErrorNullableNumber", node.range);
                }
            } else if !opts.allow_number {
                ctx.report("conditionErrorNumber", node.range);
            }
        }
        Variant::Boolean => {
            if info.is_truthy {
                return;
            }
            if info.is_nullable && !opts.allow_nullable_boolean {
                ctx.report("conditionErrorNullableBoolean", node.range);
            }
        }
        Variant::Object => {
            if info.is_nullable && !opts.allow_nullable_object {
                ctx.report("conditionErrorNullableObject", node.range);
            } else if !info.is_nullable {
                ctx.report("conditionErrorObject", node.range);
            }
        }
        Variant::Mixed => {
            if info.is_enum {
                if info.is_nullable && !opts.allow_nullable_enum {
                    ctx.report("conditionErrorNullableEnum", node.range);
                }
                return;
            }
            ctx.report("conditionErrorOther", node.range);
        }
        Variant::BigInt => {
            if info.is_nullable && !opts.allow_nullable_number {
                ctx.report("conditionErrorNullableNumber", node.range);
            } else if !info.is_nullable && !opts.allow_number {
                ctx.report("conditionErrorNumber", node.range);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use super::StrictBooleanExpressionsRule;

    fn run(node: &LintNode) -> Vec<String> {
        let rule = StrictBooleanExpressionsRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
            .into_iter()
            .map(|diagnostic| diagnostic.message_id)
            .collect()
    }

    fn if_with_test(test: Value) -> LintNode {
        serde_json::from_value(json!({
            "kind": "IfStatement",
            "range": { "start": 0, "end": 10 },
            "children": { "test": test }
        }))
        .unwrap()
    }

    fn typed_test(type_texts: &[&str]) -> Value {
        json!({
            "kind": "Identifier",
            "range": { "start": 4, "end": 5 },
            "typeTexts": type_texts,
        })
    }

    #[test]
    fn reports_plain_object_condition() {
        // `declare const x: {}; if (x) {}` -> object is always truthy.
        let node = if_with_test(typed_test(&["{}"]));
        let ids = run(&node);
        assert_eq!(ids, vec!["conditionErrorObject"]);
    }

    #[test]
    fn reports_nullable_string_condition() {
        // `declare const x: string | null; if (x) {}` with default options.
        let node = if_with_test(typed_test(&["string | null"]));
        let ids = run(&node);
        assert_eq!(ids, vec!["conditionErrorNullableString"]);
    }

    #[test]
    fn reports_nullish_condition() {
        let node = if_with_test(typed_test(&["null"]));
        let ids = run(&node);
        assert_eq!(ids, vec!["conditionErrorNullish"]);
    }

    #[test]
    fn allows_boolean_condition() {
        // `declare const x: boolean; if (x) {}`
        let node = if_with_test(typed_test(&["boolean"]));
        let ids = run(&node);
        assert!(ids.is_empty(), "expected no diagnostics, got {ids:?}");
    }

    #[test]
    fn allows_negated_boolean() {
        // `(x: boolean) => !x;`
        let node: LintNode = serde_json::from_value(json!({
            "kind": "UnaryExpression",
            "range": { "start": 0, "end": 2 },
            "fields": { "operator": "!" },
            "children": {
                "argument": {
                    "kind": "Identifier",
                    "range": { "start": 1, "end": 2 },
                    "typeTexts": ["boolean"]
                }
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    #[test]
    fn allows_string_literal_default() {
        // `if ('') {}` -- allowString defaults true; string variant w/o nullable
        // is allowed regardless of truthiness.
        let node = if_with_test(json!({
            "kind": "Literal",
            "range": { "start": 4, "end": 6 },
            "typeTexts": ["\"\""],
            "fields": { "value": "" }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn reports_string_when_allow_string_disabled() {
        let mut node = if_with_test(typed_test(&["string"]));
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "allowString": false }]),
        );
        let ids = run(&node);
        assert_eq!(ids, vec!["conditionErrorString"]);
    }

    #[test]
    fn allows_nullable_boolean_with_option() {
        let mut node = if_with_test(typed_test(&["boolean | null"]));
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "allowNullableBoolean": true }]),
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn reports_nullable_boolean_default() {
        let node = if_with_test(typed_test(&["boolean | undefined"]));
        let ids = run(&node);
        assert_eq!(ids, vec!["conditionErrorNullableBoolean"]);
    }

    #[test]
    fn uses_condition_type_parts_fact() {
        // Host-provided raw fact: a single number variant, falsy, not enum.
        let mut node = if_with_test(json!({
            "kind": "Identifier",
            "range": { "start": 4, "end": 5 },
            "fields": {
                "__conditionTypeParts": [
                    { "variant": "number", "isTruthy": false, "isEnum": false }
                ]
            }
        }));
        // With `allowNumber: false`, the number variant carried by the raw
        // `__conditionTypeParts` fact drives a `conditionErrorNumber` report
        // (matching the Go `!opts.AllowNumber` branch). With the default
        // `allowNumber: true` a non-truthy number is permitted and nothing is
        // reported, so this also proves the option/fact wiring is honored.
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "allowNumber": false }]),
        );
        let ids = run(&node);
        assert_eq!(ids, vec!["conditionErrorNumber"]);
    }

    #[test]
    fn logical_expression_checks_left_operand() {
        // `(x: object) && y` -- left object is always truthy -> reported once.
        let node: LintNode = serde_json::from_value(json!({
            "kind": "LogicalExpression",
            "range": { "start": 0, "end": 6 },
            "fields": { "operator": "&&" },
            "children": {
                "left": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 1 },
                    "typeTexts": ["{ a: number }"]
                },
                "right": {
                    "kind": "Identifier",
                    "range": { "start": 5, "end": 6 },
                    "typeTexts": ["boolean"]
                }
            }
        }))
        .unwrap();
        let ids = run(&node);
        assert_eq!(ids, vec!["conditionErrorObject"]);
    }

    #[test]
    fn nullish_coalescing_is_not_a_condition() {
        // `a ?? b` is not a boolean position; no reports even for objects.
        let node: LintNode = serde_json::from_value(json!({
            "kind": "LogicalExpression",
            "range": { "start": 0, "end": 6 },
            "fields": { "operator": "??" },
            "children": {
                "left": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 1 },
                    "typeTexts": ["{ a: number }"]
                },
                "right": {
                    "kind": "Identifier",
                    "range": { "start": 5, "end": 6 },
                    "typeTexts": ["{ a: number }"]
                }
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }
}
