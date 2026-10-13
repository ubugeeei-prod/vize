use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{rule_option_bool, strip_chain_expression};

/// Type-aware rule that prefers an optional-chain expression over the
/// equivalent short-circuiting boolean chain.
///
/// This is a faithful port of the tsgolint `prefer-optional-chain` rule's
/// *reporting decision*. Upstream additionally produces an autofix; the Rust
/// dedicated rule keeps the decision logic in Rust and reports the diagnostic.
/// The fix-text synthesis (`flattenForFix`/`buildOptionalChain`) is not part of
/// the decision and is intentionally omitted here.
#[derive(Clone, Copy, Debug, Default)]
pub struct PreferOptionalChainRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "preferOptionalChain",
        description: "Prefer using an optional chain expression instead, as it's more concise and easier to read.",
    },
    RuleMessage {
        id: "optionalChainSuggest",
        description: "Change to an optional chain.",
    },
];

// Upstream listens on every `BinaryExpression` and filters to the logical
// operators `&&`, `||`, `??`. In ESTree, `&&`/`||`/`??` are `LogicalExpression`
// (not `BinaryExpression`), so that is the node kind we visit. The empty-object
// pattern `(x || {}).y` reports on the surrounding `MemberExpression`, so we
// listen on it as well.
const LISTENERS: &[&str] = &["LogicalExpression", "MemberExpression"];

impl RustLintRule for PreferOptionalChainRule {
    fn name(&self) -> &'static str {
        "prefer-optional-chain"
    }

    fn docs_description(&self) -> &'static str {
        "Prefer optional chaining over repeated short-circuit checks."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        let opts = Options::from_node(node);
        match node.kind.as_str() {
            "LogicalExpression" => {
                let Some(operator) = node.field_str("operator") else {
                    return;
                };
                match operator {
                    "&&" | "||" => process_chain(ctx, node, operator, opts),
                    // `??` only participates in the empty-object pattern, which
                    // is handled from the `MemberExpression` listener.
                    _ => {}
                }
            }
            "MemberExpression" => handle_empty_object_pattern(ctx, node, opts),
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Options {
    // `allowPotentiallyUnsafeFixesThatModifyTheReturnTypeIKnowWhatImDoing`.
    allow_unsafe: bool,
    check_any: bool,
    check_bigint: bool,
    check_boolean: bool,
    check_number: bool,
    check_string: bool,
    check_unknown: bool,
    require_nullish: bool,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        Self {
            allow_unsafe: rule_option_bool(
                node,
                "allowPotentiallyUnsafeFixesThatModifyTheReturnTypeIKnowWhatImDoing",
            )
            .unwrap_or(false),
            check_any: rule_option_bool(node, "checkAny").unwrap_or(true),
            check_bigint: rule_option_bool(node, "checkBigInt").unwrap_or(true),
            check_boolean: rule_option_bool(node, "checkBoolean").unwrap_or(true),
            check_number: rule_option_bool(node, "checkNumber").unwrap_or(true),
            check_string: rule_option_bool(node, "checkString").unwrap_or(true),
            check_unknown: rule_option_bool(node, "checkUnknown").unwrap_or(true),
            require_nullish: rule_option_bool(node, "requireNullish").unwrap_or(false),
        }
    }
}

/// Raw nullish/loose-boolean type facts the host attaches to each operand's
/// compared expression (and to its base identifier). Mirrors the subset of
/// upstream `TypeInfo` flags the reporting decision consumes.
#[derive(Clone, Copy, Debug, Default)]
struct TypeInfo {
    /// `true` when the host could not compute type flags (treated like an empty
    /// union: cannot prove anything about nullability).
    empty: bool,
    has_null: bool,
    has_undefined: bool,
    has_any: bool,
    has_unknown: bool,
    has_bigint_like: bool,
    has_boolean_like: bool,
    has_number_like: bool,
    has_string_like: bool,
    // A falsy-but-non-nullish literal member (e.g. `0`, `''`, `false`).
    has_falsy_non_nullish_literal: bool,
}

impl TypeInfo {
    /// Reads the `__nullishParts` raw fact off a node, if present.
    fn from_node(node: &LintNode) -> Self {
        let Some(parts) = node.fields.get("__nullishParts") else {
            return Self {
                empty: true,
                ..Self::default()
            };
        };
        let flag = |key: &str| {
            parts
                .get(key)
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        };
        Self {
            empty: false,
            has_null: flag("hasNull"),
            has_undefined: flag("hasUndefined"),
            has_any: flag("hasAny"),
            has_unknown: flag("hasUnknown"),
            has_bigint_like: flag("hasBigIntLike"),
            has_boolean_like: flag("hasBooleanLike"),
            has_number_like: flag("hasNumberLike"),
            has_string_like: flag("hasStringLike"),
            has_falsy_non_nullish_literal: flag("hasFalsyNonNullishLiteral"),
        }
    }

    fn has_explicit_nullish(self) -> bool {
        self.has_null || self.has_undefined
    }

    fn is_any_or_unknown(self) -> bool {
        self.has_any || self.has_unknown
    }
}

