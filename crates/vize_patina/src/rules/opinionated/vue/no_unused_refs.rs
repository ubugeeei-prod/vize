//! vue/no-unused-refs
//!
//! Report template refs (`ref="x"`) that are never referenced from `<script>`.
//!
//! In Vue, a template ref attribute `ref="x"` only does something if the script
//! reads it back — through a `<script setup>` variable named `x`, a
//! `useTemplateRef('x')` call, or `this.$refs.x` / `$refs.x` access. A `ref="x"`
//! the script never touches is dead markup that should be removed.
//!
//! ## Scope
//!
//! Patina correlates each static `ref="x"` with the already-built SFC semantic
//! analysis and the shared script ASTs. Script-setup bindings and values exposed
//! from an Options API `setup()` return count as uses, as do
//! `useTemplateRef('x')` and static `$refs.x` reads. Looking at
//! syntax and bindings instead of raw source text is important: an identifier in
//! a comment, documentation string, CSS selector, or an unrelated Options API
//! local must not keep a dead template ref alive.
//!
//! To stay sound in the few cases the text test cannot resolve, the rule reports
//! nothing for the file when:
//!
//! - a script or the template uses `$refs` opaquely — any computed access
//!   (including `$refs['x']`), or `$refs` passed / destructured / iterated as a
//!   whole object — since any ref could then be reached without a proven static
//!   member read.
//!
//! Dynamic ref bindings (`:ref` / `v-bind:ref`) are skipped: their target is an
//! expression, not a named template ref.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template><input ref="unused" /></template>
//! <script setup>
//! const x = 1
//! </script>
//! ```
//!
//! ### Valid
//! ```vue
//! <template><input ref="inputEl" /></template>
//! <script setup>
//! import { ref } from 'vue'
//! const inputEl = ref(null)
//! </script>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ComputedMemberExpression, Declaration, Expression,
    ForInStatement, ImportDeclarationSpecifier, ObjectExpression, ObjectPropertyKind, Program,
    PropertyKey, ReturnStatement, Statement, StaticMemberExpression, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::SemanticBuilder;
use vize_l0::{CompactString, cstr};
use vize_relief::{
    BindingType, ElementNode, PropNode, RootNode, SourceLocation, TemplateChildNode,
};

use crate::rules::script::vue_object::VueObjects;

static META: RuleMeta = RuleMeta {
    name: "vue/no-unused-refs",
    description: "Report template refs (ref=\"x\") never referenced in <script>",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Report template refs never referenced from script.
#[derive(Default)]
pub struct NoUnusedRefs;

/// A static template ref declaration collected from the template.
struct TemplateRef {
    name: CompactString,
    loc: SourceLocation,
}

impl Rule for NoUnusedRefs {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let Some(descriptor) = ctx.sfc_descriptor() else {
            return;
        };
        let Some(template) = descriptor.template.as_ref() else {
            return;
        };
        let Some(name) = template.attrs.get("ref").filter(|name| !name.is_empty()) else {
            return;
        };
        let template_source = template.content.as_ref();
        let scripts = ctx.sfc_scripts();
        let used_in_script = scripts.is_some_and(|scripts| {
            scripts.script().is_some_and(|parsed| {
                program_uses_template_ref(
                    &parsed.program,
                    descriptor
                        .script
                        .as_ref()
                        .map(|block| block.content.as_ref())
                        .unwrap_or(""),
                    true,
                    name,
                )
            }) || scripts.script_setup().is_some_and(|parsed| {
                program_uses_template_ref(
                    &parsed.program,
                    descriptor
                        .script_setup
                        .as_ref()
                        .map(|block| block.content.as_ref())
                        .unwrap_or(""),
                    false,
                    name,
                )
            })
        });
        if has_opaque_refs_access(template_source)
            || template_uses_static_ref(template_source, name)
            || binding_uses_template_ref(ctx, name)
            || used_in_script
        {
            return;
        }
        let Some((start, end)) = sfc_attribute_value_span(
            ctx.source,
            template.loc.tag_start,
            template.loc.start,
            "ref",
        ) else {
            return;
        };
        let loc = SourceLocation::new(start, end);
        ctx.warn_with_help(
            cstr!("Template ref '{name}' is never referenced in <script>"),
            &loc,
            cstr!(
                "Remove the unused ref=\"{name}\" attribute, or reference it in <script> \
                 (e.g. a ref named {name}, useTemplateRef('{name}'), or this.$refs.{name})."
            ),
        );
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Collect every static `ref="x"` in the template up front.
        let mut refs: Vec<TemplateRef> = Vec::new();
        collect_template_refs(&root.children, ctx.source, &mut refs);
        if refs.is_empty() {
            return;
        }
        // An active SFC rule normally receives semantic analysis. Its absence
        // here means the template had a fatal parse error, so the recovered
        // partial tree cannot prove that a ref declaration is really unused.
        if ctx.analysis().is_none() {
            return;
        }

        // Resolve which refs are unused while the descriptor borrow is held, then
        // drop it so diagnostics can be reported through `&mut ctx`.
        let unused: Vec<TemplateRef> = {
            let Some(descriptor) = ctx.sfc_descriptor() else {
                return;
            };
            let script = descriptor
                .script
                .as_ref()
                .map(|block| block.content.as_ref())
                .unwrap_or("");
            let script_setup = descriptor
                .script_setup
                .as_ref()
                .map(|block| block.content.as_ref())
                .unwrap_or("");
            // A dynamic or whole-object `$refs` use in the template can reach
            // any name. Static member reads, however, only keep that exact ref
            // alive; one `$refs.calendar` must not hide an unrelated dead
            // `ref="inlineCalendar"`.
            let template_src = descriptor
                .template
                .as_ref()
                .map(|block| block.content.as_ref())
                .unwrap_or("");
            if has_opaque_refs_access(template_src) {
                return;
            }

            let scripts = ctx.sfc_scripts();
            refs.into_iter()
                .filter(|template_ref| {
                    let name = template_ref.name.as_str();
                    !template_uses_static_ref(template_src, name)
                        && !binding_uses_template_ref(ctx, name)
                        && !scripts.is_some_and(|scripts| {
                            scripts.script().is_some_and(|parsed| {
                                program_uses_template_ref(&parsed.program, script, true, name)
                            }) || scripts.script_setup().is_some_and(|parsed| {
                                program_uses_template_ref(
                                    &parsed.program,
                                    script_setup,
                                    false,
                                    name,
                                )
                            })
                        })
                })
                .collect()
        };

        for template_ref in &unused {
            let name = template_ref.name.as_str();
            ctx.warn_with_help(
                cstr!("Template ref '{name}' is never referenced in <script>"),
                &template_ref.loc,
                cstr!(
                    "Remove the unused ref=\"{name}\" attribute, or reference it in <script> \
                     (e.g. a ref named {name}, useTemplateRef('{name}'), or this.$refs.{name})."
                ),
            );
        }
    }
}

