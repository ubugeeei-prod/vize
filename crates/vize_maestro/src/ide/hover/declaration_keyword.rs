//! Align the leading declaration keyword of a Corsa quick-info hover with the
//! authored source (#3894).
//!
//! Template bindings are synthesized as `var {name}: T = undefined as any` in
//! the virtual TypeScript, so the checker's quick info opens with `var ` even
//! when the authored declaration is a `const` destructuring or a `let`. The
//! keyword is the first thing a hover shows; leaking the synthesis detail
//! misstates mutability. The rewrite consults the authored `<script setup>`
//! text and only ever replaces a leading `var {word}` whose word has exactly
//! one known top-level declaration kind — anything else keeps the checker's
//! answer.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Declaration, Statement, VariableDeclarationKind};
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::{Hover, HoverContents};

use crate::ide::IdeContext;

/// Replace a leading synthesized `var` in a Corsa hover with the authored
/// declaration keyword for the hovered template binding, and a leading
/// `(parameter)` with `const` when the word is a `v-for` alias: the virtual
/// TS lowers `v-for` to a callback, but the authored construct is an
/// immutable per-iteration binding, and Volar's for-of lowering presents it
/// as `const` (#3894).
pub(super) fn align_hover(ctx: &IdeContext<'_>, word: &str, hover: &mut Hover) {
    let HoverContents::Markup(ref mut markup) = hover.contents else {
        return;
    };
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
        return;
    };
    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        let lang = script_setup.lang.as_deref();
        if let Some(aligned) = align_leading_var(&markup.value, &script_setup.content, lang, word) {
            markup.value = aligned;
            return;
        }
    }
    if let Some(template) = descriptor.template.as_ref()
        && ctx.offset >= template.loc.start
        && ctx.offset < template.loc.end
        && let Ok(offset) = u32::try_from(ctx.offset - template.loc.start)
        && let Some(aligned) = align_v_for_parameter(&markup.value, &template.content, word, offset)
    {
        markup.value = aligned;
    }
}

/// Rewrite a quick-info block opening with `(parameter) {word}` to
/// `const {word}` when the hovered position resolves to a `v-for` value, key,
/// or index alias. `offset` is the hovered position relative to the template
/// block's content. Anything else keeps the checker's answer.
fn align_v_for_parameter(
    markdown: &str,
    template: &str,
    word: &str,
    offset: u32,
) -> Option<String> {
    if word.is_empty() {
        return None;
    }
    let fence_start = markdown.find("```")?;
    let after_fence = markdown[fence_start..].find('\n')? + fence_start + 1;
    let synthesized = format!("(parameter) {word}");
    if !markdown[after_fence..].starts_with(&synthesized) {
        return None;
    }
    let following = markdown[after_fence + synthesized.len()..].chars().next();
    if following.is_some_and(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric()) {
        return None;
    }
    if !is_v_for_alias(template, word, offset) {
        return None;
    }
    let mut rewritten = String::with_capacity(markdown.len());
    rewritten.push_str(&markdown[..after_fence]);
    rewritten.push_str("const");
    rewritten.push_str(&markdown[after_fence + "(parameter)".len()..]);
    Some(rewritten)
}

/// Whether `word` resolves to a `v-for` alias (value binding, key alias, or
/// index alias) at `offset`, resolved through the same analyzer the generator
/// uses rather than a textual scan.
///
/// The binding is looked up by position, not by name: a name is only an alias
/// where the enclosing `v-for` is in effect, and an inner scope may shadow it.
/// `bindings_visible_at` walks outward from the innermost scope containing the
/// offset and lets the first declaration of a name win, so an event handler's
/// or callback's parameter named like the alias — inside the loop element or
/// anywhere else in the template — keeps the checker's `(parameter)` answer.
fn is_v_for_alias(template: &str, word: &str, offset: u32) -> bool {
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    summary
        .scopes
        .bindings_visible_at(offset)
        .into_iter()
        .find(|(name, _, _)| *name == word)
        .is_some_and(|(_, _, kind)| kind == vize_croquis::ScopeKind::VFor)
}