// ---------------------------------------------------------------------------
// Operand classification
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperandType {
    Invalid,
    Plain,
    NotStrictEqualNull,
    NotStrictEqualUndef,
    NotEqualBoth,
    Not,
    NegatedAndOperand,
    TypeofCheck,
    Comparison,
    EqualNull,
    StrictEqualNull,
    StrictEqualUndef,
}

/// A single flattened operand of the logical chain. `compared` is the access
/// expression being null-checked (e.g. for `foo != null`, this is `foo`).
struct Operand<'a> {
    typ: OperandType,
    node: &'a LintNode,
    compared: Option<&'a LintNode>,
}

/// ESTree wrapper-skipping that mirrors `unwrapForComparison` in Go: skips
/// parentheses (implicit in ESTree), non-null assertions and type assertions.
fn unwrap_for_comparison(node: &LintNode) -> &LintNode {
    let mut current = strip_chain_expression(node);
    loop {
        current = match current.kind.as_str() {
            "TSNonNullExpression" | "TSAsExpression" | "TSTypeAssertion" => {
                match current.child("expression") {
                    Some(inner) => strip_chain_expression(inner),
                    None => return current,
                }
            }
            _ => return current,
        };
    }
}

/// Mirrors upstream `utils.IsAccessExpression`, which treats property access,
/// element access, and call expressions as access expressions. In ESTree both
/// property and element access are `MemberExpression` (distinguished by the
/// `computed` flag), so we match `MemberExpression` and `CallExpression`.
fn is_access_expression(node: &LintNode) -> bool {
    let node = strip_chain_expression(node);
    matches!(node.kind.as_str(), "MemberExpression" | "CallExpression")
}

fn is_identifier(node: &LintNode) -> bool {
    node.kind == "Identifier"
}

fn is_this(node: &LintNode) -> bool {
    node.kind == "ThisExpression"
}

fn is_null_literal(node: &LintNode) -> bool {
    let node = strip_chain_expression(node);
    node.kind == "Literal"
        && node
            .fields
            .get("value")
            .is_some_and(serde_json::Value::is_null)
}

fn is_undefined_identifier(node: &LintNode) -> bool {
    let node = strip_chain_expression(node);
    node.kind == "Identifier" && node.field_stringish("name").as_deref() == Some("undefined")
}

fn is_nullish_literal(node: &LintNode) -> bool {
    is_null_literal(node) || is_undefined_identifier(node)
}

/// Walks an access chain down to the root expression (base identifier / this).
fn get_base(node: &LintNode) -> &LintNode {
    let mut current = strip_chain_expression(node);
    loop {
        current = match current.kind.as_str() {
            "MemberExpression" => match current.child("object") {
                Some(object) => strip_chain_expression(object),
                None => return current,
            },
            "CallExpression" => match current.child("callee") {
                Some(callee) => strip_chain_expression(callee),
                None => return current,
            },
            "TSNonNullExpression" | "TSAsExpression" | "TSTypeAssertion" => {
                match current.child("expression") {
                    Some(inner) => strip_chain_expression(inner),
                    None => return current,
                }
            }
            _ => return current,
        };
    }
}

/// Structural equality ignoring optional/parenthesis/assertion wrappers. Mirror
/// of `areNodesStructurallyEqual` over the ESTree subset that appears in chains.
fn nodes_structurally_equal(a: &LintNode, b: &LintNode) -> bool {
    let a = unwrap_for_comparison(a);
    let b = unwrap_for_comparison(b);
    if a.kind != b.kind {
        return false;
    }
    match a.kind.as_str() {
        "Identifier" => a.field_stringish("name") == b.field_stringish("name"),
        "ThisExpression" | "Super" => true,
        "Literal" => a.field_stringish("raw") == b.field_stringish("raw"),
        "MemberExpression" => {
            let computed = a.field_bool("computed").unwrap_or(false);
            if computed != b.field_bool("computed").unwrap_or(false) {
                return false;
            }
            let (Some(ao), Some(bo)) = (a.child("object"), b.child("object")) else {
                return false;
            };
            let (Some(ap), Some(bp)) = (a.child("property"), b.child("property")) else {
                return false;
            };
            nodes_structurally_equal(ao, bo) && nodes_structurally_equal(ap, bp)
        }
        "CallExpression" => {
            let (Some(ac), Some(bc)) = (a.child("callee"), b.child("callee")) else {
                return false;
            };
            if !nodes_structurally_equal(ac, bc) {
                return false;
            }
            let aa = a.child_list("arguments").unwrap_or(&[]);
            let ba = b.child_list("arguments").unwrap_or(&[]);
            aa.len() == ba.len()
                && aa
                    .iter()
                    .zip(ba.iter())
                    .all(|(x, y)| nodes_structurally_equal(x, y))
        }
        _ => false,
    }
}

