use serde_json::Value;

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule, TextRange};
use crate::utils::{TypeTextKind, classify_type_text, split_type_text};

/// Type-aware rule that prefers the nullish coalescing operator over logical
/// `||`/`||=`, ternaries, and `if` guards that fall back on nullish values.
#[derive(Clone, Copy, Debug, Default)]
pub struct PreferNullishCoalescingRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "noStrictNullCheck",
        description: "This rule requires the `strictNullChecks` compiler option to be turned on to function correctly.",
    },
    RuleMessage {
        id: "preferNullishOverOr",
        description: "Prefer using nullish coalescing operator (`??`) instead of a logical or (`||`), as it is a safer operator.",
    },
    RuleMessage {
        id: "preferNullishOverTernary",
        description: "Prefer using nullish coalescing operator (`??`) instead of a ternary expression, as it is simpler to read.",
    },
    RuleMessage {
        id: "preferNullishOverAssignment",
        description: "Prefer using nullish coalescing operator (`??=`) instead of an assignment expression, as it is simpler to read.",
    },
];

// ESTree node kinds.
//
// Upstream (TypeScript AST) listens to BinaryExpression / ConditionalExpression /
// IfStatement. In ESTree:
//   - `a || b`   => LogicalExpression(operator "||")
//   - `a ||= b`  => AssignmentExpression(operator "||=")
//   - `a ? b : c`=> ConditionalExpression
//   - `if (...)` => IfStatement
const LISTENERS: &[&str] = &[
    "LogicalExpression",
    "AssignmentExpression",
    "ConditionalExpression",
    "IfStatement",
];

