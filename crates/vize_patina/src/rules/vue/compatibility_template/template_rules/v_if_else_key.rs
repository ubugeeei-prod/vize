//! Keys on repeated conditional components.
//!
//! Mirrors eslint-plugin-vue: each parent keeps one conditional family, which
//! every `v-if` restarts, and a keyless repeated component is reported only
//! while it is a member of its parent's current family. Repeats are counted
//! per enclosing component (or the template root).

use super::shared::has_key;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::{FxHashMap, cstr, ensure_sufficient_stack};
use vize_relief::{ElementNode, PropNode, RootNode, TemplateChildNode};

use crate::rules::vue::require_v_for_key::relief_element_is_custom_component;

static META: RuleMeta = RuleMeta {
    name: "vue/v-if-else-key",
    description: "Require keys on repeated conditional components",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/v-if-else-key`.
pub struct VIfElseKey;

impl Rule for VIfElseKey {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let mut walk = Walk {
            families: FxHashMap::default(),
            scopes: vec![FxHashMap::default()],
        };
        walk.children(ctx, address(root), &root.children);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Condition {
    If,
    ElseIf,
    Else,
}

/// Element addresses of the current `v-if` / `v-else-if` / `v-else` family.
struct Family {
    if_branch: usize,
    else_if_branches: Vec<usize>,
    else_branch: Option<usize>,
}

impl Family {
    fn new(if_branch: usize) -> Self {
        Self {
            if_branch,
            else_if_branches: Vec::new(),
            else_branch: None,
        }
    }

    fn contains(&self, element: usize) -> bool {
        self.if_branch == element
            || self.else_branch == Some(element)
            || self.else_if_branches.contains(&element)
    }
}

/// The first conditional use of a component name in a scope.
struct FirstUse<'s, 'a> {
    element: &'s ElementNode<'a>,
    parent: usize,
    siblings: &'s [TemplateChildNode<'a>],
}

struct Usage<'s, 'a> {
    count: usize,
    first: Option<FirstUse<'s, 'a>>,
}

struct Walk<'s, 'a> {
    families: FxHashMap<usize, Family>,
    scopes: Vec<FxHashMap<&'a str, Usage<'s, 'a>>>,
}

fn address<T>(node: &T) -> usize {
    std::ptr::from_ref(node) as usize
}

impl<'s, 'a> Walk<'s, 'a> {
    fn children(
        &mut self,
        ctx: &mut LintContext<'a>,
        parent: usize,
        siblings: &'s [TemplateChildNode<'a>],
    ) {
        for child in siblings {
            if let TemplateChildNode::Element(element) = child {
                ensure_sufficient_stack(|| self.element(ctx, element, parent, siblings));
            }
        }
    }

    fn element(
        &mut self,
        ctx: &mut LintContext<'a>,
        element: &'s ElementNode<'a>,
        parent: usize,
        siblings: &'s [TemplateChildNode<'a>],
    ) {
        let own = address(element);
        if matches!(element.tag, "component" | "slot" | "template") {
            self.children(ctx, own, &element.children);
            return;
        }
        let condition = condition(element);
        if let Some(condition) = condition {
            let family = self
                .families
                .entry(parent)
                .or_insert_with(|| Family::new(own));
            match condition {
                Condition::If => *family = Family::new(own),
                Condition::ElseIf => family.else_if_branches.push(own),
                Condition::Else => family.else_branch = Some(own),
            }
        }
        let custom = relief_element_is_custom_component(element);
        if !custom || condition.is_none() {
            if custom {
                self.scopes.push(FxHashMap::default());
            }
            self.children(ctx, own, &element.children);
            if custom {
                self.scopes.pop();
            }
            return;
        }

        let mut first = None;
        let mut repeated = false;
        if let Some(scope) = self.scopes.last_mut() {
            let usage = scope.entry(element.tag).or_insert(Usage {
                count: 0,
                first: None,
            });
            if usage.count == 0 {
                usage.first = Some(FirstUse {
                    element,
                    parent,
                    siblings,
                });
            } else {
                repeated = true;
                if usage.count == 1 {
                    first = usage
                        .first
                        .as_ref()
                        .map(|first| (first.element, first.parent, first.siblings));
                }
            }
            usage.count += 1;
        }
        if repeated {
            self.check(ctx, element, parent, siblings);
            if let Some((element, parent, siblings)) = first {
                self.check(ctx, element, parent, siblings);
            }
        }

        self.scopes.push(FxHashMap::default());
        self.children(ctx, own, &element.children);
        self.scopes.pop();
    }

    fn check(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        parent: usize,
        siblings: &[TemplateChildNode<'a>],
    ) {
        let own = address(element);
        let has_conditional_sibling = siblings.iter().any(|sibling| {
            matches!(sibling, TemplateChildNode::Element(sibling)
                if address(sibling.as_ref()) != own
                    && sibling.tag == element.tag
                    && condition(sibling).is_some())
        });
        if !has_conditional_sibling
            || has_key(element)
            || !self
                .families
                .get(&parent)
                .is_some_and(|family| family.contains(own))
        {
            return;
        }
        ctx.warn_with_help(
            cstr!(
                "Conditionally rendered repeated component `{}` should have a key",
                element.tag
            ),
            &element.loc,
            "Add a unique `key` or `:key` to each conditional branch",
        );
    }
}

fn condition(element: &ElementNode<'_>) -> Option<Condition> {
    let has = |name: &str| {
        element
            .props
            .iter()
            .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == name))
    };
    if has("if") {
        Some(Condition::If)
    } else if has("else-if") {
        Some(Condition::ElseIf)
    } else if has("else") {
        Some(Condition::Else)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::VIfElseKey;
    use crate::linter::{LintResult, Linter};
    use crate::rule::RuleRegistry;

    fn lint(template: &str) -> LintResult {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VIfElseKey));
        Linter::with_registry(registry).lint_template(template, "Component.vue")
    }

    #[test]
    fn reports_members_of_the_current_family_only() {
        for (template, expected) in [
            (r#"<b><MyIcon v-if="a" /><MyIcon v-else /></b>"#, 2),
            (
                r#"<b><MyIcon v-if="a" /><MyIcon v-if="b" key="k" /></b>"#,
                0,
            ),
            (
                r#"<b><span v-if="a" /><MyIcon v-else /><MyIcon v-if="b" key="k" /></b>"#,
                0,
            ),
            (r#"<b><MyIcon v-if="a" /><MyIcon v-if="b" /></b>"#, 1),
            (r#"<MyIcon v-if="a" /><MyIcon v-else />"#, 2),
            (
                r#"<b><MyIcon v-if="a" /><MyIcon v-else-if="b" /><MyIcon v-else /></b>"#,
                3,
            ),
        ] {
            let result = lint(template);
            assert_eq!(
                result.warning_count, expected,
                "{template}: {:?}",
                result.diagnostics
            );
        }
    }
}
