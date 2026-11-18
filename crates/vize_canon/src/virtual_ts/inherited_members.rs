//! `mixins:` / `extends:` members an Options API component inherits.
//!
//! The Vue 2 typed-instance bridge folds them into `$This`
//! (`generator::options_api_bridge::inherited`), and a Vue 3 template's
//! authored instance reads them for imported SFCs (#13270).
//!
//! A Vue 3 Options API template reads its non-own names off the authored
//! instance, `typeof __default__`'s instance. Vue's own mixin inference reads
//! a `defineComponent` result or a plain options object, but not a Vize SFC
//! export, whose component options carry open members. A component that
//! extends or mixes in another SFC therefore got an `any` index for every
//! inherited name, so `cx(1)` against a base `cx(key: string)` went
//! unchecked. For those references only, `$InheritedMembers` fills each name
//! the authored instance leaves `any` or lacks. A name the component itself
//! types keeps its own member, as Vue's option merging does. The index
//! signature stays, so a name nothing declares keeps today's `any`. Other
//! references keep their instance, and every diagnostic that names it,
//! unchanged.

use std::ops::Range;

use oxc_ast::ast::{Expression, ImportOrExportKind, ObjectExpression, Program, Statement};
use oxc_span::{GetSpan, SourceType};

use crate::options_api_setup_spread::{
    component_options_from_program, is_safe_value_identifier, option_expression_property,
};
use vize_carton::{FxHashSet, String, append};

/// Type-level helpers backing the inherited-member aliases.
pub(crate) const INHERITED_MEMBER_HELPERS: &str = concat!(
    "  type $InheritedInstance<T> = 0 extends (1 & T) ? any : [T] extends [never] ? $NoInheritedMembers : T extends abstract new (...args: any[]) => infer __I ? __I : T extends { methods: infer __M } ? 0 extends (1 & __M) ? $NoInheritedMembers : [__M] extends [never] ? $NoInheritedMembers : __M extends object ? [keyof __M] extends [never] ? $NoInheritedMembers : string extends keyof __M ? $NoInheritedMembers : number extends keyof __M ? $NoInheritedMembers : symbol extends keyof __M ? $NoInheritedMembers : { [__K in keyof __M]: __M[__K] } : $NoInheritedMembers : $NoInheritedMembers;\n",
    "  interface $NoInheritedMembers {}\n",
    "  type $InheritedMembers<T, __I = $InheritedInstance<T>> = [$IsAny<__I>] extends [true] ? $NoInheritedMembers : __I;\n",
);

/// The authored template instance type, without inherited members.
pub(crate) const AUTHORED_INSTANCE: &str =
    "(typeof __default__ extends abstract new (...args: any) => infer __I ? __I : {})";

/// The alias that replaces [`AUTHORED_INSTANCE`] for an SFC-inheriting component.
pub(crate) const INHERITED_AUTHORED_INSTANCE: &str = "$AuthoredTemplateInstance";

const MERGE_HELPER: &str = "  type $WithInheritedMembers<__O, __H> = { [__K in keyof __O as __K extends keyof __H ? (0 extends (1 & __O[__K]) ? never : __K) : __K]: __O[__K] } & { [__K in keyof __H as __K extends keyof __O ? (0 extends (1 & __O[__K]) ? __K : never) : __K]: __H[__K] };\n";

/// One `mixins:` element (or the `extends:` value) whose instance members
/// `$This` inherits.
#[derive(Debug)]
pub(crate) struct InheritedComponent {
    /// The authored reference, reproduced verbatim as a `typeof` operand.
    pub(crate) reference: String,
    /// Script-relative range of the authored expression it was derived from.
    pub(crate) src: Range<usize>,
}

/// Collect every `mixins:` element and the `extends:` value that can be named
/// by a `typeof` query.
///
/// An inline options object (`mixins: [{ methods: { … } }]`) is skipped here:
/// it has no name to query, and Croquis already merges its local members into
/// the direct `$This` shape.
pub(crate) fn collect_inherited_components(
    options: &ObjectExpression<'_>,
    inherited: &mut Vec<InheritedComponent>,
) {
    if let Some(mixins) = option_expression_property(options, "mixins")
        && let Expression::ArrayExpression(array) = unwrap_expression(mixins)
    {
        for element in &array.elements {
            if let Some(expression) = element.as_expression() {
                push_inherited(expression, inherited);
            }
        }
    }
    if let Some(extends) = option_expression_property(options, "extends") {
        push_inherited(extends, inherited);
    }
}