impl RustLintRule for PreferNullishCoalescingRule {
    fn name(&self) -> &'static str {
        "prefer-nullish-coalescing"
    }

    fn docs_description(&self) -> &'static str {
        "Prefer nullish coalescing for nullish fallbacks."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        let opts = Options::from_node(node);

        // strictNullChecks gating. The host attaches `__isStrictNullChecks` (a raw
        // checker query). When it is explicitly `false`, report once and bail.
        // Upstream reports `noStrictNullCheck` at range (0,0); we mirror that.
        if node.field_bool("__isStrictNullChecks") == Some(false) {
            ctx.report("noStrictNullCheck", TextRange::new(0, 0));
            return;
        }

        match node.kind.as_str() {
            "LogicalExpression" if node.field_str("operator") == Some("||") => {
                check_prefer_over_or(ctx, node, &opts);
            }
            "AssignmentExpression" if node.field_str("operator") == Some("||=") => {
                check_prefer_over_or(ctx, node, &opts);
            }
            "ConditionalExpression" if !opts.ignore_ternary_tests => {
                check_ternary(ctx, node, &opts);
            }
            "IfStatement" if !opts.ignore_if_statements => {
                check_if_statement(ctx, node, &opts);
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct IgnorePrimitives {
    bigint: bool,
    boolean: bool,
    number: bool,
    string: bool,
}

impl IgnorePrimitives {
    const NONE: Self = Self {
        bigint: false,
        boolean: false,
        number: false,
        string: false,
    };

    const ALL: Self = Self {
        bigint: true,
        boolean: true,
        number: true,
        string: true,
    };

    fn is_any(self) -> bool {
        self.bigint || self.boolean || self.number || self.string
    }
}

#[derive(Clone, Copy, Debug)]
struct Options {
    ignore_boolean_coercion: bool,
    ignore_conditional_tests: bool,
    ignore_if_statements: bool,
    ignore_mixed_logical_expressions: bool,
    ignore_ternary_tests: bool,
    ignore_primitives: IgnorePrimitives,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        let obj = node
            .fields
            .get("__ruleOptions")
            .and_then(Value::as_array)
            .and_then(|items| items.first());

        let bool_opt = |key: &str, default: bool| -> bool {
            obj.and_then(|obj| obj.get(key))
                .and_then(Value::as_bool)
                .unwrap_or(default)
        };

        let ignore_primitives = match obj.and_then(|obj| obj.get("ignorePrimitives")) {
            Some(Value::Bool(true)) => IgnorePrimitives::ALL,
            Some(Value::Object(map)) => IgnorePrimitives {
                bigint: map.get("bigint").and_then(Value::as_bool).unwrap_or(false),
                boolean: map.get("boolean").and_then(Value::as_bool).unwrap_or(false),
                number: map.get("number").and_then(Value::as_bool).unwrap_or(false),
                string: map.get("string").and_then(Value::as_bool).unwrap_or(false),
            },
            _ => IgnorePrimitives::NONE,
        };

        Self {
            ignore_boolean_coercion: bool_opt("ignoreBooleanCoercion", false),
            // Note: upstream default for ignoreConditionalTests is `true`.
            ignore_conditional_tests: bool_opt("ignoreConditionalTests", true),
            ignore_if_statements: bool_opt("ignoreIfStatements", false),
            ignore_mixed_logical_expressions: bool_opt("ignoreMixedLogicalExpressions", false),
            ignore_ternary_tests: bool_opt("ignoreTernaryTests", false),
            ignore_primitives,
        }
    }
}

/// Strip ESTree wrappers we want to look through (parentheses do not appear as
/// nodes in ESTree, but ChainExpression and TSNonNullExpression do).
fn skip_wrappers(mut node: &LintNode) -> &LintNode {
    loop {
        match node.kind.as_str() {
            "ChainExpression" => {
                if let Some(expression) = node.child("expression") {
                    node = expression;
                    continue;
                }
            }
            "TSNonNullExpression" => {
                if let Some(expression) = node.child("expression") {
                    node = expression;
                    continue;
                }
            }
            _ => {}
        }
        return node;
    }
}

fn is_member_access_like(node: &LintNode) -> bool {
    let node = skip_wrappers(node);
    matches!(node.kind.as_str(), "Identifier" | "MemberExpression")
}

/// `null` literal in ESTree is a `Literal` with `value: null` and `raw: "null"`.
fn is_null_literal(node: &LintNode) -> bool {
    let node = skip_wrappers(node);
    node.kind == "Literal"
        && node.field_str("raw") == Some("null")
        && node.fields.get("value").is_some_and(Value::is_null)
}

fn is_undefined_identifier(node: &LintNode) -> bool {
    let node = skip_wrappers(node);
    node.kind == "Identifier" && node.field_stringish("name").as_deref() == Some("undefined")
}

fn is_null_or_undefined(node: &LintNode) -> bool {
    is_null_literal(node) || is_undefined_identifier(node)
}

/// Two member-access expressions refer to the same access sequence. Mirrors
/// upstream `areNodesSimilarMemberAccess`/`isNodeEqual` over the facts we have.
fn are_nodes_similar(a: &LintNode, b: &LintNode) -> bool {
    let a = skip_wrappers(a);
    let b = skip_wrappers(b);
    if a.kind != b.kind {
        return false;
    }
    match a.kind.as_str() {
        "Identifier" => a.field_stringish("name") == b.field_stringish("name"),
        "ThisExpression" => true,
        "MemberExpression" => {
            if a.field_bool("computed").unwrap_or(false)
                != b.field_bool("computed").unwrap_or(false)
            {
                return false;
            }
            let (Some(a_obj), Some(b_obj)) = (a.child("object"), b.child("object")) else {
                return false;
            };
            if !are_nodes_similar(a_obj, b_obj) {
                return false;
            }
            match (a.child("property"), b.child("property")) {
                (Some(a_prop), Some(b_prop)) => are_nodes_similar(a_prop, b_prop),
                _ => false,
            }
        }
        "Literal" => a.field_str("raw") == b.field_str("raw"),
        _ => false,
    }
}

/// A type is nullable when it includes `null` / `undefined`, or is `any`/`unknown`
/// (which could be null/undefined). Works over rendered type texts.
fn is_nullable_type(node: &LintNode) -> bool {
    if node.type_texts.is_empty() {
        return false;
    }
    node.type_texts.iter().any(|text| {
        let trimmed = text.trim();
        if trimmed == "any" || trimmed == "unknown" {
            return true;
        }
        split_type_text(trimmed)
            .iter()
            .any(|part| matches!(part.trim(), "null" | "undefined"))
    })
}

fn is_any_or_unknown(node: &LintNode) -> bool {
    node.type_texts
        .iter()
        .any(|text| matches!(text.trim(), "any" | "unknown"))
}

/// Whether `node`'s tested-for-truthiness type is eligible for conversion,
/// honoring `ignorePrimitives`.
fn is_type_eligible(node: &LintNode, opts: &Options) -> bool {
    if !is_nullable_type(node) {
        return false;
    }
    if !opts.ignore_primitives.is_any() {
        return true;
    }
    // For any/unknown we cannot make assumptions; upstream returns false here when
    // any ignorable flag is set.
    if is_any_or_unknown(node) {
        return false;
    }
    // If any union constituent matches an ignorable primitive flag, skip.
    for text in &node.type_texts {
        for part in split_type_text(text.trim()) {
            let kind = classify_type_text(Some(part.trim()));
            let ignorable = match kind {
                TypeTextKind::Bigint => opts.ignore_primitives.bigint,
                TypeTextKind::Boolean => opts.ignore_primitives.boolean,
                TypeTextKind::Number => opts.ignore_primitives.number,
                TypeTextKind::String => opts.ignore_primitives.string,
                _ => false,
            };
            if ignorable {
                return false;
            }
        }
    }
    true
}

/// Whether a control-flow construct's truthiness test is eligible, factoring in
/// `ignoreConditionalTests` and `ignoreBooleanCoercion`.
fn is_truthiness_eligible(node: &LintNode, test_node: &LintNode, opts: &Options) -> bool {
    if !is_type_eligible(test_node, opts) {
        return false;
    }
    if opts.ignore_conditional_tests && is_conditional_test(node) {
        return false;
    }
    if opts.ignore_boolean_coercion && is_boolean_constructor_context(node) {
        // For conditional expressions inside Boolean calls, still check.
        let is_cond_in_call = node.kind == "ConditionalExpression"
            && node.field_str("__parentKind") == Some("CallExpression");
        if !is_cond_in_call {
            return false;
        }
    }
    true
}

/// Approximation of upstream `isConditionalTest`. Upstream walks up the parent
/// chain through logical/conditional/comma/`!` nodes until it reaches a
/// conditional position. We approximate using the host `__parentKind` fact for
/// the immediate parent, which covers the common cases (the option defaults to
/// `true`, so simple `a || b` at statement position is NOT a conditional test).
fn is_conditional_test(node: &LintNode) -> bool {
    matches!(
        node.field_str("__parentKind"),
        Some(
            "IfStatement"
                | "ConditionalExpression"
                | "WhileStatement"
                | "DoWhileStatement"
                | "ForStatement"
        )
    )
}

/// Approximation of upstream `isBooleanConstructorContext` using `__parentKind`.
fn is_boolean_constructor_context(node: &LintNode) -> bool {
    node.field_str("__parentKind") == Some("CallExpression")
        && node.field_bool("__inBooleanConstructorCall") == Some(true)
}

/// Whether the logical expression participates in a mixed `&&`/`||` expression.
fn is_mixed_logical_expression(node: &LintNode) -> bool {
    // Parent or either operand is a `&&` logical expression.
    if node.field_str("__parentKind") == Some("LogicalExpression")
        && node.field_str("__parentLogicalOperator") == Some("&&")
    {
        return true;
    }
    for key in ["left", "right"] {
        if let Some(child) = node.child(key) {
            let child = skip_wrappers(child);
            if child.kind == "LogicalExpression" && child.field_str("operator") == Some("&&") {
                return true;
            }
        }
    }
    false
}

fn check_prefer_over_or(ctx: &mut RuleContext<'_>, node: &LintNode, opts: &Options) {
    let Some(left) = node.child("left") else {
        return;
    };
    if !is_truthiness_eligible(node, left, opts) {
        return;
    }
    if opts.ignore_mixed_logical_expressions && is_mixed_logical_expression(node) {
        return;
    }
    ctx.report("preferNullishOverOr", node.range);
}

/// Operator extracted from a ternary/if test expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NullishOp {
    Empty,
    Not,
    Equal,
    StrictEqual,
    NotEqual,
    NotStrictEqual,
}

