//! `withDefaults(defineProps<T>(), <expression>)` whose defaults cannot be read
//! from an object literal.
//!
//! Vue types `withDefaults` from the *type* of its second argument, so a
//! defaults object produced by a call, a conditional or a spread still makes
//! those props non-`undefined` in the template. Their names are not in the
//! source, and a resolver that proves some of them (the literal keys beside a
//! spread, say) cannot prove the rest. The template contract is therefore the
//! type of the same macro call repeated in a setup-scope capture, which is how
//! `vue-tsc` types the template: from the macro call itself.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectPropertyKind};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, append, cstr};
use vize_croquis::Croquis;

/// Setup-scope binding typed by the macro call over the authored argument.
const CAPTURE: &str = "__vize_with_defaults_props";

/// The authored defaults argument when its keys are opaque to static analysis.
fn opaque_defaults_expression(summary: &Croquis) -> Option<&str> {
    summary.macros.define_props()?.type_args.as_ref()?;
    let expression = summary.macros.with_defaults_expression()?;
    is_opaque(expression).then_some(expression)
}

fn is_opaque(expression: &str) -> bool {
    let source = cstr!("({expression})");
    let allocator = Allocator::default();
    let Ok(parsed) = Parser::new(&allocator, source.as_str(), SourceType::ts()).parse_expression()
    else {
        return false;
    };
    match parsed.get_inner_expression() {
        Expression::ObjectExpression(object) => object
            .properties
            .iter()
            .any(|property| matches!(property, ObjectPropertyKind::SpreadProperty(_))),
        _ => true,
    }
}

/// The template's prop contract when the defaults are opaque: the type the
/// macro call itself resolves to.
pub(super) fn opaque_defaults_props_type_ref(summary: &Croquis) -> Option<String> {
    opaque_defaults_expression(summary).map(|_| cstr!("typeof {CAPTURE}"))
}

/// Emit the setup-scope capture the template contract reads its type from.
///
/// The argument may only reference module-scope bindings, so repeating the
/// macro call after the setup body resolves exactly as the authored one does.
pub(super) fn generate_opaque_defaults_capture(
    ts: &mut String,
    summary: &Croquis,
    authored_type_ref: &str,
) {
    let Some(expression) = opaque_defaults_expression(summary) else {
        return;
    };
    ts.push_str(
        "\n  // `withDefaults` over opaque defaults; its type is the template prop contract\n",
    );
    // An invalid argument is reported once, on the authored call.
    ts.push_str(
        "  // @ts-ignore TS2345: the authored withDefaults call reports an invalid argument.\n",
    );
    append!(
        *ts,
        "  const {CAPTURE} = __vize_withDefaults(__vize_defineProps<{authored_type_ref}>(), ({expression}));\n"
    );
    append!(*ts, "  void {CAPTURE};\n");
}

#[cfg(test)]
mod tests {
    use super::is_opaque;

    #[test]
    fn object_literals_with_static_keys_are_not_opaque() {
        assert!(!is_opaque("{ tone: 'light', size: 1 }"));
        assert!(!is_opaque("({ tone: 'light' })"));
        assert!(!is_opaque("{ tone: 'light' } as const"));
        assert!(!is_opaque("{}"));
    }

    #[test]
    fn computed_defaults_are_opaque() {
        assert!(is_opaque("treeDefaults()"));
        assert!(is_opaque("defaults"));
        assert!(is_opaque("shared.defaults"));
        assert!(is_opaque("compact ? dense : relaxed"));
        assert!(is_opaque("{ ...treeDefaults(), tone: 'light' }"));
    }

    #[test]
    fn unparsable_arguments_keep_the_keyed_contract() {
        assert!(!is_opaque("{ tone: "));
    }
}
