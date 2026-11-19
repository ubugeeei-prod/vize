//! a11y/landmark-roles
//!
//! Validate landmark role placement and uniqueness.
//! Based on markuplint's `landmark-roles` rule.
//!
//! Checks:
//! - Duplicate `<main>` landmarks (only one allowed)
//! - Nested interactive landmarks of the same type
//! - Multiple `<nav>` elements without distinct labels
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <main>content</main>
//!   <main>another main</main>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <main>content</main>
//!   <nav aria-label="Primary">primary nav</nav>
//!   <nav aria-label="Footer">footer nav</nav>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::ir::TemplateSyntax;
use crate::markup::{
    MarkupBindingKind, MarkupBranch, MarkupContext, MarkupDocument, MarkupElement, MarkupRule,
    MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::a11y::markup_helpers::get_static_or_bound_literal_markup_value;
use vize_l0::FxHashMap;
use vize_l0::String;
use vize_l0::ToCompactString;
use vize_relief::RootNode;

static META: RuleMeta = RuleMeta {
    name: "a11y/landmark-roles",
    description: "Validate landmark role placement and uniqueness",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
/// Implements the `opinionated/a11y/landmark-roles` lint rule.
pub struct LandmarkRoles;

struct LandmarkInfo {
    role: String,
    label: Option<LandmarkLabel>,
    start: u32,
    end: u32,
    branches: Vec<MarkupBranch>,
}

enum LandmarkLabel {
    Static(String),
    Dynamic,
}

/// Landmark roles an explicit `role` attribute can declare.
const EXPLICIT_LANDMARK_ROLES: [&str; 8] = [
    "banner",
    "complementary",
    "contentinfo",
    "form",
    "main",
    "navigation",
    "region",
    "search",
];

/// The landmark role of an element: its first valued static `role`, else the
/// tag's implicit role.
fn get_landmark_role(element: &MarkupElement<'_>) -> Option<&'static str> {
    let mut explicit = None;
    element.walk_bindings(&mut |binding| {
        if explicit.is_none()
            && binding.kind() == MarkupBindingKind::Attribute
            && binding.is_unqualified_arg_exact("role")
        {
            explicit = binding.static_value();
        }
    });
    if let Some(value) = explicit {
        return EXPLICIT_LANDMARK_ROLES
            .into_iter()
            .find(|role| *role == value);
    }

    // Implicit roles from tags
    match element.tag() {
        "main" => Some("main"),
        "nav" => Some("navigation"),
        "aside" => Some("complementary"),
        "header" => Some("banner"),
        "footer" => Some("contentinfo"),
        "form" => Some("form"),
        "section" if get_label(element).is_some() => Some("region"),
        "search" => Some("search"),
        _ => None,
    }
}

fn has_bound_attribute(element: &MarkupElement<'_>, name: &str) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        found |=
            binding.kind() == MarkupBindingKind::Bind && binding.is_unqualified_arg_exact(name);
    });
    found
}

fn get_label(element: &MarkupElement<'_>) -> Option<LandmarkLabel> {
    for name in ["aria-label", "aria-labelledby", "title"] {
        if let Some(value) = get_static_or_bound_literal_markup_value(element, name)
            && !value.trim().is_empty()
        {
            return Some(LandmarkLabel::Static(value.to_compact_string()));
        }

        if has_bound_attribute(element, name) {
            return Some(LandmarkLabel::Dynamic);
        }
    }

    None
}

fn collect_landmarks(document: &MarkupDocument<'_>) -> Vec<LandmarkInfo> {
    let mut landmarks = Vec::new();
    document.walk_branched_elements(&mut |element, branches| {
        if !element.is_component()
            && let Some(role) = get_landmark_role(&element)
        {
            let range = element.range();
            landmarks.push(LandmarkInfo {
                role: role.to_compact_string(),
                label: get_label(&element),
                start: range.start,
                end: range.end,
                branches: branches.to_vec(),
            });
        }
    });
    landmarks
}

fn can_coexist(left: &LandmarkInfo, right: &LandmarkInfo) -> bool {
    MarkupBranch::can_coexist(&left.branches, &right.branches)
}

impl LandmarkRoles {
    fn check_document(ctx: &mut LintContext<'_>, document: &MarkupDocument<'_>) {
        let landmarks = collect_landmarks(document);

        // Check 1: Duplicate main landmarks
        let mains: Vec<&LandmarkInfo> = landmarks.iter().filter(|l| l.role == "main").collect();
        if mains.len() > 1 {
            for (index, main) in mains.iter().enumerate().skip(1) {
                if !mains
                    .iter()
                    .take(index)
                    .any(|prior| can_coexist(prior, main))
                {
                    continue;
                }
                let message = ctx.t("a11y/landmark-roles.duplicate_main");
                let diag = LintDiagnostic::warn(META.name, message, main.start, main.end)
                    .with_help(
                        ctx.t("a11y/landmark-roles.help_duplicate_main")
                            .into_owned(),
                    );
                ctx.report(diag);
            }
        }

        // Check 2: Multiple nav/aside/etc. without labels
        let mut role_groups: FxHashMap<&str, Vec<&LandmarkInfo>> = FxHashMap::default();
        for landmark in &landmarks {
            role_groups
                .entry(landmark.role.as_str())
                .or_default()
                .push(landmark);
        }

        for (role, group) in &role_groups {
            if *role == "main" {
                continue; // Already handled
            }
            if group.len() > 1 {
                let unlabeled: Vec<&&LandmarkInfo> = group
                    .iter()
                    .filter(|landmark| {
                        landmark.label.is_none()
                            && group.iter().any(|other| {
                                other.start != landmark.start && can_coexist(other, landmark)
                            })
                    })
                    .collect();
                for landmark in unlabeled {
                    let message =
                        ctx.t_fmt("a11y/landmark-roles.missing_label", &[("role", *role)]);
                    let diag =
                        LintDiagnostic::warn(META.name, message, landmark.start, landmark.end)
                            .with_help(
                                ctx.t("a11y/landmark-roles.help_missing_label").into_owned(),
                            );
                    ctx.report(diag);
                }

                let mut static_labels: FxHashMap<&str, Vec<&LandmarkInfo>> = FxHashMap::default();
                for landmark in group {
                    if let Some(LandmarkLabel::Static(label)) = &landmark.label {
                        static_labels
                            .entry(label.as_str())
                            .or_default()
                            .push(landmark);
                    }
                }

                for duplicate_group in static_labels.values().filter(|labels| labels.len() > 1) {
                    for landmark in duplicate_group {
                        if !duplicate_group.iter().any(|other| {
                            other.start != landmark.start && can_coexist(other, landmark)
                        }) {
                            continue;
                        }
                        let message =
                            ctx.t_fmt("a11y/landmark-roles.missing_label", &[("role", *role)]);
                        let diag =
                            LintDiagnostic::warn(META.name, message, landmark.start, landmark.end)
                                .with_help(
                                    ctx.t("a11y/landmark-roles.help_missing_label").into_owned(),
                                );
                        ctx.report(diag);
                    }
                }
            }
        }
    }
}

impl MarkupRule for LandmarkRoles {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Document
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        Self::check_document(ctx.lint(), document);
    }
}

impl Rule for LandmarkRoles {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn template_hooks(&self) -> crate::rule::RuleHooks {
        crate::rule::RuleHooks::TEMPLATE
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        // Landmarks are compared within each render root, across the branches
        // its conditionals choose between.
        true
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        Self::check_document(ctx, &MarkupDocument::new(root, TemplateSyntax::Vue));
    }
}

#[cfg(test)]
mod tests;