/// The oxc source type for a `<script setup lang="…">` block. `tsx`/`jsx` need
/// the JSX variant, or a JSX initializer derails the parse and the synthesized
/// `var` survives. Everything else parses as TypeScript, a superset of the
/// plain-JS default that only has to yield declaration kinds and patterns here.
fn script_source_type(lang: Option<&str>) -> SourceType {
    let lang = lang.unwrap_or_default().trim();
    if lang.eq_ignore_ascii_case("tsx") {
        SourceType::tsx()
    } else if lang.eq_ignore_ascii_case("jsx") {
        SourceType::jsx()
    } else {
        SourceType::ts()
    }
}

/// The authored declaration keyword for `word`, when its top-level
/// `<script setup>` declaration is a `const` or `let` binding (including
/// destructuring patterns and default values).
pub(super) fn authored_keyword(
    script_setup: &str,
    lang: Option<&str>,
    word: &str,
) -> Option<&'static str> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script_setup, script_source_type(lang)).parse();
    let mut found: Option<&'static str> = None;
    for statement in &parsed.program.body {
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => declaration,
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::VariableDeclaration(declaration)) => declaration,
                _ => continue,
            },
            _ => continue,
        };
        let keyword = match declaration.kind {
            VariableDeclarationKind::Const => "const",
            VariableDeclarationKind::Let => "let",
            _ => continue,
        };
        for declarator in &declaration.declarations {
            if pattern_binds(&declarator.id, word) {
                match found {
                    None => found = Some(keyword),
                    Some(existing) if existing == keyword => {}
                    // Two top-level declarations of one name is invalid code;
                    // do not guess which one the checker resolved.
                    Some(_) => return None,
                }
            }
        }
    }
    found
}

fn pattern_binds(pattern: &BindingPattern<'_>, word: &str) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => identifier.name == word,
        BindingPattern::ObjectPattern(object) => {
            object
                .properties
                .iter()
                .any(|property| pattern_binds(&property.value, word))
                || object
                    .rest
                    .as_ref()
                    .is_some_and(|rest| pattern_binds(&rest.argument, word))
        }
        BindingPattern::ArrayPattern(array) => {
            array
                .elements
                .iter()
                .flatten()
                .any(|element| pattern_binds(element, word))
                || array
                    .rest
                    .as_ref()
                    .is_some_and(|rest| pattern_binds(&rest.argument, word))
        }
        BindingPattern::AssignmentPattern(assignment) => pattern_binds(&assignment.left, word),
    }
}