fn sfc_attribute_value_span(
    source: &str,
    start: usize,
    end: usize,
    target: &str,
) -> Option<(u32, u32)> {
    let header = source.get(start..end)?.as_bytes();
    let mut cursor = 1;
    while header
        .get(cursor)
        .is_some_and(|&byte| !byte.is_ascii_whitespace() && !matches!(byte, b'>' | b'/'))
    {
        cursor += 1;
    }
    while cursor < header.len() {
        while header.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if header
            .get(cursor)
            .is_none_or(|&byte| matches!(byte, b'>' | b'/'))
        {
            break;
        }
        let name_start = cursor;
        while header
            .get(cursor)
            .is_some_and(|&byte| !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'>' | b'/'))
        {
            cursor += 1;
        }
        let name_end = cursor;
        while header.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if header.get(cursor) != Some(&b'=') {
            continue;
        }
        cursor += 1;
        while header.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let value_start = cursor;
        if let Some(quote @ (b'\'' | b'"')) = header.get(cursor).copied() {
            cursor += 1;
            while header.get(cursor).is_some_and(|byte| *byte != quote) {
                cursor += 1;
            }
            if header.get(cursor) != Some(&quote) {
                return None;
            }
            cursor += 1;
        } else {
            while header
                .get(cursor)
                .is_some_and(|&byte| !byte.is_ascii_whitespace() && !matches!(byte, b'>' | b'/'))
            {
                cursor += 1;
            }
        }
        if header.get(name_start..name_end) == Some(target.as_bytes()) {
            return Some((
                u32::try_from(start + value_start).ok()?,
                u32::try_from(start + cursor).ok()?,
            ));
        }
    }
    None
}

/// Recursively gather static `ref="..."` attributes from the template tree.
fn collect_template_refs<'a>(
    children: &[TemplateChildNode<'a>],
    source: &str,
    out: &mut Vec<TemplateRef>,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                if let Some(template_ref) = static_ref_of(element, source) {
                    out.push(template_ref);
                }
                collect_template_refs(&element.children, source, out);
            }
            TemplateChildNode::If(if_node) => {
                for branch in &if_node.branches {
                    collect_template_refs(&branch.children, source, out);
                }
            }
            TemplateChildNode::IfBranch(branch) => {
                collect_template_refs(&branch.children, source, out);
            }
            TemplateChildNode::For(for_node) => {
                collect_template_refs(&for_node.children, source, out);
            }
            _ => {}
        }
    }
}