/// `shorter` is a structural prefix of `longer` (e.g. `foo.bar` of `foo.bar.baz`).
fn is_node_prefix_of(shorter: &LintNode, longer: &LintNode) -> bool {
    let shorter = unwrap_for_comparison(shorter);
    let longer = unwrap_for_comparison(longer);
    if nodes_structurally_equal(shorter, longer) {
        return true;
    }
    let base = match longer.kind.as_str() {
        "MemberExpression" => longer.child("object"),
        "CallExpression" => longer.child("callee"),
        "TSNonNullExpression" | "TSAsExpression" | "TSTypeAssertion" => longer.child("expression"),
        _ => None,
    };
    base.is_some_and(|base| is_node_prefix_of(shorter, base))
}

/// True when the node carries side effects that block chain merging (mirror of
/// upstream `hasSideEffects` over ESTree).
fn has_side_effects(node: &LintNode) -> bool {
    match node.kind.as_str() {
        "UpdateExpression" => true,
        "YieldExpression" => true,
        "AssignmentExpression" => true,
        "MemberExpression" => {
            node.child("object").is_some_and(has_side_effects)
                || (node.field_bool("computed").unwrap_or(false)
                    && node.child("property").is_some_and(has_side_effects))
        }
        "CallExpression" => node.child("callee").is_some_and(has_side_effects),
        _ => false,
    }
}

fn contains_optional_chain(node: &LintNode) -> bool {
    if node.kind == "ChainExpression" {
        return true;
    }
    match node.kind.as_str() {
        "MemberExpression" => {
            node.field_bool("optional").unwrap_or(false)
                || node.child("object").is_some_and(contains_optional_chain)
        }
        "CallExpression" => {
            node.field_bool("optional").unwrap_or(false)
                || node.child("callee").is_some_and(contains_optional_chain)
        }
        "LogicalExpression" | "BinaryExpression" => {
            node.child("left").is_some_and(contains_optional_chain)
                || node.child("right").is_some_and(contains_optional_chain)
        }
        _ => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cmp {
    Equal,
    Subset,
    Superset,
    Invalid,
}

/// Compare two compared-expressions for chainability. Mirror of `compareNodes`
/// (without the call-signature divergence check, which needs source text).
fn compare_nodes(left: &LintNode, right: &LintNode) -> Cmp {
    if has_side_effects(left) || has_side_effects(right) {
        return Cmp::Invalid;
    }
    let left_un = strip_chain_expression(left);
    // Block fresh-instance / standalone-call roots unless an optional chain is
    // already present, matching upstream's instance-stability gate.
    if !contains_optional_chain(left) {
        let is_standalone_call = left_un.kind == "CallExpression"
            && left_un
                .child("callee")
                .map(strip_chain_expression)
                .is_some_and(is_identifier);
        if is_standalone_call
            || matches!(
                left_un.kind.as_str(),
                "NewExpression"
                    | "ArrayExpression"
                    | "ObjectExpression"
                    | "FunctionExpression"
                    | "ArrowFunctionExpression"
                    | "ClassExpression"
                    | "JSXElement"
                    | "JSXFragment"
                    | "TemplateLiteral"
                    | "TaggedTemplateExpression"
                    | "AwaitExpression"
            )
        {
            return Cmp::Invalid;
        }
    }
    if nodes_structurally_equal(left, right) {
        Cmp::Equal
    } else if is_node_prefix_of(left, right) {
        Cmp::Subset
    } else if is_node_prefix_of(right, left) {
        Cmp::Superset
    } else {
        Cmp::Invalid
    }
}

fn is_and_operator(operator: &str) -> bool {
    operator == "&&"
}

/// Classifies one operand node into an `OperandType` plus its compared access
/// expression. Mirror of `parseOperand` over ESTree.
fn parse_operand<'a>(node: &'a LintNode, operator: &str) -> Operand<'a> {
    let is_and = is_and_operator(operator);
    let unwrapped = unwrap_for_comparison(node);

    // Bare `this` cannot be converted.
    if is_this(unwrapped) {
        return Operand {
            typ: OperandType::Invalid,
            node,
            compared: None,
        };
    }

    // `!x` style operands.
    if unwrapped.kind == "UnaryExpression" && unwrapped.field_str("operator") == Some("!") {
        let Some(argument) = unwrapped.child("argument") else {
            return invalid(node);
        };
        if is_this(unwrap_for_comparison(argument)) {
            return invalid(node);
        }
        let typ = if is_and {
            OperandType::NegatedAndOperand
        } else {
            OperandType::Not
        };
        return Operand {
            typ,
            node,
            compared: Some(argument),
        };
    }

    if unwrapped.kind == "BinaryExpression" {
        return parse_binary_operand(node, unwrapped, operator);
    }

    // Plain truthiness operand (`foo`, `foo.bar`, `foo()`...).
    Operand {
        typ: OperandType::Plain,
        node,
        compared: Some(unwrapped),
    }
}

fn invalid(node: &LintNode) -> Operand<'_> {
    Operand {
        typ: OperandType::Invalid,
        node,
        compared: None,
    }
}

