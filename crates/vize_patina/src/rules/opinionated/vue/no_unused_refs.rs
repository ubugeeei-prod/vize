//! vue/no-unused-refs
//!
//! Report template refs (`ref="x"`) that are never referenced from `<script>`.
//!
//! In Vue, a template ref attribute `ref="x"` only does something if the script
//! reads it back — through a `<script setup>` variable named `x`, a
//! `useTemplateRef('x')` call, or `this.$refs.x` / `$refs.x` access. A `ref="x"`
//! the script never touches is dead markup that should be removed.
//!
//! ## Scope (sound conservative subset)
//!
//! Patina runs its template pass and its script pass separately, so this rule
//! cannot reuse the full semantic binding graph the ESLint rule relies on.
//! Instead it correlates each static `ref="x"` against the **raw `<script>`
//! source text**: a ref is considered *used* whenever its name appears anywhere
//! in a script block as a whole identifier/string token. That single token test
//! covers every real usage soundly — a `<script setup>` variable `x`, the string
//! key in `useTemplateRef('x')`, the member access in `$refs.x` / `$refs['x']`,
//! and destructuring `const { x } = $refs` — because each of those forms contains
//! the literal token `x` in the script. The rule therefore only ever reports a
//! ref whose name is *textually absent* from every script block, so it does not
//! false-positive on refs accessed indirectly.
//!
//! To stay sound in the few cases the text test cannot resolve, the rule reports
//! nothing for the file when:
//!
//! - there is no `<script>` / `<script setup>` block to correlate against;
//! - a script or the template uses `$refs` opaquely — a computed access
//!   `$refs[expr]`, or `$refs` passed / destructured / iterated as a whole object
//!   — since any ref could then be reached without its name appearing literally.
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
use vize_carton::{CompactString, cstr};
use vize_relief::{ElementNode, PropNode, RootNode, SourceLocation, TemplateChildNode};

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

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Collect every static `ref="x"` in the template up front.
        let mut refs: Vec<TemplateRef> = Vec::new();
        collect_template_refs(&root.children, &mut refs);
        if refs.is_empty() {
            return;
        }

        // Resolve which refs are unused while the descriptor borrow is held, then
        // drop it so diagnostics can be reported through `&mut ctx`.
        let unused: Vec<TemplateRef> = {
            // This rule only makes sense against a script block to correlate
            // with. Without one there is nothing that could "use" the ref, and
            // we also cannot observe `$refs` / `useTemplateRef` resolution, so
            // stay quiet.
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
            if script.is_empty() && script_setup.is_empty() {
                return;
            }

            // Opaque `$refs` usage in script (computed access or whole-object
            // handling) can reach any ref without naming it; bail out to avoid
            // false positives.
            if has_opaque_refs_access(script) || has_opaque_refs_access(script_setup) {
                return;
            }

            // Any `$refs` in the *template* means refs may be consumed there
            // (e.g. `{{ $refs.x.value }}`) rather than in `<script>`. Resolving
            // those against the per-ref check is unreliable across the separate
            // passes, so treat the presence of template `$refs` as a reason to
            // stay quiet for the whole file.
            let template_src = descriptor
                .template
                .as_ref()
                .map(|block| block.content.as_ref())
                .unwrap_or("");
            if mentions_refs_token(template_src) {
                return;
            }

            refs.into_iter()
                .filter(|template_ref| {
                    let name = template_ref.name.as_str();
                    !(token_present(script, name) || token_present(script_setup, name))
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

/// Recursively gather static `ref="..."` attributes from the template tree.
fn collect_template_refs<'a>(children: &[TemplateChildNode<'a>], out: &mut Vec<TemplateRef>) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                if let Some(template_ref) = static_ref_of(element) {
                    out.push(template_ref);
                }
                collect_template_refs(&element.children, out);
            }
            TemplateChildNode::If(if_node) => {
                for branch in &if_node.branches {
                    collect_template_refs(&branch.children, out);
                }
            }
            TemplateChildNode::IfBranch(branch) => collect_template_refs(&branch.children, out),
            TemplateChildNode::For(for_node) => collect_template_refs(&for_node.children, out),
            _ => {}
        }
    }
}

/// The static `ref` attribute of an element, when it has a non-empty string
/// value. Dynamic `:ref` / `v-bind:ref` are directives, not attributes, so they
/// are not matched here.
fn static_ref_of(element: &ElementNode<'_>) -> Option<TemplateRef> {
    for prop in &element.props {
        if let PropNode::Attribute(attr) = prop
            && attr.name == "ref"
        {
            let value = attr.value.as_ref()?;
            let name = value.content.as_str();
            if name.is_empty() {
                return None;
            }
            return Some(TemplateRef {
                name: name.into(),
                loc: attr.loc.clone(),
            });
        }
    }
    None
}

/// Whether `name` occurs in `source` as a whole identifier/string token.
///
/// Matching on token boundaries means `$refs.inputEl`, `useTemplateRef('inputEl')`,
/// a bare `inputEl` reference, and `const { inputEl } = $refs` all count, while a
/// longer identifier such as `inputElement` does not.
fn token_present(source: &str, name: &str) -> bool {
    if name.is_empty() || source.len() < name.len() {
        return false;
    }
    let bytes = source.as_bytes();
    source.match_indices(name).any(|(index, _)| {
        let before = index.checked_sub(1).map(|i| bytes[i]);
        let after = bytes.get(index + name.len()).copied();
        is_token_boundary(before) && is_token_boundary(after)
    })
}

/// A byte that cannot be part of a JS identifier delimits an identifier token.
fn is_token_boundary(byte: Option<u8>) -> bool {
    !byte.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'$')
}

/// Whether `source` contains a `$refs` token at all (used to gate on template
/// `$refs` access, which this rule does not attempt to resolve).
fn mentions_refs_token(source: &str) -> bool {
    let bytes = source.as_bytes();
    source.match_indices("$refs").any(|(index, _)| {
        let before = index.checked_sub(1).map(|i| bytes[i]);
        // `$` cannot follow an identifier char in a real `$refs`, but guard the
        // alphanumeric/underscore case anyway to avoid matching `x$refs`.
        !before.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
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
        let before = index.checked_sub(1).map(|i| bytes[i]);
        if before.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_') {
            continue;
        }
        // Inspect the first non-whitespace byte after `$refs`.
        let mut cursor = index + NEEDLE.len();
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        match bytes.get(cursor).copied() {
            // `$refs.member` — a resolvable static read; the member name will be
            // matched by `token_present`, so this is not opaque.
            Some(b'.') => {}
            // `$refs` at end of input, or anything else (`[`, `,`, `)`, `;`, `=`,
            // operators, etc.) is treated as opaque to stay sound.
            _ => return true,
        }
    }
    false
}

#[cfg(test)]
mod tests;
