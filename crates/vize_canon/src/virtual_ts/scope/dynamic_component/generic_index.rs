//! A generic key defers `map[key]`, which the component checks cannot read.
//! Index a mapped box instead: a property read resolves the key's constraint,
//! so `T extends 'a' | 'b'` selects exactly those entries, as `vue-tsc` does.
//! `-?` stops TypeScript substituting the key into the template, which would
//! defer the read again. A non-generic key keeps its exact entry type.
use oxc_ast::ast::Expression;
use oxc_span::{GetSpan, Span};
use vize_carton::String;
use vize_croquis::croquis::ComponentUsage;
use vize_relief::{ExpressionNode, RootNode};

const BOX: &str =
    "(<__M,>(__m: __M) => __m as unknown as { [__K in keyof __M]-?: { v: __M[__K] } })";

/// The authored `:is` text with every selected computed member boxed.
pub(super) fn value(root: &RootNode<'_>, usage: &ComponentUsage) -> Option<String> {
    let ExpressionNode::Simple(expression) = super::is_expression_node(root, usage.start)? else {
        return None;
    };
    let js = expression.js_ast?.as_expression()?;
    if js.raw != expression.content {
        return None;
    }
    let mut out = crate::virtual_ts::TextBuf::default();
    write(js.raw, js.ast, &mut out)?.then_some(out.into_compact())
}

/// Writes `ast`, returning whether any member was boxed.
fn write(raw: &str, ast: &Expression<'_>, out: &mut crate::virtual_ts::TextBuf) -> Option<bool> {
    Some(match ast {
        Expression::ComputedMemberExpression(member) if !member.optional => {
            out.push_str(BOX);
            out.push('(');
            write(raw, &member.object, out)?;
            out.push_str(")[");
            out.push_str(slice(raw, member.expression.span())?);
            out.push_str("].v");
            true
        }
        Expression::ParenthesizedExpression(inner) => {
            out.push('(');
            let boxed = write(raw, &inner.expression, out)?;
            out.push(')');
            boxed
        }
        Expression::ConditionalExpression(conditional) => {
            out.push_str(slice(raw, conditional.test.span())?);
            out.push_str(" ? ");
            let consequent = write(raw, &conditional.consequent, out)?;
            out.push_str(" : ");
            write(raw, &conditional.alternate, out)? || consequent
        }
        Expression::LogicalExpression(logical) => {
            let left = write(raw, &logical.left, out)?;
            out.push(' ');
            out.push_str(logical.operator.as_str());
            out.push(' ');
            write(raw, &logical.right, out)? || left
        }
        other => {
            out.push_str(slice(raw, other.span())?);
            false
        }
    })
}

fn slice(raw: &str, span: Span) -> Option<&str> {
    raw.get(span.start as usize..span.end as usize)
}

#[cfg(test)]
mod tests {
    use vize_armature::parse;
    use vize_carton::{Allocator, cstr};
    use vize_croquis::{Drawer, DrawerOptions};

    fn boxed(is: &str) -> Option<vize_carton::String> {
        let allocator = Allocator::new();
        let template = cstr!("<component :is=\"{is}\"></component>");
        let (root, errors) = parse(&allocator, template.as_str());
        assert!(errors.is_empty());
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_script_setup_with_generic(
            "import A from './A.vue'; const map = { a: A }; const foo = { type: 'a' as K };",
            Some("K extends 'a'"),
        );
        drawer.draw_template(&root);
        let summary = drawer.finish();
        assert_eq!(summary.component_usages.len(), 1);
        super::value(&root, &summary.component_usages[0])
    }

    #[test]
    fn boxes_selected_members_and_keeps_other_text() {
        let wrap = |object: &str, key: &str| cstr!("{}({object})[{key}].v", super::BOX);
        assert_eq!(
            boxed("map[foo.type]").as_deref(),
            Some(wrap("map", "foo.type").as_str())
        );
        assert_eq!(
            boxed("ok ? map[foo.type] : A").as_deref(),
            Some(cstr!("ok ? {} : A", wrap("map", "foo.type")).as_str())
        );
        assert_eq!(
            boxed("(map[foo.type] ?? A)").as_deref(),
            Some(cstr!("({} ?? A)", wrap("map", "foo.type")).as_str())
        );
        assert_eq!(boxed("map?.[foo.type]"), None);
        assert_eq!(boxed("ok ? A : map.a"), None);
    }
}