/// Analyze a conditional/if test, returning the operator and the operand nodes
/// inside the test (empty for a plain truthiness/negation check).
fn operator_and_test_nodes(test: &LintNode) -> Option<(NullishOp, Vec<&LintNode>)> {
    let test = skip_wrappers(test);

    if is_member_access_like(test) {
        return Some((NullishOp::Empty, Vec::new()));
    }
    if test.kind == "UnaryExpression" && test.field_str("operator") == Some("!") {
        let arg = test.child("argument").map(skip_wrappers)?;
        if is_member_access_like(arg) {
            return Some((NullishOp::Not, Vec::new()));
        }
        return None;
    }
    if test.kind == "BinaryExpression" {
        let left = test.child("left").map(skip_wrappers)?;
        let right = test.child("right").map(skip_wrappers)?;
        let op = match test.field_str("operator")? {
            "==" => NullishOp::Equal,
            "!=" => NullishOp::NotEqual,
            "===" => NullishOp::StrictEqual,
            "!==" => NullishOp::NotStrictEqual,
            _ => return None,
        };
        return Some((op, vec![left, right]));
    }
    // Compound checks `(a === null || a === undefined)` map to LogicalExpression
    // in ESTree.
    if test.kind == "LogicalExpression" {
        let logical_op = test.field_str("operator")?;
        let left = test.child("left").map(skip_wrappers)?;
        let right = test.child("right").map(skip_wrappers)?;
        if left.kind != "BinaryExpression" || right.kind != "BinaryExpression" {
            return None;
        }
        let (ll, lr) = (
            left.child("left").map(skip_wrappers)?,
            left.child("right").map(skip_wrappers)?,
        );
        let (rl, rr) = (
            right.child("left").map(skip_wrappers)?,
            right.child("right").map(skip_wrappers)?,
        );
        let left_is_nullish_cmp = is_null_or_undefined(ll) && is_null_or_undefined(lr);
        let right_is_nullish_cmp = is_null_or_undefined(rl) && is_null_or_undefined(rr);
        if left_is_nullish_cmp || right_is_nullish_cmp {
            return None;
        }
        let nodes = vec![ll, lr, rl, rr];
        let lop = left.field_str("operator")?;
        let rop = right.field_str("operator")?;
        if logical_op == "||" {
            if lop == "===" && rop == "===" {
                return Some((NullishOp::StrictEqual, nodes));
            }
            if matches!(lop, "===" | "==") && matches!(rop, "===" | "==") {
                return Some((NullishOp::Equal, nodes));
            }
        } else if logical_op == "&&" {
            if lop == "!==" && rop == "!==" {
                return Some((NullishOp::NotStrictEqual, nodes));
            }
            if matches!(lop, "!==" | "!=") && matches!(rop, "!==" | "!=") {
                return Some((NullishOp::NotEqual, nodes));
            }
        }
    }
    None
}

