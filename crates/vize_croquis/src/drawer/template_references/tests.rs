use super::*;
use serde_json::Value;
use vize_armature::Parser;
use vize_l0::Allocator;

fn component_tokens(source: &str) -> (Vec<(String, u32, u32)>, Vec<TemplateReferenceGap>) {
    let allocator = Allocator::default();
    let (root, errors, syntax) = Parser::new(&allocator, source).parse_with_template_syntax_facts();
    assert!(errors.is_empty(), "{errors:?}: {source}");
    let packet = collect_template_references(&root, &syntax);
    let tokens = packet
        .references
        .into_iter()
        .filter(|reference| {
            reference.owner == TemplateReferenceOwner::ComponentContext
                && reference.name == "$listeners"
        })
        .map(|reference| {
            (
                reference.name.to_string(),
                reference.span.start,
                reference.span.end,
            )
        })
        .collect();
    (tokens, packet.gaps)
}

#[test]
fn complete_original_seventy_packets_keep_all_seventeen_template_contracts() {
    let authored: Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/independent.json"
    )).unwrap();
    let native: Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/native.json"
    ))
    .unwrap();
    let mut count = 0;
    assert_eq!(authored["records"].as_array().unwrap().len(), 70);
    for case in authored["records"].as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
        let Some(template) = descriptor.template else {
            continue;
        };
        count += 1;
        let (tokens, _) = component_tokens(&template.content);
        let actual: Vec<_> = tokens
            .into_iter()
            .map(|(_, start, end)| {
                (
                    start + template.loc.start as u32,
                    end + template.loc.start as u32,
                )
            })
            .collect();
        let expected = native["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == case["id"])
            .unwrap();
        let expected: Vec<_> = expected["messages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|message| {
                (
                    message["start"].as_u64().unwrap() as u32,
                    message["end"].as_u64().unwrap() as u32,
                )
            })
            .collect();
        assert_eq!(actual, expected, "{}", case["id"]);
    }
    assert_eq!(count, 17);
}

#[test]
fn collection_precedes_alias_and_slot_attributes_precede_props() {
    for (source, spelling, occurrence) in [
        (
            "<div v-for=\"$listeners in $listeners\">{{ $listeners }}</div>",
            "$listeners",
            1,
        ),
        (
            "<Child v-slot=\"{ $listeners }\" :value=\"$listeners\">{{ $listeners }}</Child>",
            "$listeners",
            1,
        ),
        (
            "<Child v-slot:[$listeners]=\"{ $listeners }\">{{ $listeners }}</Child>",
            "$listeners",
            0,
        ),
    ] {
        let (tokens, gaps) = component_tokens(source);
        let start = source.match_indices(spelling).nth(occurrence).unwrap().0 as u32;
        assert_eq!(
            tokens,
            vec![(spelling.to_owned(), start, start + spelling.len() as u32)],
            "{source}"
        );
        assert!(gaps.is_empty(), "{gaps:?}: {source}");
    }
}

#[test]
fn whole_boundary_packets_preserve_physical_tokens_and_explicit_refusals() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-boundaries.json"
    )).unwrap();
    let cases = corpus["records"].as_array().unwrap();
    assert_eq!(cases.len(), 42);
    let overlay: Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-authority-review-corrections.json"
    )).unwrap();
    let corrections = overlay["legacyCorrections"].as_array().unwrap();
    assert_eq!(corrections.len(), 6);
    for historical in cases {
        let case = corrections
            .iter()
            .find(|row| row["id"] == historical["id"])
            .unwrap_or(historical);
        assert_eq!(case["source"], historical["source"]);
        let source = case["source"].as_str().unwrap();
        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
        let template = descriptor.template.unwrap();
        let (tokens, gaps) = component_tokens(&template.content);
        let actual: Vec<_> = tokens
            .into_iter()
            .map(|(_, start, end)| {
                (
                    start + template.loc.start as u32,
                    end + template.loc.start as u32,
                )
            })
            .collect();
        let expected: Vec<_> = case["nativeMessages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|message| {
                (
                    message["start"].as_u64().unwrap() as u32,
                    message["end"].as_u64().unwrap() as u32,
                )
            })
            .collect();
        assert_eq!(actual, expected, "{}; gaps={gaps:?}", case["id"]);
        assert_eq!(
            !gaps.is_empty(),
            case.get("nativeGap")
                .or_else(|| case.get("expectedReferenceGap"))
                .unwrap()
                .as_bool()
                .unwrap(),
            "{}; gaps={gaps:?}",
            case["id"]
        );
    }
}

