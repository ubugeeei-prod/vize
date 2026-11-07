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
//! `useTemplateRef('x')` and static `$refs.x` / `$refs['x']` reads. Looking at
//! syntax and bindings instead of raw source text is important: an identifier in
//! a comment, documentation string, CSS selector, or an unrelated Options API
//! local must not keep a dead template ref alive.
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
use oxc_ast::ast::{
    Argument, CallExpression, ComputedMemberExpression, Expression, ObjectExpression,
    ObjectProperty, ObjectPropertyKind, Program, PropertyKey, Statement, StaticMemberExpression,
};
use oxc_ast_visit::{Visit, walk};
use vize_l0::{CompactString, cstr};
use vize_relief::{
    BindingType, ElementNode, PropNode, RootNode, SourceLocation, TemplateChildNode,
};

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
        collect_template_refs(&root.children, ctx.source, &mut refs);
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

            let scripts = ctx.sfc_scripts();
            refs.into_iter()
                .filter(|template_ref| {
                    let name = template_ref.name.as_str();
                    !binding_uses_template_ref(ctx, name)
                        && !scripts.is_some_and(|scripts| {
                            scripts.script().is_some_and(|parsed| {
                                program_uses_template_ref(&parsed.program, name)
                            }) || scripts.script_setup().is_some_and(|parsed| {
                                program_uses_template_ref(&parsed.program, name)
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

/// Script-setup top-level bindings and values returned from Options API
/// `setup()` are exposed to the template under their binding name.
fn binding_uses_template_ref(ctx: &LintContext<'_>, name: &str) -> bool {
    let Some(analysis) = ctx.analysis() else {
        return false;
    };
    let Some(kind) = analysis.bindings.get(name) else {
        return false;
    };
    analysis.bindings.is_script_setup
        || matches!(
            kind,
            BindingType::SetupLet
                | BindingType::SetupMaybeRef
                | BindingType::SetupRef
                | BindingType::SetupReactiveConst
                | BindingType::SetupConst
                | BindingType::LiteralConst
                | BindingType::Props
                | BindingType::PropsAliased
                | BindingType::ExternalModule
        )
}

fn program_uses_template_ref(program: &Program<'_>, name: &str) -> bool {
    let mut visitor = RefUseVisitor { name, used: false };
    visitor.visit_program(program);
    visitor.used
}

struct RefUseVisitor<'n> {
    name: &'n str,
    used: bool,
}

impl RefUseVisitor<'_> {
    fn refs_object(expression: &Expression<'_>) -> bool {
        match expression.get_inner_expression() {
            Expression::Identifier(identifier) => identifier.name == "$refs",
            Expression::StaticMemberExpression(member) => member.property.name == "$refs",
            Expression::ComputedMemberExpression(member) => {
                member.static_property_name().as_deref() == Some("$refs")
            }
            _ => false,
        }
    }
}

impl<'a> Visit<'a> for RefUseVisitor<'_> {
    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if !property.computed
            && property_key_name(&property.key) == Some("setup")
            && returned_object(&property.value).is_some_and(|object| {
                object.properties.iter().any(|property| {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return false;
                    };
                    !property.computed && property_key_name(&property.key) == Some(self.name)
                })
            })
        {
            self.used = true;
        }
        walk::walk_object_property(self, property);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && callee.name == "useTemplateRef"
            && let Some(Argument::StringLiteral(argument)) = call.arguments.first()
            && argument.value == self.name
        {
            self.used = true;
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if Self::refs_object(&member.object) && member.property.name == self.name {
            self.used = true;
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if Self::refs_object(&member.object)
            && member.static_property_name().as_deref() == Some(self.name)
        {
            self.used = true;
        }
        walk::walk_computed_member_expression(self, member);
    }
}

fn returned_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::FunctionExpression(function) => function
            .body
            .as_ref()?
            .statements
            .iter()
            .find_map(returned_object_statement),
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            let Statement::ExpressionStatement(statement) = arrow.body.statements.first()? else {
                return None;
            };
            object_expression(&statement.expression)
        }
        Expression::ArrowFunctionExpression(arrow) => arrow
            .body
            .statements
            .iter()
            .find_map(returned_object_statement),
        _ => None,
    }
}

fn returned_object_statement<'a>(statement: &'a Statement<'a>) -> Option<&'a ObjectExpression<'a>> {
    let Statement::ReturnStatement(statement) = statement else {
        return None;
    };
    object_expression(statement.argument.as_ref()?)
}

fn object_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

/// Whether `source` contains a `$refs` token at all (used to gate on template
/// `$refs` access, which this rule does not attempt to resolve).
fn mentions_refs_token(source: &str) -> bool {
    let bytes = source.as_bytes();
    source.match_indices("$refs").any(|(index, _)| {
        let before = index.checked_sub(1).and_then(|i| bytes.get(i).copied());
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
            // `$refs` at end of input, or anything else (`[`, `,`, `)`, `;`, `=`,
            // operators, etc.) is treated as opaque to stay sound.
            _ => return true,
        }
    }
    false
}

#[cfg(test)]
mod tests;