/// Returns `(non_nullish_branch, nullish_branch)` for a ConditionalExpression.
fn branch_nodes(node: &LintNode, operator: NullishOp) -> Option<(&LintNode, &LintNode)> {
    let when_true = node.child("consequent")?;
    let when_false = node.child("alternate")?;
    if matches!(
        operator,
        NullishOp::Empty | NullishOp::NotEqual | NullishOp::NotStrictEqual
    ) {
        Some((when_true, when_false))
    } else {
        Some((when_false, when_true))
    }
}

/// Mirrors `shouldSkipDueToPartialNullishCheck`. Requires the type of `target`.
fn should_skip_partial_nullish_check(
    has_null_check: bool,
    has_undefined_check: bool,
    operator: NullishOp,
    target: &LintNode,
    allow_not_equal: bool,
) -> bool {
    if has_null_check && has_undefined_check {
        return false;
    }
    if !has_null_check && !has_undefined_check {
        return true;
    }
    if operator == NullishOp::Equal {
        return false;
    }
    if allow_not_equal && operator == NullishOp::NotEqual {
        return false;
    }

    // Need the type of the target. any/unknown => skip.
    if is_any_or_unknown(target) {
        return true;
    }
    let mut has_null_type = false;
    let mut has_undefined_type = false;
    for text in &target.type_texts {
        for part in split_type_text(text.trim()) {
            match part.trim() {
                "null" => has_null_type = true,
                "undefined" => has_undefined_type = true,
                _ => {}
            }
        }
    }
    if has_undefined_check && has_null_type {
        return true;
    }
    if has_null_check && has_undefined_type {
        return true;
    }
    false
}

