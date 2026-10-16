use serde_json::Value;

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};

/// Type-aware rule that disallows comparing an enum value against a value that
/// does not share that enum's domain.
///
/// This is a faithful port of the tsgolint Go rule
/// `no-unsafe-enum-comparison`. The comparison decision logic lives entirely in
/// Rust; the host supplies raw checker facts (enum type ids, union-part enum
/// value kinds, number/string-likeness, and union-part type ids) on each
/// operand node. The Rust code reconstructs the upstream `isMismatchedComparison`
/// predicate from those facts.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnsafeEnumComparisonRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "mismatchedCase",
        description: "The case statement does not have a shared enum type with the switch predicate.",
    },
    RuleMessage {
        id: "mismatchedCondition",
        description: "The two values in this comparison do not have a shared enum type.",
    },
];

const LISTENERS: &[&str] = &["BinaryExpression", "CaseClause", "SwitchCase"];

impl RustLintRule for NoUnsafeEnumComparisonRule {
    fn name(&self) -> &'static str {
        "no-unsafe-enum-comparison"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow comparing an enum value with a value from a different enum domain."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        match node.kind.as_str() {
            "BinaryExpression" => check_binary_expression(ctx, node),
            // tsgolint registers `CaseClause`; TS-ESTree calls these `SwitchCase`.
            // Accept both so the rule works regardless of which kind the host
            // forwards.
            "CaseClause" | "SwitchCase" => check_case_clause(ctx, node),
            _ => {}
        }
    }
}

fn check_binary_expression(ctx: &mut RuleContext<'_>, node: &LintNode) {
    // Upstream only inspects relational / equality operators.
    if !matches!(
        node.field_str("operator"),
        Some("<" | "<=" | ">" | ">=" | "==" | "===" | "!=" | "!==")
    ) {
        return;
    }
    let Some(left) = node.child("left") else {
        return;
    };
    let Some(right) = node.child("right") else {
        return;
    };
    if is_mismatched_comparison(left, right) {
        ctx.report("mismatchedCondition", node.range);
    }
}

fn check_case_clause(ctx: &mut RuleContext<'_>, node: &LintNode) {
    // Upstream: leftType = type of node.Parent.Parent.Expression() (the switch
    // discriminant), rightType = type of node.Expression() (the case test).
    //
    // `default:` clauses carry no test expression and are skipped, matching the
    // Go code which only fires when the case has an expression.
    let Some(test) = case_test(node) else {
        return;
    };
    let Some(discriminant) = case_discriminant(node) else {
        return;
    };
    if is_mismatched_comparison(discriminant, test) {
        ctx.report("mismatchedCase", node.range);
    }
}

/// Returns the case clause's test expression (`case <test>:`), if present.
///
/// TS-ESTree names the field `test`; some hosts forward `expression`. Either is
/// accepted.
fn case_test(node: &LintNode) -> Option<&LintNode> {
    node.child("test").or_else(|| node.child("expression"))
}

/// Returns the switch discriminant for a case clause.
///
/// The host attaches the discriminant operand as the `__switchDiscriminant`
/// child so the enum facts that live on operand nodes are available without
/// re-walking the AST in Rust. Fall back to a `discriminant` child when present.
fn case_discriminant(node: &LintNode) -> Option<&LintNode> {
    node.child("__switchDiscriminant")
        .or_else(|| node.child("discriminant"))
}

/// Faithful port of the Go `isMismatchedComparison`.
fn is_mismatched_comparison(left: &LintNode, right: &LintNode) -> bool {
    let left_enum_types = enum_type_ids(left);
    let right_enum_types = enum_type_ids(right);

    // Allow comparisons that don't have anything to do with enums:
    //   1 === 2;
    if left_enum_types.is_empty() && right_enum_types.is_empty() {
        return false;
    }

    // Allow comparisons that share an enum type:
    //   Fruit.Apple === Fruit.Banana;
    if left_enum_types
        .iter()
        .any(|id| right_enum_types.contains(id))
    {
        return false;
    }

    // If a union-part type exists on both sides, the comparison is safe:
    //   declare const fruit: Fruit.Apple | 0;
    //   fruit === 0;
    let left_parts = union_part_type_ids(left);
    let right_parts = union_part_type_ids(right);
    if !left_parts.is_empty()
        && !right_parts.is_empty()
        && left_parts.iter().any(|id| right_parts.contains(id))
    {
        return false;
    }

    type_violates(left, right) || type_violates(right, left)
}