fn parse_binary_operand<'a>(node: &'a LintNode, bin: &'a LintNode, operator: &str) -> Operand<'a> {
    let is_and = is_and_operator(operator);
    let op = bin.field_str("operator").unwrap_or("");
    let (Some(left), Some(right)) = (bin.child("left"), bin.child("right")) else {
        return invalid(node);
    };
    let left = strip_chain_expression(left);
    let right = strip_chain_expression(right);

    // Relational operators never participate in optional chaining.
    if matches!(op, "<" | ">" | "<=" | ">=" | "in" | "instanceof") {
        return invalid(node);
    }

    // typeof x === 'undefined' style checks.
    let typeof_operand = |expr: &'a LintNode, value: &LintNode| -> Option<Operand<'a>> {
        let expr = strip_chain_expression(expr);
        if expr.kind == "UnaryExpression" && expr.field_str("operator") == Some("typeof") {
            let is_undef_str = value.kind == "Literal"
                && value.field_stringish("value").as_deref() == Some("undefined");
            if is_undef_str {
                let undef_and = matches!(op, "!==" | "!=") && is_and;
                let undef_or = matches!(op, "===" | "==") && !is_and;
                if undef_and || undef_or {
                    return expr.child("argument").map(|argument| Operand {
                        typ: OperandType::TypeofCheck,
                        node,
                        compared: Some(argument),
                    });
                }
            }
        }
        None
    };
    if let Some(operand) = typeof_operand(left, right) {
        return operand;
    }
    if let Some(operand) = typeof_operand(right, left) {
        return operand;
    }

    // Identify the (expression, compared-value) pair.
    let (expr, value) = if is_nullish_literal(right) || is_string_literal(right) {
        (Some(left), Some(right))
    } else if is_nullish_literal(left) || is_string_literal(left) {
        (Some(right), Some(left))
    } else {
        (None, None)
    };

    if let (Some(expr), Some(value)) = (expr, value) {
        let expr = strip_chain_expression(expr);
        let is_null = is_null_literal(value);
        let is_undef = is_undefined_identifier(value);
        if is_and {
            match op {
                "!==" if is_null => {
                    return access(node, expr, OperandType::NotStrictEqualNull);
                }
                "!==" if is_undef => {
                    return access(node, expr, OperandType::NotStrictEqualUndef);
                }
                "!=" if is_null || is_undef => {
                    return access(node, expr, OperandType::NotEqualBoth);
                }
                "===" if is_null && (is_identifier(expr) || is_this(expr)) => {
                    return access(node, expr, OperandType::StrictEqualNull);
                }
                "===" if is_undef && (is_identifier(expr) || is_this(expr)) => {
                    return access(node, expr, OperandType::StrictEqualUndef);
                }
                "==" if (is_null || is_undef) && (is_identifier(expr) || is_this(expr)) => {
                    return access(node, expr, OperandType::EqualNull);
                }
                _ => {}
            }
        } else {
            let is_access = is_access_expression(expr);
            if is_access && (is_null || is_undef) {
                return access(node, expr, OperandType::Comparison);
            } else if !is_access {
                match op {
                    "===" if is_null => {
                        return access(node, expr, OperandType::NotStrictEqualNull);
                    }
                    "===" if is_undef => {
                        return access(node, expr, OperandType::NotStrictEqualUndef);
                    }
                    "==" if is_null || is_undef => {
                        return access(node, expr, OperandType::NotEqualBoth);
                    }
                    _ => {}
                }
            }
        }
    }

    // Generic comparison operand: must contain at least one access expression
    // and the two access sides cannot share a base.
    if matches!(op, "==" | "!=" | "===" | "!==") {
        let left_access = is_access_expression(left);
        let right_access = is_access_expression(right);
        if left_access && right_access {
            let left_base = get_base(left);
            let right_base = get_base(right);
            if nodes_structurally_equal(left_base, right_base) {
                return invalid(node);
            }
        }
        let compared = if right_access {
            Some(right)
        } else if left_access {
            Some(left)
        } else {
            None
        };
        if let Some(compared) = compared {
            return Operand {
                typ: OperandType::Comparison,
                node,
                compared: Some(compared),
            };
        }
    }

    if is_and {
        // Logical/comparison binary operands inside an AND chain that are not
        // access-based comparisons are invalid.
        if matches!(op, "==" | "!=" | "===" | "!==") {
            return invalid(node);
        }
        return Operand {
            typ: OperandType::Plain,
            node,
            compared: Some(bin),
        };
    }

    invalid(node)
}

fn access<'a>(node: &'a LintNode, expr: &'a LintNode, typ: OperandType) -> Operand<'a> {
    Operand {
        typ,
        node,
        compared: Some(expr),
    }
}

fn is_string_literal(node: &LintNode) -> bool {
    let node = strip_chain_expression(node);
    node.kind == "Literal"
        && node
            .fields
            .get("value")
            .is_some_and(serde_json::Value::is_string)
}

// ---------------------------------------------------------------------------
// Chain collection + reporting decision
// ---------------------------------------------------------------------------

/// Flattens a left-associated `LogicalExpression` of a single operator into its
/// operand nodes (mirror of `collectOperands`).
fn collect_operands<'a>(node: &'a LintNode, operator: &str, out: &mut Vec<&'a LintNode>) {
    let node = strip_chain_expression(node);
    if node.kind == "LogicalExpression" && node.field_str("operator") == Some(operator) {
        if let Some(left) = node.child("left") {
            collect_operands(left, operator, out);
        }
        if let Some(right) = node.child("right") {
            collect_operands(right, operator, out);
        }
    } else {
        out.push(node);
    }
}

