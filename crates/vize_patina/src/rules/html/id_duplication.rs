//! html/id-duplication
//!
//! Detect duplicate static `id` attribute values in the same template.
//! Based on markuplint's `id-duplication` rule.
//!
//! This is different from `vue/use-unique-element-ids` which warns about
//! any static ID. This rule only warns when the same ID literal appears
//! more than once.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <div id="content">first</div>
//!   <div id="content">second</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <div id="content">first</div>
//!   <div id="sidebar">second</div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::FxHashMap;
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_relief::{ElementNode, PropNode, RootNode, SourceLocation, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "html/id-duplication",
    description: "Disallow duplicate element IDs",
    category: RuleCategory::HtmlConformance,
    fixable: false,
    default_severity: Severity::Error,
};

#[derive(Default)]
pub struct IdDuplication;

struct IdEntry {
    value: String,
    loc: LocInfo,
    branches: Vec<BranchChoice>,
}

#[derive(Clone, Copy)]
struct BranchChoice {
    if_start: u32,
    branch_index: usize,
}

#[derive(Clone)]
struct LocInfo {
    start: u32,
    end: u32,
}

impl Rule for IdDuplication {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let mut ids: Vec<IdEntry> = Vec::new();
        let mut branches: Vec<BranchChoice> = Vec::new();

        collect_static_ids(&root.children, &mut branches, &mut ids);

        // Find duplicates
        let mut seen: FxHashMap<&str, Vec<&IdEntry>> = FxHashMap::default();

        for entry in &ids {
            let duplicate = seen
                .get(entry.value.as_str())
                .and_then(|entries| entries.iter().find(|prev| can_coexist(prev, entry)));

            if let Some(first) = duplicate {
                let message = ctx.t_fmt(
                    "html/id-duplication.message",
                    &[("id", entry.value.as_str())],
                );
                let help = ctx.t("html/id-duplication.help");
                let diag =
                    LintDiagnostic::error(META.name, message, entry.loc.start, entry.loc.end)
                        .with_help(help.into_owned())
                        .with_label(
                            "first defined here".to_compact_string(),
                            first.loc.start,
                            first.loc.end,
                        );
                ctx.report(diag);
            }
            seen.entry(entry.value.as_str()).or_default().push(entry);
        }
    }
}

fn collect_static_ids<'a>(
    children: &[TemplateChildNode<'a>],
    branches: &mut Vec<BranchChoice>,
    ids: &mut Vec<IdEntry>,
) {
    let mut index = 0;
    while index < children.len() {
        if let TemplateChildNode::Element(el) = &children[index]
            && element_has_directive(el, "if")
        {
            index = collect_conditional_element_chain(children, index, branches, ids);
            continue;
        }

        match &children[index] {
            TemplateChildNode::Element(el) => {
                collect_element_id(el, branches, ids);
                collect_static_ids(&el.children, branches, ids);
            }
            TemplateChildNode::If(if_node) => {
                for (branch_index, branch) in if_node.branches.iter().enumerate() {
                    branches.push(BranchChoice {
                        if_start: if_node.loc.start.offset,
                        branch_index,
                    });
                    collect_static_ids(&branch.children, branches, ids);
                    branches.pop();
                }
            }
            TemplateChildNode::IfBranch(branch) => {
                collect_static_ids(&branch.children, branches, ids)
            }
            TemplateChildNode::For(for_node) => {
                collect_static_ids(&for_node.children, branches, ids);
            }
            _ => {}
        }
        index += 1;
    }
}

fn collect_conditional_element_chain<'a>(
    children: &[TemplateChildNode<'a>],
    mut index: usize,
    branches: &mut Vec<BranchChoice>,
    ids: &mut Vec<IdEntry>,
) -> usize {
    let TemplateChildNode::Element(first) = &children[index] else {
        return index + 1;
    };
    let if_start = first.loc.start.offset;
    let mut branch_index = 0;

    collect_element_branch(first, if_start, branch_index, branches, ids);
    index += 1;

    while let Some(next_index) = next_branch_candidate(children, index) {
        let TemplateChildNode::Element(branch) = &children[next_index] else {
            break;
        };
        let is_else = element_has_directive(branch, "else");
        if !is_else && !element_has_directive(branch, "else-if") {
            break;
        }

        branch_index += 1;
        collect_element_branch(branch, if_start, branch_index, branches, ids);
        index = next_index + 1;

        if is_else {
            break;
        }
    }

    index
}

fn collect_element_branch(
    element: &ElementNode,
    if_start: u32,
    branch_index: usize,
    branches: &mut Vec<BranchChoice>,
    ids: &mut Vec<IdEntry>,
) {
    branches.push(BranchChoice {
        if_start,
        branch_index,
    });
    collect_element_id(element, branches, ids);
    collect_static_ids(&element.children, branches, ids);
    branches.pop();
}

fn next_branch_candidate(children: &[TemplateChildNode<'_>], index: usize) -> Option<usize> {
    children
        .iter()
        .enumerate()
        .skip(index)
        .find(|(_, child)| !is_ignorable_between_branches(child))
        .map(|(index, _)| index)
}

fn is_ignorable_between_branches(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Text(text) => text.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => true,
        _ => false,
    }
}

fn collect_element_id(element: &ElementNode, branches: &[BranchChoice], ids: &mut Vec<IdEntry>) {
    for prop in &element.props {
        if let PropNode::Attribute(attr) = prop
            && attr.name == "id"
            && let Some(value) = &attr.value
        {
            ids.push(IdEntry {
                value: value.content.to_compact_string(),
                loc: loc_info(&attr.loc),
                branches: branches.to_vec(),
            });
        }
    }
}

fn can_coexist(left: &IdEntry, right: &IdEntry) -> bool {
    left.branches.iter().all(|left_branch| {
        right
            .branches
            .iter()
            .find(|right_branch| right_branch.if_start == left_branch.if_start)
            .is_none_or(|right_branch| right_branch.branch_index == left_branch.branch_index)
    })
}

fn element_has_directive(element: &ElementNode, name: &str) -> bool {
    element.props.iter().any(
        |prop| matches!(prop, PropNode::Directive(directive) if directive.name.as_str() == name),
    )
}

fn loc_info(loc: &SourceLocation) -> LocInfo {
    LocInfo {
        start: loc.start.offset,
        end: loc.end.offset,
    }
}

#[cfg(test)]
mod tests {
    use super::IdDuplication;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(IdDuplication));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_unique_ids() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div id="a">A</div><div id="b">B</div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_ids() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>A</div><div>B</div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_dynamic_ids() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :id="id1">A</div><div :id="id2">B</div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_ids() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div id="content">A</div><div id="content">B</div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_triple_duplicate() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div id="x">A</div><div id="x">B</div><div id="x">C</div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_invalid_nested_duplicate() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div id="foo"><span id="foo">text</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_duplicate_ids_across_v_if_branches() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="show"><input id="radio1"></div><div v-else><input id="radio1"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_ids_inside_same_v_if_branch() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="show"><input id="radio1"><p id="radio1"></p></div><div v-else><input id="radio1"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_id_after_v_if_branch() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="show"><input id="radio1"></div><div v-else><input id="other"></div><p id="radio1"></p>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }
}