/// Faithful port of the Go `typeViolates`.
///
/// `left` provides the union-part enum value kinds; `right` provides whether the
/// compared-against type is number-like / string-like (computed over its
/// intersection parts by the host).
fn type_violates(left: &LintNode, right: &LintNode) -> bool {
    let right_number_like = is_number_like(right);
    let right_string_like = is_string_like(right);
    if !right_number_like && !right_string_like {
        return false;
    }

    union_part_enum_value_kinds(left).iter().any(|kind| {
        (kind == "number" && right_number_like) || (kind == "string" && right_string_like)
    })
}

/// Reads `__enumTypeIds`: stable identifiers of the enum types that the node's
/// type belongs to (upstream `utils.GetEnumTypes`).
fn enum_type_ids(node: &LintNode) -> Vec<String> {
    string_array_field(node, "__enumTypeIds")
}

/// Reads `__unionPartTypeIds`: a stable identifier per union constituent of the
/// node's type (upstream `utils.UnionTypeParts`).
fn union_part_type_ids(node: &LintNode) -> Vec<String> {
    string_array_field(node, "__unionPartTypeIds")
}

/// Reads `__unionPartEnumValueKinds`: for each union constituent, the enum value
/// kind ("number" / "string"), filtered to enum-like parts only (upstream
/// `getEnumValueType` over `leftTypeParts`).
fn union_part_enum_value_kinds(node: &LintNode) -> Vec<String> {
    string_array_field(node, "__unionPartEnumValueKinds")
}

/// Reads `__isNumberLike`: whether any intersection part of the node's type is
/// number-like (upstream `checker.TypeFlagsNumberLike`).
fn is_number_like(node: &LintNode) -> bool {
    node.field_bool("__isNumberLike").unwrap_or(false)
}

/// Reads `__isStringLike`: whether any intersection part of the node's type is
/// string-like (upstream `checker.TypeFlagsStringLike`).
fn is_string_like(node: &LintNode) -> bool {
    node.field_bool("__isStringLike").unwrap_or(false)
}

