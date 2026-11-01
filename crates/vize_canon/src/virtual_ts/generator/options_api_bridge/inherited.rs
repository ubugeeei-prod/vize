//! `mixins:` / `extends:` members incorporated into the Options API `$This`.
//!
//! Without this, `$This` only lists the component's own
//! `props`/`data`/`computed`/`methods`/`inject`/`setup` names, so a
//! `methods`/`computed` body that legitimately calls a member contributed by a
//! mixin reports `TS2339 Property 'x' does not exist on type '$This'` on
//! code `vue-tsc` accepts (#3609).
//!
//! Component constructors contribute their instance. A plain options-object
//! mixin can also contribute its finite, typed `methods` object, including
//! plugin-style dollar names. The method signatures remain intact, so an
//! invalid argument does not acquire an `any` receiver. Unknown, `any`, and
//! open-index method dictionaries contribute no keys: intersecting those into
//! `$This` would silently drop genuine missing-member findings.
//!
//! The neutral element is a named empty interface on purpose. TypeScript drops
//! `{}` / `unknown` while building the intersection and then prints the
//! expanded own-member object in diagnostics. Keeping a named neutral member
//! preserves the `$This` alias and the pinned legacy messages.

use std::ops::Range;

use oxc_ast::ast::{Expression, ObjectExpression};
use oxc_span::GetSpan;

use super::super::options_api::option_expression_property;
use super::super::options_api_support::is_safe_value_identifier;
use vize_carton::{String, append};

/// Type-level helpers backing the inherited-member aliases.
pub(super) const INHERITED_MEMBER_HELPERS: &str = concat!(
    "  type $InheritedInstance<T> = 0 extends (1 & T) ? any : T extends abstract new (...args: any[]) => infer __I ? __I : T extends { methods: infer __M } ? 0 extends (1 & __M) ? $NoInheritedMembers : [__M] extends [never] ? $NoInheritedMembers : __M extends object ? [keyof __M] extends [never] ? $NoInheritedMembers : string extends keyof __M ? $NoInheritedMembers : number extends keyof __M ? $NoInheritedMembers : symbol extends keyof __M ? $NoInheritedMembers : __M : $NoInheritedMembers : $NoInheritedMembers;\n",
    "  interface $NoInheritedMembers {}\n",
    "  type $InheritedMembers<T, __I = $InheritedInstance<T>> = [$IsAny<__I>] extends [true] ? $NoInheritedMembers : __I;\n",
);

/// One `mixins:` element (or the `extends:` value) whose instance members
/// `$This` inherits.
#[derive(Debug)]
pub(super) struct InheritedComponent {
    /// The authored reference, reproduced verbatim as a `typeof` operand.
    pub(super) reference: String,
    /// Script-relative range of the authored expression it was derived from.
    pub(super) src: Range<usize>,
}

/// Collect every `mixins:` element and the `extends:` value that can be named
/// by a `typeof` query.
///
/// An inline options object (`mixins: [{ methods: { … } }]`) is skipped here:
/// it has no name to query, and Croquis already merges its local members into
/// the direct `$This` shape.
pub(super) fn collect_inherited_components(
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

#[cfg(test)]
mod tests {
    use super::INHERITED_MEMBER_HELPERS;
    use std::{path::Path, process::Command};

    #[test]
    fn native_mixin_method_laws_preserve_opaque_and_constructor_types() {
        if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
            return;
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let provider = std::env::var_os("CORSA_PATH")
            .map(std::path::PathBuf::from)
            .or_else(|| vize_carton::corsa_resolver::discover_corsa_in_ancestors(root));
        let Some(provider) = provider else {
            assert!(
                std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
                "required native checker unavailable"
            );
            return;
        };
        assert!(provider.is_file(), "native checker must be a regular file");
        let project =
            std::env::temp_dir().join(format!("vize-legacy-method-laws-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&project);
        std::fs::create_dir_all(&project).unwrap();
        let source = format!(
            "{}\n{}\n{}",
            include_str!("../../helpers/declarations/core-types.d.ts.txt"),
            INHERITED_MEMBER_HELPERS,
            include_str!("inherited/guard_laws.ts.txt")
                .replace("__INHERITED_HELPERS__", INHERITED_MEMBER_HELPERS)
        );
        std::fs::write(project.join("laws.ts"), &source).unwrap();
        let output = Command::new(&provider)
            .current_dir(&project)
            .args([
                "--noEmit",
                "--strict",
                "--skipLibCheck",
                "--target",
                "ES2022",
                "laws.ts",
            ])
            .output()
            .expect("native helper law process");
        if let Some(capture) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
            let capture =
                std::path::PathBuf::from(capture).join("legacy-typed-mixin-methods/helper-laws");
            std::fs::create_dir_all(&capture).unwrap();
            std::fs::write(capture.join("laws.ts"), source).unwrap();
            std::fs::write(capture.join("stdout.bin"), &output.stdout).unwrap();
            std::fs::write(capture.join("stderr.bin"), &output.stderr).unwrap();
            std::fs::write(
                capture.join("status.json"),
                serde_json::to_vec_pretty(
                    &serde_json::json!({"status": output.status.code(), "provider": provider}),
                )
                .unwrap(),
            )
            .unwrap();
        }
        let _ = std::fs::remove_dir_all(project);
        assert_eq!(
            output.status.code(),
            Some(0),
            "stdout={:?} stderr={:?}",
            std::str::from_utf8(&output.stdout).unwrap(),
            std::str::from_utf8(&output.stderr).unwrap()
        );
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
    }
}
