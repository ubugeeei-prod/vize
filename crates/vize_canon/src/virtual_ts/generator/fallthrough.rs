use vize_carton::{String, is_native_tag};
use vize_croquis::Croquis;
use vize_relief::{ElementNode, ExpressionNode, IfNode, PropNode, RootNode, TemplateChildNode};

pub(super) fn fallthrough_props_type_ref(
    summary: &Croquis,
    template_ast: Option<&RootNode<'_>>,
    legacy_vue2: bool,
) -> Option<String> {
    if legacy_vue2 || summary.template_info.inherit_attrs_disabled {
        return None;
    }
    let tags = possible_single_native_root_tags(template_ast?)?;
    let mut ty = String::default();
    for (index, tag) in tags.iter().enumerate() {
        if index > 0 {
            ty.push_str(" & ");
        }
        ty.push_str("Partial<__VizeNativeElement<");
        push_ts_string_literal(&mut ty, tag.as_str());
        ty.push_str(">>");
    }
    Some(ty)
}

fn possible_single_native_root_tags(root: &RootNode<'_>) -> Option<Vec<String>> {
    possible_single_native_root_tags_from_children(root.children.as_slice())
}

fn possible_single_native_root_tags_from_children(
    children: &[TemplateChildNode<'_>],
) -> Option<Vec<String>> {
    let roots = children
        .iter()
        .filter(|child| !is_ignorable_root_child(child))
        .collect::<Vec<_>>();
    if let [root] = roots.as_slice() {
        return possible_single_native_root_tags_from_child(root);
    }
    possible_raw_if_chain_tags(roots.as_slice())
}

fn possible_single_native_root_tags_from_child(
    child: &TemplateChildNode<'_>,
) -> Option<Vec<String>> {
    match child {
        TemplateChildNode::Element(element)
            if element.tag.as_str() == "template" && !has_for_directive(element) =>
        {
            possible_single_native_root_tags_from_children(element.children.as_slice())
        }
        TemplateChildNode::Element(element)
            if is_native_tag(element.tag.as_str()) && !has_for_directive(element) =>
        {
            Some(vec![String::from(element.tag.as_str())])
        }
        TemplateChildNode::If(node) => possible_if_root_tags(node),
        _ => None,
    }
}

fn possible_raw_if_chain_tags(children: &[&TemplateChildNode<'_>]) -> Option<Vec<String>> {
    let (first, rest) = children.split_first()?;
    let first_element = element_child(first)?;
    let ElementBranchKind::If(first_condition) = element_branch_kind(first_element)? else {
        return None;
    };
    if condition_is_literal_true(first_condition) && rest_is_only_if_chain_branches(rest) {
        return possible_element_branch_tags(first_element);
    }

    let mut tags = possible_element_branch_tags(first_element)?;
    for child in rest {
        let element = element_child(child)?;
        match element_branch_kind(element)? {
            ElementBranchKind::ElseIf(condition) => {
                tags.extend(possible_element_branch_tags(element)?);
                if condition_is_literal_true(condition) {
                    return Some(tags);
                }
            }
            ElementBranchKind::Else => {
                tags.extend(possible_element_branch_tags(element)?);
                return Some(tags);
            }
            _ => return None,
        }
    }
    None
}

fn rest_is_only_if_chain_branches(children: &[&TemplateChildNode<'_>]) -> bool {
    let mut has_final_else = false;
    for child in children {
        let Some(element) = element_child(child) else {
            return false;
        };
        match element_branch_kind(element) {
            Some(ElementBranchKind::ElseIf(_)) if !has_final_else => {}
            Some(ElementBranchKind::Else) if !has_final_else => has_final_else = true,
            _ => return false,
        }
    }
    true
}

fn element_child<'a>(child: &'a TemplateChildNode<'a>) -> Option<&'a ElementNode<'a>> {
    match child {
        TemplateChildNode::Element(element) => Some(element),
        _ => None,
    }
}

fn possible_element_branch_tags(element: &ElementNode<'_>) -> Option<Vec<String>> {
    if has_for_directive(element) {
        return None;
    }
    if element.tag.as_str() == "template" {
        return possible_single_native_root_tags_from_children(element.children.as_slice());
    }
    is_native_tag(element.tag.as_str()).then(|| vec![String::from(element.tag.as_str())])
}

enum ElementBranchKind<'a> {
    If(&'a ExpressionNode<'a>),
    ElseIf(&'a ExpressionNode<'a>),
    Else,
}

fn element_branch_kind<'a>(element: &'a ElementNode<'a>) -> Option<ElementBranchKind<'a>> {
    for prop in &element.props {
        let PropNode::Directive(directive) = prop else {
            continue;
        };
        match directive.name.as_str() {
            "if" => return directive.exp.as_ref().map(ElementBranchKind::If),
            "else-if" => {
                return directive.exp.as_ref().map(ElementBranchKind::ElseIf);
            }
            "else" => return Some(ElementBranchKind::Else),
            _ => {}
        }
    }
    None
}

fn has_for_directive(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(directive) if directive.name.as_str() == "for"
        )
    })
}