#[test]
fn unknown_slot_does_not_turn_children_into_component_globals() {
    let source = "<Child v-slot=\"{ $listeners } broken\">{{ $listeners }}</Child>{{ $listeners }}";
    let (tokens, gaps) = component_tokens(source);
    let start = source.rfind("$listeners").unwrap() as u32;
    assert_eq!(tokens, vec![("$listeners".to_owned(), start, start + 10)]);
    assert!(!gaps.is_empty());
}

#[test]
fn source_identity_failure_and_traversal_limits_publish_no_partial_facts() {
    let allocator = Allocator::default();
    let original = "{{ $listeners }}";
    let copied = original.to_owned();
    let (mut root, _, syntax) =
        Parser::new(&allocator, original).parse_with_template_syntax_facts();
    root.source = &copied;
    let packet = collect_template_references(&root, &syntax);
    assert!(packet.references().is_empty());
    assert!(!packet.gaps().is_empty());
    for source in [
        vize_l0::cstr!("{{ $listeners }}{}", "<i/>".repeat(MAX_UNITS)),
        vize_l0::cstr!(
            "{}{{{{ $listeners }}}}{}",
            "<div>".repeat(MAX_DEPTH + 2),
            "</div>".repeat(MAX_DEPTH + 2)
        ),
    ] {
        let (tokens, gaps) = component_tokens(&source);
        assert!(tokens.is_empty());
        assert!(
            gaps.iter()
                .any(|gap| gap.kind == TemplateReferenceGapKind::TraversalLimit)
        );
    }
}

#[test]
fn if_branch_keeps_original_condition_key_and_child_owners() {
    use vize_l0::{Box, Vec as ArenaVec};
    use vize_relief::{IfBranchNode, IfNode, PropNode, TemplateChildNode};
    let allocator = Allocator::default();
    let arena = &allocator;
    let source = "<div v-if=\"$listeners\" :key=\"$listeners\">{{ $listeners }}</div>";
    let (mut root, errors, syntax) =
        Parser::new(&allocator, source).parse_with_template_syntax_facts();
    assert!(errors.is_empty());
    let TemplateChildNode::Element(mut element) = root.children.pop().unwrap() else {
        panic!("original element")
    };
    let mut condition = None;
    let mut key = None;
    // Move the genuine original nodes through the structural If conversion;
    // no expression, source map, span or retained AST is manufactured here.
    for mut property in std::mem::replace(&mut element.props, ArenaVec::new_in(&arena)) {
        match &mut property {
            PropNode::Directive(directive) if directive.name == "if" => {
                condition = directive.exp.take();
            }
            PropNode::Directive(directive) if directive.name == "bind" => {
                key = Some(property);
            }
            _ => {}
        }
    }
    let mut branch = IfBranchNode::new(&allocator, condition, element.loc.clone());
    branch.user_key = key;
    branch.children = std::mem::replace(&mut element.children, ArenaVec::new_in(&arena));
    let mut conditional = IfNode::new(&allocator, element.loc.clone());
    conditional.branches.push(branch);
    root.children
        .push(TemplateChildNode::If(Box::new_in(conditional, &arena)));
    let packet = collect_template_references(&root, &syntax);
    let expected: Vec<_> = source
        .match_indices("$listeners")
        .map(|(start, _)| Span::new(start as u32, start as u32 + 10))
        .collect();
    assert_eq!(
        packet
            .references()
            .iter()
            .map(|reference| reference.span)
            .collect::<Vec<_>>(),
        expected
    );
    assert!(packet.gaps().is_empty(), "{:?}", packet.gaps());
}