fn check_ternary(ctx: &mut RuleContext<'_>, node: &LintNode, opts: &Options) {
    let Some(condition) = node.child("test") else {
        return;
    };
    let Some((operator, nodes_inside_test)) = operator_and_test_nodes(condition) else {
        return;
    };
    let Some((non_nullish_branch, _nullish_branch)) = branch_nodes(node, operator) else {
        return;
    };
    let non_nullish = skip_wrappers(non_nullish_branch);

    let mut has_truthiness_check = false;
    let nullish_left: &LintNode;

    if nodes_inside_test.is_empty() {
        has_truthiness_check = true;
        let test_node = skip_wrappers(condition);
        let candidate = if test_node.kind == "UnaryExpression"
            && test_node.field_str("operator") == Some("!")
        {
            match test_node.child("argument") {
                Some(arg) => skip_wrappers(arg),
                None => return,
            }
        } else {
            test_node
        };
        if !are_nodes_similar(candidate, non_nullish) {
            return;
        }
        nullish_left = candidate;
    } else {
        let mut has_null_check = false;
        let mut has_undefined_check = false;
        let mut found: Option<&LintNode> = None;
        for test_node in &nodes_inside_test {
            if is_null_literal(test_node) {
                has_null_check = true;
            } else if is_undefined_identifier(test_node) {
                has_undefined_check = true;
            } else if are_nodes_similar(test_node, non_nullish) {
                if found.is_none() {
                    found = Some(test_node);
                }
            } else {
                return;
            }
        }
        let Some(found) = found else {
            return;
        };
        nullish_left = found;
        if should_skip_partial_nullish_check(
            has_null_check,
            has_undefined_check,
            operator,
            nullish_left,
            true,
        ) {
            return;
        }
    }

    if has_truthiness_check && !is_truthiness_eligible(node, nullish_left, opts) {
        return;
    }

    ctx.report("preferNullishOverTernary", node.range);
}