/// The static `ref` attribute of an element, when it has a non-empty string
/// value. Dynamic `:ref` / `v-bind:ref` are directives, not attributes, so they
/// are not matched here.
fn static_ref_of(element: &ElementNode<'_>, source: &str) -> Option<TemplateRef> {
    for prop in &element.props {
        if let PropNode::Attribute(attr) = prop
            && attr.name == "ref"
        {
            let value = attr.value.as_ref()?;
            let name = value.content;
            if name.is_empty() {
                return None;
            }
            let mut loc = value.loc.clone();
            let start = loc.span.start as usize;
            let end = loc.span.end as usize;
            if let (Some(before), Some(after)) = (
                start
                    .checked_sub(1)
                    .and_then(|index| source.as_bytes().get(index)),
                source.as_bytes().get(end),
            ) && matches!(before, b'\'' | b'"')
                && before == after
            {
                loc = SourceLocation::new(loc.span.start - 1, loc.span.end + 1);
            }
            return Some(TemplateRef {
                name: name.into(),
                loc,
            });
        }
    }
    None
}

include!("no_unused_refs/script_usage.rs");
fn template_uses_static_ref(source: &str, name: &str) -> bool {
    let mut needle = CompactString::with_capacity("$refs.".len() + name.len());
    needle.push_str("$refs.");
    needle.push_str(name);
    source
        .match_indices(needle.as_str())
        .any(|(index, matched)| {
            let before = index.checked_sub(1).and_then(|i| source.as_bytes().get(i));
            let after = source.as_bytes().get(index + matched.len());
            !before.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                && !after.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        })
}

/// Whether `source` references `$refs` in a way whose target cannot be resolved
/// statically: a computed access `$refs[...]`, or `$refs` used as a whole value
/// (passed, destructured, assigned, returned, iterated) rather than a plain
/// `$refs.member` read. Such usage could reach any ref without naming it.
fn has_opaque_refs_access(source: &str) -> bool {
    const NEEDLE: &str = "$refs";
    let bytes = source.as_bytes();
    for (index, _) in source.match_indices(NEEDLE) {
        // Require a real `$refs` token (not e.g. `my$refs`).
        let before = index.checked_sub(1).and_then(|i| bytes.get(i).copied());
        if before.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_') {
            continue;
        }
        // Inspect the first non-whitespace byte after `$refs`.
        let mut cursor = index + NEEDLE.len();
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        match bytes.get(cursor).copied() {
            // `$refs.member` — a resolvable static read; the member name will be
            // matched by `token_present`, so this is not opaque.
            Some(b'.') => {}
            // Static object destructuring enumerates the exact ref names and
            // is handled by `RefUseVisitor`; it is not a whole-object escape.
            _ if is_static_refs_destructure(source, index) => {}
            // eslint-plugin-vue does not treat a bare `$refs` value observed by
            // an equality comparison as an escape. It continues to report
            // unrelated static refs in the same component.
            _ if is_refs_comparison(source, index, cursor) => {}
            // `$refs` at end of input, or anything else (`[`, `,`, `)`, `;`, `=`,
            // operators, etc.) is treated as opaque to stay sound.
            _ => return true,
        }
    }
    false
}

fn is_refs_comparison(source: &str, refs_start: usize, after_refs: usize) -> bool {
    const OPERATORS: [&str; 4] = ["===", "!==", "==", "!="];
    let before = source.get(..refs_start).map(str::trim_end).unwrap_or("");
    let before = before.strip_suffix("this.").unwrap_or(before).trim_end();
    let after = source.get(after_refs..).map(str::trim_start).unwrap_or("");
    OPERATORS
        .iter()
        .any(|operator| before.ends_with(operator) || after.starts_with(operator))
}

fn is_static_refs_destructure(source: &str, refs_start: usize) -> bool {
    let Some(prefix) = source.get(..refs_start) else {
        return false;
    };
    let Some(assignment) = prefix.rfind('=') else {
        return false;
    };
    let Some(rhs_prefix) = prefix.get(assignment + 1..).map(str::trim) else {
        return false;
    };
    if !matches!(rhs_prefix, "" | "this.") {
        return false;
    }
    let Some(left) = prefix.get(..assignment).map(str::trim_end) else {
        return false;
    };
    if !left.ends_with('}') {
        return false;
    }
    let Some(open) = left.rfind('{') else {
        return false;
    };
    let Some(entries) = left.get(open + 1..left.len() - 1) else {
        return false;
    };
    !entries.contains("...") && !entries.contains('[') && !entries.contains(']')
}

#[cfg(test)]
mod tests;
