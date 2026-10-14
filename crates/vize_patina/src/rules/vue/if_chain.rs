//! Shared `v-if` / `v-else-if` / `v-else` chain walk.
//!
//! Both `vue/valid-v-else` and `vue/valid-v-else-if` need to know whether a
//! branch directive directly follows an element that opens or continues a
//! conditional chain. Whitespace-only text and comments may sit between the
//! branches; any other sibling ends the chain, matching the Vue compiler.

use vize_relief::{ElementNode, PropNode, SourceLocation, TemplateChildNode};

/// A branch directive found without a preceding chain element.
pub(super) enum OrphanBranch {
    /// `v-else-if` with no adjacent `v-if` / `v-else-if` before it.
    ElseIf,
    /// `v-else` with no adjacent `v-if` / `v-else-if` before it.
    Else,
}

struct BranchInfo {
    has_v_if: bool,
    else_if_loc: Option<SourceLocation>,
    else_loc: Option<SourceLocation>,
}

/// Call `report` for every `v-else-if` / `v-else` among `children` that does
/// not directly follow an element carrying `v-if` or `v-else-if`.
pub(super) fn for_each_orphan_branch(
    children: &[TemplateChildNode],
    mut report: impl FnMut(OrphanBranch, &SourceLocation),
) {
    let mut can_follow_if = false;

    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                let info = branch_info(element);

                if info.has_v_if {
                    can_follow_if = true;
                    continue;
                }

                if let Some(loc) = info.else_if_loc {
                    if !can_follow_if {
                        report(OrphanBranch::ElseIf, &loc);
                    }
                    // Like eslint-plugin-vue, a following branch only looks at
                    // its previous element, so one orphan yields one report.
                    can_follow_if = true;
                    continue;
                }

                if let Some(loc) = info.else_loc
                    && !can_follow_if
                {
                    report(OrphanBranch::Else, &loc);
                }
                can_follow_if = false;
            }
            TemplateChildNode::Text(text) if text.content.trim().is_empty() => {}
            TemplateChildNode::Comment(_) => {}
            _ => {
                can_follow_if = false;
            }
        }
    }
}

fn branch_info(element: &ElementNode) -> BranchInfo {
    let mut info = BranchInfo {
        has_v_if: false,
        else_if_loc: None,
        else_loc: None,
    };

    for prop in element.props.iter() {
        if let PropNode::Directive(dir) = prop {
            match dir.name {
                "if" => info.has_v_if = true,
                "else-if" => info.else_if_loc = Some(dir.loc.clone()),
                "else" => info.else_loc = Some(dir.loc.clone()),
                _ => {}
            }
        }
    }

    info
}