fn process_chain(ctx: &mut RuleContext<'_>, node: &LintNode, operator: &str, opts: Options) {
    // Upstream skips chains inside JSX (different short-circuit semantics).
    if node.field_str("__insideJsx") == Some("true") || node.field_bool("__insideJsx") == Some(true)
    {
        return;
    }

    // Only report on the outermost expression of a same-operator chain: if the
    // parent is a `LogicalExpression` with the same operator, defer to it.
    if node.field_str("__parentKind") == Some("LogicalExpression")
        && node.field_str("__parentOperator") == Some(operator)
    {
        return;
    }

    let mut operand_nodes = Vec::new();
    collect_operands(node, operator, &mut operand_nodes);
    if operand_nodes.len() < 2 {
        return;
    }

    let operands: Vec<Operand<'_>> = operand_nodes
        .iter()
        .map(|operand| parse_operand(operand, operator))
        .collect();

    let chains = if is_and_operator(operator) {
        build_and_chains(&operands)
    } else {
        build_or_chains(&operands)
    };

    for chain in chains {
        if validate_chain_for_reporting(&chain, operator, opts) {
            ctx.report("preferOptionalChain", node.range);
            return;
        }
    }
}

/// Builds maximal AND chains by greedily extending while the compared
/// expressions are prefixes/equal. Simplified faithful core of `buildAndChains`.
fn build_and_chains<'a, 'b>(operands: &'b [Operand<'a>]) -> Vec<Vec<&'b Operand<'a>>> {
    let mut all: Vec<Vec<&Operand>> = Vec::new();
    let mut current: Vec<&Operand> = Vec::new();
    let mut last_expr: Option<&LintNode> = None;

    for op in operands {
        if matches!(
            op.typ,
            OperandType::Invalid
                | OperandType::NegatedAndOperand
                | OperandType::EqualNull
                | OperandType::StrictEqualNull
                | OperandType::StrictEqualUndef
        ) {
            flush(&mut all, &mut current);
            last_expr = None;
            continue;
        }
        match (current.is_empty(), last_expr, op.compared) {
            (true, _, compared) => {
                current.push(op);
                last_expr = compared;
            }
            (false, Some(prev), Some(compared)) => {
                let cmp = compare_nodes(prev, compared);
                if matches!(cmp, Cmp::Subset | Cmp::Equal) {
                    current.push(op);
                    last_expr = Some(compared);
                } else {
                    flush(&mut all, &mut current);
                    current.push(op);
                    last_expr = Some(compared);
                }
            }
            (false, _, _) => {
                flush(&mut all, &mut current);
                last_expr = None;
            }
        }
    }
    flush(&mut all, &mut current);
    all
}

/// Builds a single OR chain by greedily extending. Core of `buildOrChains`.
fn build_or_chains<'a, 'b>(operands: &'b [Operand<'a>]) -> Vec<Vec<&'b Operand<'a>>> {
    let mut chain: Vec<&Operand> = Vec::new();
    let mut last_expr: Option<&LintNode> = None;

    for op in operands {
        if !is_valid_or_operand(op.typ) {
            if chain.len() >= 2 {
                break;
            }
            chain.clear();
            last_expr = None;
            continue;
        }
        match (chain.is_empty(), last_expr, op.compared) {
            (true, _, compared) => {
                chain.push(op);
                last_expr = compared;
            }
            (false, Some(prev), Some(compared)) => {
                let cmp = compare_nodes(prev, compared);
                if matches!(cmp, Cmp::Subset | Cmp::Equal) {
                    chain.push(op);
                    last_expr = Some(compared);
                } else if chain.len() >= 2 {
                    break;
                } else {
                    chain.clear();
                    chain.push(op);
                    last_expr = Some(compared);
                }
            }
            (false, _, _) => {
                if chain.len() >= 2 {
                    break;
                }
                chain.clear();
                last_expr = None;
            }
        }
    }

    if chain.len() < 2 {
        Vec::new()
    } else {
        vec![chain]
    }
}

fn is_valid_or_operand(typ: OperandType) -> bool {
    matches!(
        typ,
        OperandType::Not
            | OperandType::Comparison
            | OperandType::Plain
            | OperandType::TypeofCheck
            | OperandType::NotStrictEqualNull
            | OperandType::NotStrictEqualUndef
            | OperandType::NotEqualBoth
            | OperandType::StrictEqualNull
            | OperandType::StrictEqualUndef
            | OperandType::EqualNull
    )
}

fn flush<'a, 'b>(all: &mut Vec<Vec<&'b Operand<'a>>>, current: &mut Vec<&'b Operand<'a>>) {
    if current.len() >= 2 {
        all.push(std::mem::take(current));
    } else {
        current.clear();
    }
}

// ---------------------------------------------------------------------------
// Validation (faithful subset of validateChain* family)
// ---------------------------------------------------------------------------

