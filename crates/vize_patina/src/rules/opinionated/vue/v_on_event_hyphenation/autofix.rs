//! Opt-in static argument edits, following eslint-plugin-vue10.9.2's converters.

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, TextEdit};
use vize_l0::CompactString;
use vize_relief::SourceLocation;

pub(super) fn report(
    ctx: &mut LintContext<'_>,
    directive: &SourceLocation,
    argument: &SourceLocation,
    message: impl Into<CompactString>,
    help: impl Into<CompactString>,
    replacement: CompactString,
    enabled: bool,
) {
    if !enabled {
        ctx.warn_with_help(message, directive, help);
        return;
    }
    let help = help.into();
    let mut diagnostic = LintDiagnostic::warn(
        ctx.current_rule,
        message,
        directive.span.start,
        directive.span.end,
    );
    if let Some(processed) = ctx.help_level().process(help.as_str()) {
        diagnostic = diagnostic.with_help(processed);
    }
    ctx.report(diagnostic.with_fix(Fix::new(
        help,
        TextEdit::replace(argument.span.start, argument.span.end, replacement),
    )));
}

/// Only called for reported `never` names, which contain `-` and therefore
/// cannot enter upstream camelCase's PascalCase shortcut. Its remaining
/// /[-_](\w)/gu replacement uses ASCII word characters, not Unicode camelizing.
pub(super) fn camel_case(name: &str) -> CompactString {
    debug_assert!(name.contains('-'));
    let mut characters = name.chars().peekable();
    let mut output = CompactString::with_capacity(name.len());
    while let Some(character) = characters.next() {
        if matches!(character, '-' | '_')
            && characters
                .peek()
                .is_some_and(|next| next.is_ascii_alphanumeric() || *next == '_')
        {
            output.push(characters.next().unwrap().to_ascii_uppercase());
        } else {
            output.push(character);
        }
    }
    output
}