fn string_array_field(node: &LintNode, key: &str) -> Vec<String> {
    node.fields
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule, TextRange};
    use super::NoUnsafeEnumComparisonRule;

    fn run(node: &LintNode) -> Vec<String> {
        let rule = NoUnsafeEnumComparisonRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
            .into_iter()
            .map(|diagnostic| diagnostic.message_id)
            .collect()
    }

    fn bare(kind: &str) -> LintNode {
        LintNode {
            kind: kind.to_owned(),
            range: TextRange::new(0, 1),
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields: BTreeMap::new(),
            children: BTreeMap::new(),
            child_lists: BTreeMap::new(),
        }
    }

    /// Builds an operand node carrying the raw enum facts.
    fn operand(
        enum_ids: &[&str],
        union_ids: &[&str],
        enum_kinds: &[&str],
        number_like: bool,
        string_like: bool,
    ) -> LintNode {
        let mut node = bare("Identifier");
        node.fields
            .insert("__enumTypeIds".to_owned(), json!(enum_ids));
        node.fields
            .insert("__unionPartTypeIds".to_owned(), json!(union_ids));
        node.fields
            .insert("__unionPartEnumValueKinds".to_owned(), json!(enum_kinds));
        node.fields
            .insert("__isNumberLike".to_owned(), json!(number_like));
        node.fields
            .insert("__isStringLike".to_owned(), json!(string_like));
        node
    }

    fn binary(operator: &str, left: LintNode, right: LintNode) -> LintNode {
        let mut node = bare("BinaryExpression");
        node.range = TextRange::new(0, 20);
        node.fields.insert("operator".to_owned(), json!(operator));
        node.children.insert("left".to_owned(), left);
        node.children.insert("right".to_owned(), right);
        node
    }

    #[test]
    fn reports_number_enum_compared_to_number_literal() {
        // enum Fruit { Apple } ; Fruit.Apple === 1
        let left = operand(&["Fruit"], &["Fruit.Apple"], &["number"], true, false);
        let right = operand(&[], &["1"], &[], true, false);
        let node = binary("===", left, right);
        assert_eq!(run(&node), vec!["mismatchedCondition".to_owned()]);
    }

    #[test]
    fn reports_string_enum_compared_to_string_literal() {
        // enum Vegetable { Asparagus = 'asparagus', ... } ; Vegetable.Asparagus === 'beet'
        let left = operand(
            &["Vegetable"],
            &["Vegetable.Asparagus"],
            &["string"],
            false,
            true,
        );
        let right = operand(&[], &["'beet'"], &[], false, true);
        let node = binary("===", left, right);
        assert_eq!(run(&node), vec!["mismatchedCondition".to_owned()]);
    }

    #[test]
    fn reports_two_different_enums() {
        // Fruit.Apple === Fruit2.Apple2
        let left = operand(&["Fruit"], &["Fruit.Apple"], &["number"], true, false);
        let right = operand(&["Fruit2"], &["Fruit2.Apple2"], &["number"], true, false);
        let node = binary("===", left, right);
        assert_eq!(run(&node), vec!["mismatchedCondition".to_owned()]);
    }

    #[test]
    fn reports_mismatched_case_clause() {
        // switch (fruit) { case 0: } where fruit: Fruit (numeric enum)
        let discriminant = operand(&["Fruit"], &["Fruit.Apple"], &["number"], true, false);
        let test = operand(&[], &["0"], &[], true, false);
        let mut case = bare("SwitchCase");
        case.range = TextRange::new(0, 10);
        case.children.insert("test".to_owned(), test);
        case.children
            .insert("__switchDiscriminant".to_owned(), discriminant);
        assert_eq!(run(&case), vec!["mismatchedCase".to_owned()]);
    }

    #[test]
    fn ignores_non_enum_comparison() {
        // 1 === 2
        let left = operand(&[], &["1"], &[], true, false);
        let right = operand(&[], &["2"], &[], true, false);
        let node = binary("===", left, right);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_shared_enum_type() {
        // Fruit.Apple === Fruit.Banana
        let left = operand(&["Fruit"], &["Fruit.Apple"], &["number"], true, false);
        let right = operand(&["Fruit"], &["Fruit.Banana"], &["number"], true, false);
        let node = binary("===", left, right);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_union_with_plain_number() {
        // declare const fruit: Fruit | number; fruit === -1;
        // typeViolates checks enum value kind against number/string-likeness of the
        // OTHER operand. The literal `-1` is number-like, but the union `Fruit | number`
        // shares the `number` constituent with `-1`'s type, so it is allowed.
        let left = operand(
            &["Fruit"],
            &["Fruit.Apple", "number"],
            &["number"],
            true,
            false,
        );
        let right = operand(&[], &["number"], &[], true, false);
        let node = binary("===", left, right);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_unsupported_operator() {
        // 1 << Fruit.Apple is not a relational/equality comparison.
        let left = operand(&[], &["1"], &[], true, false);
        let right = operand(&["Fruit"], &["Fruit.Apple"], &["number"], true, false);
        let node = binary("<<", left, right);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_default_case() {
        // switch (vegetable) { default: } -> no test expression.
        let discriminant = operand(
            &["Vegetable"],
            &["Vegetable.Asparagus"],
            &["string"],
            false,
            true,
        );
        let mut case = bare("SwitchCase");
        case.children
            .insert("__switchDiscriminant".to_owned(), discriminant);
        assert!(run(&case).is_empty());
    }
}