fn has_same_base(chain: &[&Operand<'_>]) -> bool {
    let mut first: Option<&LintNode> = None;
    for op in chain {
        let Some(compared) = op.compared else {
            continue;
        };
        let base = get_base(compared);
        match first {
            None => first = Some(base),
            Some(prev) => {
                if !nodes_structurally_equal(prev, base) {
                    return false;
                }
            }
        }
    }
    true
}

fn has_property_access(chain: &[&Operand<'_>]) -> bool {
    chain
        .iter()
        .any(|op| op.compared.is_some_and(is_access_expression))
}

fn validate_chain(chain: &[&Operand<'_>], operator: &str, opts: Options) -> bool {
    if chain.len() < 2 {
        return false;
    }
    if !has_same_base(chain) {
        return false;
    }
    if !has_property_access(chain) {
        return false;
    }
    if should_skip_for_require_nullish(chain, operator, opts) {
        return false;
    }
    true
}

fn should_skip_for_require_nullish(chain: &[&Operand<'_>], operator: &str, opts: Options) -> bool {
    if !opts.require_nullish {
        return false;
    }
    if !is_and_operator(operator) && chain.first().is_some_and(|op| op.typ == OperandType::Not) {
        return true;
    }
    for (i, op) in chain.iter().enumerate() {
        if op.typ != OperandType::Plain {
            return false;
        }
        if is_and_operator(operator) && i < chain.len() - 1 {
            if let Some(compared) = op.compared {
                if TypeInfo::from_node(compared).has_explicit_nullish() {
                    return false;
                }
            }
        }
    }
    true
}

fn validate_chain_for_reporting(chain: &[&Operand<'_>], operator: &str, opts: Options) -> bool {
    if !validate_chain(chain, operator, opts) {
        return false;
    }
    if is_and_operator(operator) {
        validate_and_chain(chain, opts)
    } else {
        validate_or_chain(chain, opts)
    }
}

/// `all operands check the same expression` (no deeper access to chain) -> skip.
fn all_operands_check_same_expression(chain: &[&Operand<'_>]) -> bool {
    let Some(first) = chain[0].compared else {
        return false;
    };
    chain[1..].iter().all(|op| {
        op.compared
            .is_some_and(|compared| nodes_structurally_equal(first, compared))
    })
}

fn validate_and_chain(chain: &[&Operand<'_>], opts: Options) -> bool {
    let first = chain[0];

    // `foo?.bar && foo?.bar.baz`: first operand already optional-chained.
    if first.typ == OperandType::Plain && first.compared.is_some_and(contains_optional_chain) {
        return false;
    }

    if all_operands_check_same_expression(chain) {
        return false;
    }

    if !opts.allow_unsafe {
        // Non-null assertions in the chain are unsafe to fold.
        if chain
            .iter()
            .any(|op| op.node.kind == "TSNonNullExpression" || contains_non_null(op.node))
        {
            return false;
        }
        if has_incomplete_nullish_check(chain) {
            return false;
        }
    }

    // The first plain operand gates on the "loose boolean" check options and on
    // a return-type change introduced by folding away the truthiness check.
    if first.typ == OperandType::Plain {
        if let Some(compared) = first.compared {
            if should_skip_by_type(compared, opts) {
                return false;
            }
            if !opts.allow_unsafe && would_change_return_type(compared) {
                return false;
            }
        }
    }

    true
}

fn validate_or_chain(chain: &[&Operand<'_>], opts: Options) -> bool {
    // At least one operand must be an explicit (non-plain) check.
    if chain.iter().all(|op| op.typ == OperandType::Plain) {
        return false;
    }

    if all_operands_check_same_expression(chain) {
        return false;
    }

    // First negation operand return-type-change gate.
    if !opts.allow_unsafe {
        for op in chain {
            if matches!(op.typ, OperandType::Plain | OperandType::Not) {
                if let Some(compared) = op.compared {
                    if would_change_return_type(compared) {
                        return false;
                    }
                }
            }
        }
        if has_incomplete_or_nullish_check(chain) {
            return false;
        }
    }

    true
}

fn contains_non_null(node: &LintNode) -> bool {
    if node.kind == "TSNonNullExpression" {
        return true;
    }
    match node.kind.as_str() {
        "MemberExpression" => node.child("object").is_some_and(contains_non_null),
        "CallExpression" => node.child("callee").is_some_and(contains_non_null),
        "ChainExpression" => node.child("expression").is_some_and(contains_non_null),
        _ => false,
    }
}

/// Loose-boolean operand options gate: skips when the base type only contains a
/// kind the user opted not to check. Mirror of `shouldSkipByType`.
fn should_skip_by_type(node: &LintNode, opts: Options) -> bool {
    let info = TypeInfo::from_node(get_base(node));
    if info.empty {
        return false;
    }
    if opts.require_nullish && info.has_explicit_nullish() {
        return false;
    }
    if info.has_any && !opts.check_any {
        return true;
    }
    if info.has_bigint_like && !opts.check_bigint {
        return true;
    }
    if info.has_boolean_like && !opts.check_boolean {
        return true;
    }
    if info.has_number_like && !opts.check_number {
        return true;
    }
    if info.has_string_like && !opts.check_string {
        return true;
    }
    if info.has_unknown && !opts.check_unknown {
        return true;
    }
    false
}

/// `would folding the truthiness check change the result type?` Mirror of
/// `wouldChangeReturnType`.
fn would_change_return_type(node: &LintNode) -> bool {
    let info = TypeInfo::from_node(node);
    if info.empty {
        return false;
    }
    info.has_falsy_non_nullish_literal && !info.has_explicit_nullish()
}

/// Optional chaining tests for BOTH null and undefined. A chain that only checks
/// one of them (and the type can be the other) is unsafe to fold. Mirror of the
/// core of `hasIncompleteNullishCheck` for AND chains.
fn has_incomplete_nullish_check(chain: &[&Operand<'_>]) -> bool {
    let analysis = analyze_nullish(chain, false);
    if analysis.has_both || analysis.has_typeof {
        return false;
    }
    // If the guarded base type proves only one of null/undefined is present and
    // the matching strict check is used, the check is complete.
    if let Some(first) = chain.first().and_then(|op| op.compared) {
        let info = TypeInfo::from_node(first);
        if !info.empty && !info.is_any_or_unknown() {
            let only_null = analysis.has_null && !analysis.has_undefined;
            let only_undef = !analysis.has_null && analysis.has_undefined;
            if (only_null && info.has_null && !info.has_undefined)
                || (only_undef && info.has_undefined && !info.has_null)
            {
                return false;
            }
            // First operand not nullable at all: safe.
            if !info.has_explicit_nullish() && !info.is_any_or_unknown() {
                return false;
            }
        }
    }
    analysis.is_incomplete()
}

fn has_incomplete_or_nullish_check(chain: &[&Operand<'_>]) -> bool {
    let analysis = analyze_nullish(chain, true);
    if analysis.has_both {
        return false;
    }
    if let Some(first) = chain.first().and_then(|op| op.compared) {
        if !TypeInfo::from_node(first).has_explicit_nullish() {
            // First operand not nullish -> treated as safe by upstream.
            return false;
        }
    }
    analysis.is_incomplete()
}

#[derive(Default)]
struct NullishAnalysis {
    has_null: bool,
    has_undefined: bool,
    has_both: bool,
    has_typeof: bool,
}

impl NullishAnalysis {
    fn is_incomplete(&self) -> bool {
        let only_null = self.has_null && !self.has_undefined && !self.has_both;
        let only_undef = !self.has_null && self.has_undefined && !self.has_both;
        only_null || only_undef
    }
}

fn analyze_nullish(chain: &[&Operand<'_>], or_mode: bool) -> NullishAnalysis {
    let mut analysis = NullishAnalysis::default();
    for op in chain {
        match op.typ {
            OperandType::NotStrictEqualNull | OperandType::StrictEqualNull => {
                analysis.has_null = true;
            }
            OperandType::NotStrictEqualUndef | OperandType::StrictEqualUndef => {
                analysis.has_undefined = true;
            }
            OperandType::NotEqualBoth | OperandType::EqualNull => analysis.has_both = true,
            OperandType::TypeofCheck => {
                analysis.has_undefined = true;
                analysis.has_typeof = true;
            }
            OperandType::Not if or_mode => analysis.has_both = true,
            OperandType::Plain if !or_mode => analysis.has_both = true,
            _ => {}
        }
    }
    analysis
}

// ---------------------------------------------------------------------------
// (x || {}).y empty-object pattern
// ---------------------------------------------------------------------------

fn handle_empty_object_pattern(ctx: &mut RuleContext<'_>, node: &LintNode, opts: Options) {
    if node.kind != "MemberExpression" {
        return;
    }
    if node.field_bool("optional").unwrap_or(false) {
        return;
    }
    let Some(object) = node.child("object") else {
        return;
    };
    // Unwrap a single layer of parentheses (ESTree has no paren node; the host
    // marks computed members but parentheses are transparent here).
    let logical = strip_chain_expression(object);
    if logical.kind != "LogicalExpression" {
        return;
    }
    let operator = logical.field_str("operator").unwrap_or("");
    if operator != "||" && operator != "??" {
        return;
    }
    let Some(right) = logical.child("right") else {
        return;
    };
    let right = strip_chain_expression(right);
    if right.kind != "ObjectExpression" {
        return;
    }
    if !child_is_empty(right) {
        return;
    }
    if opts.require_nullish {
        let Some(left) = logical.child("left") else {
            return;
        };
        if !TypeInfo::from_node(left).has_explicit_nullish() {
            return;
        }
    }
    if opts.allow_unsafe {
        ctx.report("preferOptionalChain", node.range);
    } else {
        let suggestion = crate::lint::LintSuggestion {
            message_id: "optionalChainSuggest".to_owned(),
            message: "Change to an optional chain.".to_owned(),
            fixes: Vec::new(),
        };
        ctx.report_with_suggestions("preferOptionalChain", node.range, vec![suggestion]);
    }
}

fn child_is_empty(object: &LintNode) -> bool {
    object
        .child_list("properties")
        .map(<[LintNode]>::is_empty)
        .unwrap_or(true)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(node: &LintNode) -> Vec<crate::lint::LintDiagnostic> {
        let rule = PreferOptionalChainRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn ident(name: &str) -> serde_json::Value {
        json!({
            "kind": "Identifier",
            "range": { "start": 0, "end": 1 },
            "fields": { "name": name }
        })
    }

    fn member(object: serde_json::Value, property: &str) -> serde_json::Value {
        json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 1 },
            "fields": { "computed": false, "optional": false },
            "children": {
                "object": object,
                "property": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 1 },
                    "fields": { "name": property }
                }
            }
        })
    }

    fn logical(operator: &str, left: serde_json::Value, right: serde_json::Value) -> LintNode {
        serde_json::from_value(json!({
            "kind": "LogicalExpression",
            "range": { "start": 0, "end": 20 },
            "fields": { "operator": operator },
            "children": { "left": left, "right": right }
        }))
        .unwrap()
    }

    // foo && foo.bar  -> should report
    #[test]
    fn reports_simple_and_chain() {
        let node = logical("&&", ident("foo"), member(ident("foo"), "bar"));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "preferOptionalChain");
    }

    // foo.bar && foo.bar.baz -> should report
    #[test]
    fn reports_nested_and_chain() {
        let node = logical(
            "&&",
            member(ident("foo"), "bar"),
            member(member(ident("foo"), "bar"), "baz"),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
    }

    fn call(callee: serde_json::Value) -> serde_json::Value {
        json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 5 },
            "fields": { "optional": false },
            "children": { "callee": callee },
            "childLists": { "arguments": [] }
        })
    }

    // foo && foo() -> reports (callable access converts to foo?.())
    #[test]
    fn reports_optional_call_chain() {
        let node = logical("&&", ident("foo"), call(ident("foo")));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "preferOptionalChain");
    }

    // foo === null || foo.bar === null -> should report (OR nullish chain)
    #[test]
    fn reports_or_nullish_chain() {
        let null_lit = json!({
            "kind": "Literal",
            "range": { "start": 0, "end": 4 },
            "fields": { "value": null, "raw": "null" }
        });
        let left = json!({
            "kind": "BinaryExpression",
            "range": { "start": 0, "end": 11 },
            "fields": { "operator": "===" },
            "children": { "left": ident("foo"), "right": null_lit }
        });
        let null_lit2 = json!({
            "kind": "Literal",
            "range": { "start": 0, "end": 4 },
            "fields": { "value": null, "raw": "null" }
        });
        let right = json!({
            "kind": "BinaryExpression",
            "range": { "start": 0, "end": 20 },
            "fields": { "operator": "===" },
            "children": { "left": member(ident("foo"), "bar"), "right": null_lit2 }
        });
        let node = logical("||", left, right);
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
    }

    // foo && bar.baz -> different base, should NOT report
    #[test]
    fn ignores_different_base() {
        let node = logical("&&", ident("foo"), member(ident("bar"), "baz"));
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    // foo && bar -> two plain identifiers, no property access, should NOT report
    #[test]
    fn ignores_no_property_access() {
        let node = logical("&&", ident("foo"), ident("bar"));
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    // requireNullish: foo && foo.bar with non-nullish base -> should NOT report
    #[test]
    fn require_nullish_skips_non_nullish_base() {
        let mut node = logical("&&", ident("foo"), member(ident("foo"), "bar"));
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "requireNullish": true }]),
        );
        // No `__nullishParts` on the first operand -> not explicitly nullish.
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    // (foo || {}).bar -> should report on the MemberExpression
    #[test]
    fn reports_empty_object_pattern() {
        let object = json!({
            "kind": "LogicalExpression",
            "range": { "start": 1, "end": 10 },
            "fields": { "operator": "||" },
            "children": {
                "left": ident("foo"),
                "right": {
                    "kind": "ObjectExpression",
                    "range": { "start": 7, "end": 9 },
                    "childLists": { "properties": [] }
                }
            }
        });
        let node: LintNode = serde_json::from_value(member(object, "bar")).unwrap();
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "preferOptionalChain");
        assert_eq!(diagnostics[0].suggestions.len(), 1);
        assert_eq!(
            diagnostics[0].suggestions[0].message_id,
            "optionalChainSuggest"
        );
    }

    // (foo || { a: 1 }).bar -> non-empty object, should NOT report
    #[test]
    fn ignores_non_empty_object_pattern() {
        let object = json!({
            "kind": "LogicalExpression",
            "range": { "start": 1, "end": 14 },
            "fields": { "operator": "||" },
            "children": {
                "left": ident("foo"),
                "right": {
                    "kind": "ObjectExpression",
                    "range": { "start": 7, "end": 13 },
                    "childLists": { "properties": [ {
                        "kind": "Property",
                        "range": { "start": 9, "end": 13 }
                    } ] }
                }
            }
        });
        let node: LintNode = serde_json::from_value(member(object, "bar")).unwrap();
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }
}