fn possible_if_root_tags(node: &IfNode<'_>) -> Option<Vec<String>> {
    let first_branch = node.branches.first()?;
    if first_branch
        .condition
        .as_ref()
        .is_some_and(condition_is_literal_true)
    {
        return possible_single_native_root_tags_from_children(first_branch.children.as_slice());
    }
    let mut tags = Vec::new();
    for branch in &node.branches {
        tags.extend(possible_single_native_root_tags_from_children(
            branch.children.as_slice(),
        )?);
        if branch.condition.is_none()
            || branch
                .condition
                .as_ref()
                .is_some_and(condition_is_literal_true)
        {
            return Some(tags);
        }
    }
    None
}

fn is_ignorable_root_child(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Text(text) => text.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => true,
        _ => false,
    }
}

fn condition_is_literal_true(condition: &ExpressionNode<'_>) -> bool {
    matches!(condition, ExpressionNode::Simple(simple) if simple.content.trim() == "true")
}

fn push_ts_string_literal(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ => output.push(character),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests {
    use vize_carton::{Box as VizeBox, Bump};
    use vize_croquis::{Analyzer, AnalyzerOptions};
    use vize_relief::{
        DirectiveNode, ElementNode, ExpressionNode, PropNode, SimpleExpressionNode, SourceLocation,
        TemplateChildNode,
    };

    use super::{fallthrough_props_type_ref, possible_raw_if_chain_tags};

    fn fallthrough_type(script: &str, template: &str) -> Option<vize_carton::String> {
        let allocator = Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        fallthrough_props_type_ref(&summary, Some(&root), false)
    }

    fn raw_branch<'a>(
        allocator: &'a Bump,
        tag: &str,
        directive_name: &str,
        condition: Option<&str>,
    ) -> TemplateChildNode<'a> {
        let mut element = ElementNode::new(allocator, tag, SourceLocation::STUB);
        let mut directive = DirectiveNode::new(allocator, directive_name, SourceLocation::STUB);
        directive.exp = condition.map(|condition| {
            ExpressionNode::Simple(VizeBox::new_in(
                SimpleExpressionNode::new(condition, false, SourceLocation::STUB),
                allocator,
            ))
        });
        element
            .props
            .push(PropNode::Directive(VizeBox::new_in(directive, allocator)));
        TemplateChildNode::Element(VizeBox::new_in(element, allocator))
    }

    #[test]
    fn emits_native_root_fallthrough_props_for_single_root() {
        let ty = fallthrough_type(
            "defineProps<{ title: string }>()",
            "<button>{{ title }}</button>",
        )
        .expect("single native root should accept fallthrough props");

        assert_eq!(ty, "Partial<__VizeNativeElement<\"button\">>");
    }

    #[test]
    fn skips_fallthrough_props_when_inherit_attrs_is_false() {
        let ty = fallthrough_type(
            "defineOptions({ inheritAttrs: false })\ndefineProps<{ title: string }>()",
            "<div>{{ title }}</div>",
        );

        assert_eq!(ty, None);
    }

    #[test]
    fn skips_fallthrough_props_for_multi_root_or_mixed_v_if_branch() {
        for template in [
            "<div /> <span />",
            "<div v-if=\"on\" /><template v-else><p /><p /></template>",
            "<template v-if=\"true\"><p /><p /></template>",
            "<div v-if=\"true\" /><span />",
            "<div v-if=\"on\" /><span v-else-if=\"maybe\" />",
            "<div v-for=\"item in items\" />",
            "<template v-for=\"item in items\"><div /></template>",
        ] {
            assert_eq!(
                fallthrough_type("const on = true\nconst items = [1]", template),
                None,
                "template should not be an always-single native root: {template}"
            );
        }
    }

    #[test]
    fn combines_native_fallthrough_props_when_all_v_if_branches_are_single_roots() {
        let ty = fallthrough_type("const on = true", "<div v-if=\"on\" /><span v-else />")
            .expect("v-if/v-else single native roots should accept common fallthrough props");

        assert_eq!(
            ty,
            "Partial<__VizeNativeElement<\"div\">> & Partial<__VizeNativeElement<\"span\">>"
        );
    }

    #[test]
    fn literal_true_v_else_if_terminates_single_native_root_chain() {
        let ty = fallthrough_type(
            "const on = true",
            "<div v-if=\"on\" /><span v-else-if=\"true\" />",
        )
        .expect("literal true v-else-if is an exhaustive native-root branch");

        assert_eq!(
            ty,
            "Partial<__VizeNativeElement<\"div\">> & Partial<__VizeNativeElement<\"span\">>"
        );
    }

    #[test]
    fn raw_literal_true_v_else_if_terminates_single_native_root_chain() {
        let allocator = Bump::new();
        let first = raw_branch(&allocator, "div", "if", Some("on"));
        let second = raw_branch(&allocator, "span", "else-if", Some("true"));
        let refs: std::vec::Vec<_> = [&first, &second].into_iter().collect();

        assert_eq!(
            possible_raw_if_chain_tags(refs.as_slice()),
            Some(vec![
                vize_carton::String::from("div"),
                vize_carton::String::from("span")
            ])
        );
    }
}