/// Rewrite a quick-info markdown block whose code fence opens with
/// `var {word}` to the authored keyword. Returns `None` when nothing applies.
pub(super) fn align_leading_var(
    markdown: &str,
    script_setup: &str,
    lang: Option<&str>,
    word: &str,
) -> Option<String> {
    if word.is_empty() {
        return None;
    }
    let fence_start = markdown.find("```")?;
    let after_fence = markdown[fence_start..].find('\n')? + fence_start + 1;
    let synthesized = format!("var {word}");
    if !markdown[after_fence..].starts_with(&synthesized) {
        return None;
    }
    let following = markdown[after_fence + synthesized.len()..].chars().next();
    // Only the whole identifier: `var counter` must not match hover on `count`.
    if following.is_some_and(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric()) {
        return None;
    }
    let keyword = authored_keyword(script_setup, lang, word)?;
    let mut rewritten = String::with_capacity(markdown.len());
    rewritten.push_str(&markdown[..after_fence]);
    rewritten.push_str(keyword);
    rewritten.push_str(&markdown[after_fence + "var".len()..]);
    Some(rewritten)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_destructured_const_rewrites_the_var_prefix() {
        let script = "const { errors, meta } = useForm()\nlet attempts = 0\n";
        let hover = "```typescript\nvar errors: Ref<string[]>\n```";
        assert_eq!(
            super::align_leading_var(hover, script, Some("ts"), "errors").as_deref(),
            Some("```typescript\nconst errors: Ref<string[]>\n```")
        );
        let hover = "```typescript\nvar attempts: number\n```";
        assert_eq!(
            super::align_leading_var(hover, script, Some("ts"), "attempts").as_deref(),
            Some("```typescript\nlet attempts: number\n```")
        );
    }

    #[test]
    fn unknown_names_prefix_words_and_non_var_hovers_stay_untouched() {
        let script = "const counter = 1\n";
        // Hover text names a longer identifier than the hovered word.
        let hover = "```typescript\nvar counter: number\n```";
        assert_eq!(
            super::align_leading_var(hover, script, Some("ts"), "count"),
            None
        );
        // The word has no top-level declaration.
        assert_eq!(
            super::align_leading_var(hover, script, Some("ts"), "missing"),
            None
        );
        // The quick info does not open with `var`.
        let hover = "```typescript\nconst counter: 1\n```";
        assert_eq!(
            super::align_leading_var(hover, script, Some("ts"), "counter"),
            None
        );
    }

    #[test]
    fn a_jsx_initializer_resolves_under_tsx_and_jsx_langs() {
        // `<script setup lang="tsx">`: parsed as TypeScript, `<Badge />` reads as
        // a type assertion and the declaration never resolves.
        let script = "const badge = <Badge count={1} />\n";
        let hover = "```typescript\nvar badge: JSX.Element\n```";
        assert_eq!(
            super::align_leading_var(hover, script, Some("tsx"), "badge").as_deref(),
            Some("```typescript\nconst badge: JSX.Element\n```")
        );
        assert_eq!(
            super::align_leading_var(hover, script, Some("jsx"), "badge").as_deref(),
            Some("```typescript\nconst badge: JSX.Element\n```")
        );
        // A JSX statement ahead of the hovered `let` must not derail the parse.
        let script = "const badge = <Badge count={1} />\nlet attempts = 0\n";
        let hover = "```typescript\nvar attempts: number\n```";
        assert_eq!(
            super::align_leading_var(hover, script, Some("tsx"), "attempts").as_deref(),
            Some("```typescript\nlet attempts: number\n```")
        );
    }

    /// The offset of `needle`'s nth occurrence, as a template-relative
    /// hover position.
    fn offset_of(template: &str, needle: &str, nth: usize) -> u32 {
        template.match_indices(needle).nth(nth).unwrap().0 as u32
    }

    #[test]
    fn a_v_for_alias_parameter_rewrites_to_const() {
        let template =
            "<ul><li v-for=\"(it, index) in users\" :key=\"it.id\">{{ it.name }}</li></ul>";
        // The alias at a use site and at its own declaration.
        let hover = "```typescript\n(parameter) it: User\n```";
        assert_eq!(
            super::align_v_for_parameter(hover, template, "it", offset_of(template, "it.name", 0))
                .as_deref(),
            Some("```typescript\nconst it: User\n```")
        );
        let hover = "```typescript\n(parameter) index: number\n```";
        assert_eq!(
            super::align_v_for_parameter(hover, template, "index", offset_of(template, "index", 0))
                .as_deref(),
            Some("```typescript\nconst index: number\n```")
        );
        // A non-alias parameter (an event handler's) keeps the checker's answer.
        let hover = "```typescript\n(parameter) event: MouseEvent\n```";
        assert_eq!(
            super::align_v_for_parameter(hover, template, "event", offset_of(template, "users", 0)),
            None
        );
    }

    #[test]
    fn a_parameter_named_like_an_alias_keeps_the_checker_answer() {
        // The handler parameter shadows the alias inside the loop element, and
        // names it again outside the loop entirely.
        let template = concat!(
            "<ul><li v-for=\"item in items\" @click=\"(item) => item.id\">{{ item.name }}</li></ul>",
            "<button @click=\"(item) => item.id\">x</button>"
        );
        let hover = "```typescript\n(parameter) item: MouseEvent\n```";
        // Shadowing parameter inside the v-for element: the handler's binding wins.
        assert_eq!(
            super::align_v_for_parameter(
                hover,
                template,
                "item",
                offset_of(template, "item.id", 0)
            ),
            None
        );
        // Same name in a handler outside the loop: the alias is not in effect.
        assert_eq!(
            super::align_v_for_parameter(
                hover,
                template,
                "item",
                offset_of(template, "item.id", 1)
            ),
            None
        );
        // The alias itself still resolves inside the loop body.
        let hover = "```typescript\n(parameter) item: Item\n```";
        assert_eq!(
            super::align_v_for_parameter(
                hover,
                template,
                "item",
                offset_of(template, "item.name", 0)
            )
            .as_deref(),
            Some("```typescript\nconst item: Item\n```")
        );
    }
}
