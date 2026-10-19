//! Change only a proven authored slot name; retain its argument and value.

use super::VSlotStyleOption;
use crate::diagnostic::{Fix, TextEdit};
use vize_relief::{DirectiveNode, ExpressionNode};

pub(super) fn slot_fix(
    source: &str,
    directive: &DirectiveNode<'_>,
    actual: VSlotStyleOption,
    expected: VSlotStyleOption,
    help: &str,
) -> Option<Fix> {
    if directive.name != "slot" || !directive.modifiers.is_empty() || actual == expected {
        return None;
    }
    let start = directive.loc.span.start;
    let end = directive.loc.span.end;
    let attribute = source.get(start as usize..end as usize)?;
    // Relief retains the directive prefix, not the full authored argument.
    let (prefix, prefix_text) = match actual {
        VSlotStyleOption::Shorthand if directive.raw_name == Some("#") => (1, "#"),
        VSlotStyleOption::Longform if directive.raw_name == Some("v-slot") => (7, "v-slot:"),
        VSlotStyleOption::VSlot if directive.raw_name == Some("v-slot") => (6, "v-slot"),
        _ => return None,
    };
    if !attribute.starts_with(prefix_text) {
        return None;
    }
    let (name_end, is_default) = match (&directive.arg, actual) {
        (None, VSlotStyleOption::VSlot) => (start.checked_add(prefix)?, true),
        (
            Some(ExpressionNode::Simple(simple)),
            VSlotStyleOption::Shorthand | VSlotStyleOption::Longform,
        ) => {
            if simple.content.is_empty() {
                return None;
            }
            let bracket = u32::from(!simple.is_static);
            let argument = simple.loc.span;
            if argument.start != start.checked_add(prefix)?.checked_add(bracket)?
                || source.get(argument.start as usize..argument.end as usize)? != simple.content
            {
                return None;
            }
            if !simple.is_static
                && (source.as_bytes().get((argument.start - 1) as usize) != Some(&b'[')
                    || source.as_bytes().get(argument.end as usize) != Some(&b']'))
            {
                return None;
            }
            (
                argument.end.checked_add(bracket)?,
                simple.is_static && simple.content == "default",
            )
        }
        _ => return None,
    };
    let value = source.get(name_end as usize..end as usize)?;
    if start
        .checked_sub(1)
        .and_then(|offset| source.as_bytes().get(offset as usize))
        .is_some_and(|previous| !separator(*previous))
        || source
            .as_bytes()
            .get(end as usize)
            .is_some_and(|next| !separator(*next))
        || (!value.is_empty()
            && !value
                .trim_start_matches([' ', '\t', '\r', '\n', '\u{c}'])
                .starts_with('='))
    {
        return None;
    }
    let (width, replacement) = match expected {
        VSlotStyleOption::Shorthand if directive.arg.is_some() => (prefix, "#"),
        VSlotStyleOption::Shorthand => (prefix, "#default"),
        VSlotStyleOption::Longform if directive.arg.is_some() => (prefix, "v-slot:"),
        VSlotStyleOption::Longform => (prefix, "v-slot:default"),
        // Dropping a named or dynamic argument would select a different slot.
        VSlotStyleOption::VSlot if is_default => (name_end - start, "v-slot"),
        _ => return None,
    };
    Some(Fix::new(
        help,
        TextEdit::replace(start, start + width, replacement),
    ))
}

fn separator(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 12 | b'/' | b'>')
}

#[cfg(test)]
mod tests {
    //! Complete configured diagnostics plus refusal of unproven source intervals.

    use super::super::{VSlotStyle, VSlotStyleOption};
    use super::slot_fix;
    use crate::{Fix, HelpLevel, LintDiagnostic, Linter, TextEdit, rule::RuleRegistry};
    use vize_l0::{Allocator, Box as ArenaBox, cstr};
    use vize_relief::{DirectiveNode, ExpressionNode, SimpleExpressionNode, SourceLocation};

    const HELP: &str = "**Shorthand:** `<template #header>` (default)\n**Longform:** `<template v-slot:header>`\n\nChoose one style and be consistent.";