fn check_if_statement(ctx: &mut RuleContext<'_>, node: &LintNode, opts: &Options) {
    // `if` with an else branch is not simplifiable.
    if node.child("alternate").is_some() {
        return;
    }

    // Extract the single assignment expression from the consequent.
    let consequent = node.child("consequent");
    let assignment_expr = match consequent.map(|c| c.kind.as_str()) {
        Some("BlockStatement") => {
            let block = consequent.unwrap();
            let body = block.child_list("body").unwrap_or(&[]);
            if body.len() != 1 {
                return;
            }
            let stmt = &body[0];
            if stmt.kind != "ExpressionStatement" {
                return;
            }
            stmt.child("expression")
        }
        Some("ExpressionStatement") => consequent.unwrap().child("expression"),
        _ => return,
    };
    let Some(assignment_expr) = assignment_expr.map(skip_wrappers) else {
        return;
    };
    if assignment_expr.kind != "AssignmentExpression"
        || assignment_expr.field_str("operator") != Some("=")
    {
        return;
    }
    let Some(left) = assignment_expr.child("left") else {
        return;
    };
    if !is_member_access_like(left) {
        return;
    }

    let Some(test) = node.child("test") else {
        return;
    };
    let Some((operator, nodes_inside_test)) = operator_and_test_nodes(test) else {
        return;
    };
    // Only negation or equality checks.
    if !matches!(
        operator,
        NullishOp::Not | NullishOp::Equal | NullishOp::StrictEqual
    ) {
        return;
    }

    if nodes_inside_test.is_empty() {
        // Simple negation check `if (!foo)`.
        let mut test_node = skip_wrappers(test);
        if test_node.kind == "UnaryExpression" && test_node.field_str("operator") == Some("!") {
            match test_node.child("argument") {
                Some(arg) => test_node = skip_wrappers(arg),
                None => return,
            }
        }
        if !are_nodes_similar(test_node, left) {
            return;
        }
        if !is_truthiness_eligible(node, test_node, opts) {
            return;
        }
    } else {
        let mut has_null_check = false;
        let mut has_undefined_check = false;
        let mut found_matching = false;
        for test_node in &nodes_inside_test {
            if is_null_literal(test_node) {
                has_null_check = true;
            } else if is_undefined_identifier(test_node) {
                has_undefined_check = true;
            } else if are_nodes_similar(test_node, left) {
                found_matching = true;
            } else {
                return;
            }
        }
        if !found_matching {
            return;
        }
        if should_skip_partial_nullish_check(
            has_null_check,
            has_undefined_check,
            operator,
            left,
            false,
        ) {
            return;
        }
    }

    ctx.report("preferNullishOverAssignment", node.range);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::{Value, json};

    use super::super::super::{LintNode, RuleContext, RustLintRule, TextRange};
    use super::PreferNullishCoalescingRule;

    fn node(kind: &str, range: TextRange) -> LintNode {
        LintNode {
            kind: kind.to_owned(),
            range,
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields: BTreeMap::new(),
            children: BTreeMap::new(),
            child_lists: BTreeMap::new(),
        }
    }

    fn run(node: &LintNode) -> Vec<String> {
        let rule = PreferNullishCoalescingRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish().into_iter().map(|d| d.message_id).collect()
    }

    fn ident(name: &str, type_texts: &[&str]) -> LintNode {
        let mut n = node("Identifier", TextRange::new(0, 1));
        n.fields.insert("name".to_owned(), json!(name));
        n.type_texts = type_texts.iter().map(|t| (*t).to_owned()).collect();
        n
    }

    fn null_literal() -> LintNode {
        let mut n = node("Literal", TextRange::new(0, 4));
        n.fields.insert("raw".to_owned(), json!("null"));
        n.fields.insert("value".to_owned(), Value::Null);
        n
    }

    // ---- preferNullishOverOr ----

    #[test]
    fn reports_logical_or_on_nullable_left() {
        // `x || y` where x: string | undefined
        let mut n = node("LogicalExpression", TextRange::new(0, 6));
        n.fields.insert("operator".to_owned(), json!("||"));
        n.children
            .insert("left".to_owned(), ident("x", &["string | undefined"]));
        n.children.insert("right".to_owned(), ident("y", &[]));
        assert_eq!(run(&n), vec!["preferNullishOverOr".to_owned()]);
    }

    #[test]
    fn does_not_report_logical_or_on_non_nullable_left() {
        // `x || y` where x: string  -> not nullable
        let mut n = node("LogicalExpression", TextRange::new(0, 6));
        n.fields.insert("operator".to_owned(), json!("||"));
        n.children
            .insert("left".to_owned(), ident("x", &["string"]));
        n.children.insert("right".to_owned(), ident("y", &[]));
        assert!(run(&n).is_empty());
    }

    #[test]
    fn respects_ignore_primitives_string() {
        // `x || y` where x: string | undefined and ignorePrimitives.string => skip
        let mut n = node("LogicalExpression", TextRange::new(0, 6));
        n.fields.insert("operator".to_owned(), json!("||"));
        n.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "ignorePrimitives": { "string": true } }]),
        );
        n.children
            .insert("left".to_owned(), ident("x", &["string | undefined"]));
        n.children.insert("right".to_owned(), ident("y", &[]));
        assert!(run(&n).is_empty());
    }

    // ---- preferNullishOverTernary ----

    #[test]
    fn reports_ternary_not_undefined_strict() {
        // `this != undefined ? this : y` -> simplified detection with `this`
        // Build: test = (this != undefined), consequent=this, alternate=y
        let mut this_a = node("ThisExpression", TextRange::new(0, 4));
        this_a.type_texts = vec!["{ a: string } | undefined".to_owned()];
        let mut this_b = node("ThisExpression", TextRange::new(20, 24));
        this_b.type_texts = vec!["{ a: string } | undefined".to_owned()];

        let mut undefined_id = node("Identifier", TextRange::new(8, 17));
        undefined_id
            .fields
            .insert("name".to_owned(), json!("undefined"));

        let mut test = node("BinaryExpression", TextRange::new(0, 17));
        test.fields.insert("operator".to_owned(), json!("!="));
        test.children.insert("left".to_owned(), this_a);
        test.children.insert("right".to_owned(), undefined_id);

        let mut n = node("ConditionalExpression", TextRange::new(0, 29));
        n.children.insert("test".to_owned(), test);
        n.children.insert("consequent".to_owned(), this_b);
        n.children.insert("alternate".to_owned(), ident("y", &[]));
        assert_eq!(run(&n), vec!["preferNullishOverTernary".to_owned()]);
    }

    #[test]
    fn does_not_report_ternary_when_ignore_ternary_tests() {
        // Same shape but ignoreTernaryTests: true
        let this_a = {
            let mut t = node("ThisExpression", TextRange::new(0, 4));
            t.type_texts = vec!["{ a: string } | undefined".to_owned()];
            t
        };
        let this_b = {
            let mut t = node("ThisExpression", TextRange::new(20, 24));
            t.type_texts = vec!["{ a: string } | undefined".to_owned()];
            t
        };
        let mut undefined_id = node("Identifier", TextRange::new(8, 17));
        undefined_id
            .fields
            .insert("name".to_owned(), json!("undefined"));
        let mut test = node("BinaryExpression", TextRange::new(0, 17));
        test.fields.insert("operator".to_owned(), json!("!="));
        test.children.insert("left".to_owned(), this_a);
        test.children.insert("right".to_owned(), undefined_id);

        let mut n = node("ConditionalExpression", TextRange::new(0, 29));
        n.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "ignoreTernaryTests": true }]),
        );
        n.children.insert("test".to_owned(), test);
        n.children.insert("consequent".to_owned(), this_b);
        n.children.insert("alternate".to_owned(), ident("y", &[]));
        assert!(run(&n).is_empty());
    }

    #[test]
    fn does_not_report_ternary_partial_null_check_with_undefined_type() {
        // `x === null ? x : y` where x: string | undefined.
        // hasNullCheck only, type includes undefined => skip.
        let mut x_left = ident("x", &["string | undefined"]);
        x_left.range = TextRange::new(0, 1);
        let mut x_cons = ident("x", &["string | undefined"]);
        x_cons.range = TextRange::new(15, 16);

        let mut test = node("BinaryExpression", TextRange::new(0, 11));
        test.fields.insert("operator".to_owned(), json!("==="));
        test.children.insert("left".to_owned(), x_left);
        test.children.insert("right".to_owned(), null_literal());

        let mut n = node("ConditionalExpression", TextRange::new(0, 20));
        n.children.insert("test".to_owned(), test);
        n.children.insert("consequent".to_owned(), x_cons);
        n.children.insert("alternate".to_owned(), ident("y", &[]));
        assert!(run(&n).is_empty());
    }

    // ---- noStrictNullCheck ----

    #[test]
    fn reports_no_strict_null_check() {
        let mut n = node("IfStatement", TextRange::new(0, 10));
        n.fields
            .insert("__isStrictNullChecks".to_owned(), json!(false));
        assert_eq!(run(&n), vec!["noStrictNullCheck".to_owned()]);
    }

    // ---- preferNullishOverAssignment ----

    #[test]
    fn reports_if_statement_assignment() {
        // if (!foo) foo = makeFoo();   where foo: { a: string } | null
        let mut foo_test = ident("foo", &["{ a: string } | null"]);
        foo_test.range = TextRange::new(5, 8);
        let mut not_test = node("UnaryExpression", TextRange::new(4, 8));
        not_test.fields.insert("operator".to_owned(), json!("!"));
        not_test.children.insert("argument".to_owned(), foo_test);

        let mut foo_left = ident("foo", &["{ a: string } | null"]);
        foo_left.range = TextRange::new(10, 13);
        let mut assign = node("AssignmentExpression", TextRange::new(10, 22));
        assign.fields.insert("operator".to_owned(), json!("="));
        assign.children.insert("left".to_owned(), foo_left);
        assign.children.insert("right".to_owned(), ident("z", &[]));

        let mut expr_stmt = node("ExpressionStatement", TextRange::new(10, 23));
        expr_stmt.children.insert("expression".to_owned(), assign);

        let mut n = node("IfStatement", TextRange::new(0, 23));
        n.children.insert("test".to_owned(), not_test);
        n.children.insert("consequent".to_owned(), expr_stmt);
        assert_eq!(run(&n), vec!["preferNullishOverAssignment".to_owned()]);
    }

    #[test]
    fn does_not_report_if_statement_when_ignore_if_statements() {
        let mut foo_test = ident("foo", &["{ a: string } | null"]);
        foo_test.range = TextRange::new(5, 8);
        let mut not_test = node("UnaryExpression", TextRange::new(4, 8));
        not_test.fields.insert("operator".to_owned(), json!("!"));
        not_test.children.insert("argument".to_owned(), foo_test);

        let mut foo_left = ident("foo", &["{ a: string } | null"]);
        foo_left.range = TextRange::new(10, 13);
        let mut assign = node("AssignmentExpression", TextRange::new(10, 22));
        assign.fields.insert("operator".to_owned(), json!("="));
        assign.children.insert("left".to_owned(), foo_left);
        assign.children.insert("right".to_owned(), ident("z", &[]));
        let mut expr_stmt = node("ExpressionStatement", TextRange::new(10, 23));
        expr_stmt.children.insert("expression".to_owned(), assign);

        let mut n = node("IfStatement", TextRange::new(0, 23));
        n.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "ignoreIfStatements": true }]),
        );
        n.children.insert("test".to_owned(), not_test);
        n.children.insert("consequent".to_owned(), expr_stmt);
        assert!(run(&n).is_empty());
    }
}