/// The `$InheritedMembers<typeof …>` intersection of the imported-SFC
/// `mixins:` / `extends:` references of `script`'s default export, or `None`.
pub(crate) fn authored_template_inherited(script: &str) -> Option<String> {
    if !script.contains("mixins") && !script.contains("extends") {
        return None;
    }
    vize_croquis::script_parser::with_shared_parse(script, SourceType::ts(), |parsed| {
        if parsed.panicked {
            return None;
        }
        let options = component_options_from_program(&parsed.program)?;
        let mut inherited = Vec::new();
        collect_inherited_components(options, &mut inherited);
        let sfc_locals = sfc_import_locals(&parsed.program);
        let mut members = String::default();
        for component in &inherited {
            let root = component.reference.split('.').next().unwrap_or_default();
            if !sfc_locals.contains(root) {
                continue;
            }
            if !members.is_empty() {
                members.push_str(" & ");
            }
            append!(members, "$InheritedMembers<typeof {}>", component.reference);
        }
        (!members.is_empty()).then_some(members)
    })
}

/// Declare [`INHERITED_AUTHORED_INSTANCE`] over `members`
/// ([`authored_template_inherited`]).
pub(crate) fn emit_authored_template_instance(ts: &mut crate::virtual_ts::TextBuf, members: &str) {
    ts.push_str("  // Members `mixins:` / `extends:` SFCs give the authored template instance.\n");
    ts.push_str(INHERITED_MEMBER_HELPERS);
    ts.push_str(MERGE_HELPER);
    append!(
        *ts,
        "  type {INHERITED_AUTHORED_INSTANCE} = $WithInheritedMembers<{AUTHORED_INSTANCE}, {members}>;\n  void (undefined as unknown as {INHERITED_AUTHORED_INSTANCE});\n"
    );
}

/// Local names bound by value imports of `.vue` files.
fn sfc_import_locals<'a>(program: &'a Program<'a>) -> FxHashSet<&'a str> {
    let mut locals = FxHashSet::default();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let source = import.source.value.as_str();
        let path = source.split('?').next().unwrap_or(source);
        if import.import_kind == ImportOrExportKind::Type || !path.ends_with(".vue") {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            locals.insert(specifier.local().name.as_str());
        }
    }
    locals
}

fn push_inherited(expression: &Expression<'_>, inherited: &mut Vec<InheritedComponent>) {
    let Some(reference) = reference_text(expression) else {
        return;
    };
    let span = expression.span();
    inherited.push(InheritedComponent {
        reference,
        src: span.start as usize..span.end as usize,
    });
}

/// The authored text of a static value reference (`base`, `shared.base`), or
/// `None` when the expression is not something `typeof` can name.
fn reference_text(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            is_safe_value_identifier(name).then(|| String::from(name))
        }
        Expression::StaticMemberExpression(member) => {
            let mut text = reference_text(&member.object)?;
            let property = member.property.name.as_str();
            if !is_safe_value_identifier(property) {
                return None;
            }
            append!(text, ".{property}");
            Some(text)
        }
        Expression::ParenthesizedExpression(value) => reference_text(&value.expression),
        Expression::TSAsExpression(value) => reference_text(&value.expression),
        Expression::TSSatisfiesExpression(value) => reference_text(&value.expression),
        Expression::TSNonNullExpression(value) => reference_text(&value.expression),
        _ => None,
    }
}

fn unwrap_expression<'a, 'b>(expression: &'a Expression<'b>) -> &'a Expression<'b> {
    match expression {
        Expression::ParenthesizedExpression(value) => unwrap_expression(&value.expression),
        Expression::TSAsExpression(value) => unwrap_expression(&value.expression),
        Expression::TSSatisfiesExpression(value) => unwrap_expression(&value.expression),
        Expression::TSNonNullExpression(value) => unwrap_expression(&value.expression),
        _ => expression,
    }
}