    #[test]
    fn configured_styles_retain_complete_diagnostics_and_slot_identity() {
        use VSlotStyleOption::{Longform, Shorthand, VSlot};
        let cases = [
            (
                Longform,
                "<Card><template #header=\"props\">H</template></Card>",
                "#header=\"props\"",
                "Expected 'v-slot:header' instead of '#header'",
                Some((1, "v-slot:")),
                "<Card><template v-slot:header=\"props\">H</template></Card>",
            ),
            (
                Longform,
                "<Card v-slot=\"{ item }\">H</Card>",
                "v-slot=\"{ item }\"",
                "Expected 'v-slot:default' instead of 'v-slot'",
                Some((6, "v-slot:default")),
                "<Card v-slot:default=\"{ item }\">H</Card>",
            ),
            (
                Shorthand,
                "<Card v-slot='props'>H</Card>",
                "v-slot='props'",
                "Expected '#default' instead of 'v-slot'",
                Some((6, "#default")),
                "<Card #default='props'>H</Card>",
            ),
            (
                VSlot,
                "<Card><template #default=\"props\">H</template></Card>",
                "#default=\"props\"",
                "Expected 'v-slot' instead of '#default'",
                Some((8, "v-slot")),
                "<Card><template v-slot=\"props\">H</template></Card>",
            ),
            (
                VSlot,
                "<Card><template #header=\"props\">H</template></Card>",
                "#header=\"props\"",
                "Expected 'v-slot' instead of '#header'",
                None,
                "<Card><template #header=\"props\">H</template></Card>",
            ),
            (
                VSlot,
                "<Card><template #[name]=\"props\">H</template></Card>",
                "#[name]=\"props\"",
                "Expected 'v-slot' instead of '#[name]'",
                None,
                "<Card><template #[name]=\"props\">H</template></Card>",
            ),
            (
                Longform,
                "<Card><template #header.foo>H</template></Card>",
                "#header.foo",
                "Expected 'v-slot:header.foo' instead of '#header.foo'",
                None,
                "<Card><template #header.foo>H</template></Card>",
            ),
        ];
        for (style, source, target, message, edit, fixed) in cases {
            let mut registry = RuleRegistry::new();
            registry.register(Box::new(VSlotStyle::uniform(style)));
            let checker = Linter::with_registry(registry).with_help_level(HelpLevel::Full);
            let actual = checker.lint_template(source, "Style.vue");
            let start = source.find(target).unwrap() as u32;
            let mut expected = LintDiagnostic::warn(
                "vue/v-slot-style",
                message,
                start,
                start + target.len() as u32,
            )
            .with_help(HELP);
            if let Some((width, replacement)) = edit {
                expected = expected.with_fix(Fix::new(
                    HELP,
                    TextEdit::replace(start, start + width, replacement),
                ));
            }
            assert_eq!(
                cstr!("{:?}", actual.diagnostics),
                cstr!("{:?}", vec![expected]),
                "{source}"
            );
            assert_eq!((actual.error_count, actual.warning_count), (0, 1));
            assert_eq!(actual.filename.as_str(), "Style.vue");
            let edits = actual
                .diagnostics
                .iter()
                .filter_map(|d| d.fix.as_ref())
                .flat_map(|fix| fix.edits.iter().cloned())
                .collect();
            assert_eq!(
                Fix::with_edits("authored", edits).apply(source).as_str(),
                fixed
            );
            let after = checker.lint_template(fixed, "Style.vue");
            if edit.is_some() {
                assert_eq!(cstr!("{:?}", after.diagnostics), "[]");
                assert_eq!((after.error_count, after.warning_count), (0, 0));
            } else {
                assert_eq!(cstr!("{after:?}"), cstr!("{actual:?}"));
            }
        }
    }

    #[test]
    fn unsafe_spans_and_recovered_heads_never_receive_an_edit() {
        use VSlotStyleOption::{Longform, Shorthand};
        let allocator = Allocator::new();
        let source = " v-slot:header=\"props\" ";
        let mut directive = DirectiveNode::new(&allocator, "slot", SourceLocation::new(1, 22));
        directive.raw_name = Some("v-slot");
        directive.arg = Some(ExpressionNode::Simple(ArenaBox::new_in(
            SimpleExpressionNode::new("header", true, SourceLocation::new(8, 14)),
            &&allocator,
        )));
        let fix = slot_fix(source, &directive, Longform, Shorthand, HELP).unwrap();
        assert_eq!(
            cstr!("{fix:?}"),
            cstr!("{:?}", Fix::new(HELP, TextEdit::replace(1, 8, "#")))
        );
        assert_eq!(fix.apply(source).as_str(), " #header=\"props\" ");
        for (raw, start, end) in [
            (None, 1, 22),
            (Some("#"), 1, 22),
            (Some("v-slot"), 2, 22),
            (Some("v-slot"), 1, 100),
            (Some("v-slot"), 1, 13),
        ] {
            directive.raw_name = raw;
            directive.loc = SourceLocation::new(start, end);
            assert!(slot_fix(source, &directive, Longform, Shorthand, HELP).is_none());
        }
        directive.raw_name = Some("v-slot");
        directive.loc = SourceLocation::new(1, 22);
        assert!(
            slot_fix(
                "xv-slot:header=\"props\" ",
                &directive,
                Longform,
                Shorthand,
                HELP
            )
            .is_none()
        );
        assert!(
            slot_fix(
                " v-slot:header=\"props\"x",
                &directive,
                Longform,
                Shorthand,
                HELP
            )
            .is_none()
        );
        directive.arg = Some(ExpressionNode::Simple(ArenaBox::new_in(
            SimpleExpressionNode::new("name", false, SourceLocation::new(9, 13)),
            &&allocator,
        )));
        let malformed = " v-slot:[name=\"props\" ";
        directive.loc = SourceLocation::new(1, malformed.len() as u32 - 1);
        assert!(slot_fix(malformed, &directive, Longform, Shorthand, HELP).is_none());
        directive.loc = SourceLocation::new(0, 6);
        directive.arg = None;
        assert_eq!(
            slot_fix(
                "v-slot",
                &directive,
                VSlotStyleOption::VSlot,
                Shorthand,
                HELP
            )
            .unwrap()
            .apply("v-slot")
            .as_str(),
            "#default"
        );
    }
}
