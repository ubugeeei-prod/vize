//! Validate the supported backend shape after the generic graph verifier.
//! Indexes are built once; source order must agree with explicit S3 edges.

use vize_carton::{FxHashMap, FxHashSet};
use vize_s3::{
    op::{EdgeKind, OpId, OpKind, Phase, Program, RegionId},
    operand::{Operand, OperandRole as Role, ValueKind},
};

use super::{Binding, Content, NativeArtifact, Node};
use crate::s3::{AdmissionFailure, LegacyReason};

type Result<T> = core::result::Result<T, AdmissionFailure>;

pub(super) fn admit<'a>(program: &Program<'a>) -> Result<NativeArtifact<'a>> {
    if program.phase != Phase::Built {
        return Err(LegacyReason::Operation.into());
    }
    let mut operands: FxHashMap<OpId, std::vec::Vec<&Operand<'a>>> = FxHashMap::default();
    for operand in &program.operands {
        operands.entry(operand.op).or_default().push(operand);
    }
    let mut nodes = std::vec::Vec::new();
    let mut indexes = FxHashMap::default();
    let mut regions: FxHashMap<RegionId, std::vec::Vec<OpId>> = FxHashMap::default();
    let mut binding_order: FxHashMap<OpId, std::vec::Vec<OpId>> = FxHashMap::default();
    let mut bindings = std::vec::Vec::new();
    for op in &program.ops {
        let values = operands.get(&op.id).map_or(&[][..], |v| &v[..]);
        let content = match op.kind {
            OpKind::InsertNode => element(values)?,
            OpKind::SetText if values.len() == 1 && values[0].role == Role::Text => {
                if values[0].target.is_some()
                    || values[0].region.is_some()
                    || values[0].name.is_some()
                {
                    return Err(LegacyReason::Structure.into());
                }
                let value = values[0].value;
                match value.kind {
                    ValueKind::Literal if !value.text.contains('&') => Content::Text {
                        value: value.text,
                        dynamic: false,
                    },
                    ValueKind::Js if reference(value.text) => Content::Text {
                        value: value.text.trim(),
                        dynamic: true,
                    },
                    _ => return Err(LegacyReason::ExpressionOrEncoding.into()),
                }
            }
            OpKind::SetProp | OpKind::SetEvent => {
                let (target, binding) = binding(values, op.kind)?;
                if op.effect.is_none() {
                    return Err(AdmissionFailure::Invalid(
                        "binding lacks its dynamic partition",
                    ));
                }
                binding_order.entry(target).or_default().push(op.id);
                bindings.push((target, op.region, binding));
                continue;
            }
            _ => return Err(LegacyReason::Operation.into()),
        };
        let dynamic = matches!(content, Content::Text { dynamic: true, .. });
        if dynamic != op.effect.is_some() {
            return Err(AdmissionFailure::Invalid(
                "native payload disagrees with its partition",
            ));
        }
        indexes.insert(op.id, (nodes.len(), op.region));
        regions.entry(op.region).or_default().push(op.id);
        nodes.push(Node {
            content,
            children: std::vec::Vec::new(),
            bindings: std::vec::Vec::new(),
        });
    }
    check_order(program, &regions, &binding_order)?;
    let mut names = FxHashSet::default();
    for (id, (index, _)) in &indexes {
        if let Content::Element { attributes, .. } = &nodes[*index].content {
            names.extend(attributes.iter().map(|(name, _)| (*id, false, *name)));
        }
    }
    for (target, region, binding) in bindings {
        let Some(&(index, target_region)) = indexes.get(&target) else {
            return Err(LegacyReason::Structure.into());
        };
        let node = &mut nodes[index];
        if !matches!(node.content, Content::Element { .. }) || region != target_region {
            return Err(AdmissionFailure::Invalid(
                "binding target is outside its native region",
            ));
        }
        if !names.insert((target, binding.event, binding.name)) {
            return Err(LegacyReason::Binding.into());
        }
        node.bindings.push(binding);
    }
    let mut owners = FxHashSet::default();
    let mut parents = std::vec![None; nodes.len()];
    for region in &program.regions {
        let Some(owner) = region.owner else { continue };
        let Some(&(index, _)) = indexes.get(&owner) else {
            return Err(LegacyReason::Structure.into());
        };
        if !matches!(nodes[index].content, Content::Element { .. }) || !owners.insert(owner) {
            return Err(LegacyReason::Structure.into());
        }
        let children: std::vec::Vec<_> = regions
            .get(&region.id)
            .into_iter()
            .flatten()
            .map(|id| indexes[id].0)
            .collect();
        if children.iter().any(|child| *child <= index)
            || children.windows(2).any(|pair| {
                matches!(nodes[pair[0]].content, Content::Text { .. })
                    && matches!(nodes[pair[1]].content, Content::Text { .. })
            })
            || matches!(nodes[index].content, Content::Element { tag, .. } if vize_carton::is_void_tag(tag) && !children.is_empty())
        {
            return Err(LegacyReason::Structure.into());
        }
        for child in &children {
            parents[*child] = Some(index);
        }
        nodes[index].children = children;
    }
    for (id, (index, _)) in &indexes {
        if matches!(nodes[*index].content, Content::Element { .. }) && !owners.contains(id) {
            return Err(LegacyReason::Structure.into());
        }
    }
    let mut button_ancestor = std::vec![false; nodes.len()];
    for (index, node) in nodes.iter().enumerate() {
        let inherited = parents[index].is_some_and(|parent| button_ancestor[parent]);
        let button = matches!(node.content, Content::Element { tag: "button", .. });
        // HTML parsing closes an earlier button instead of nesting a new one.
        if inherited && button {
            return Err(LegacyReason::Structure.into());
        }
        button_ancestor[index] = inherited || button;
    }
    let roots: std::vec::Vec<_> = regions
        .get(&RegionId::ROOT)
        .into_iter()
        .flatten()
        .map(|id| indexes[id].0)
        .collect();
    // Root text and multiple roots need separate template/fragment contracts.
    if roots.len() != 1 || !matches!(nodes[roots[0]].content, Content::Element { .. }) {
        return Err(LegacyReason::Structure.into());
    }
    Ok(NativeArtifact {
        nodes,
        root: roots[0],
    })
}

fn element<'a>(values: &[&Operand<'a>]) -> Result<Content<'a>> {
    if values.iter().any(|value| value.role == Role::Comment) {
        return Err(LegacyReason::Operation.into());
    }
    let tag = one(values, Role::Tag)?;
    let namespace = one(values, Role::Namespace)?;
    if tag.value.kind != ValueKind::Literal
        || namespace.value.kind != ValueKind::Literal
        || namespace.value.text != "html"
        // These HTML tags have ordinary template parsing. Parser-context
        // elements (tables, raw text, select, templates, namespaces) require a
        // separate contract before their child indexes can be materialized.
        || !matches!(tag.value.text,
            "div" | "span" | "main" | "section" | "article" | "header" | "footer"
            | "nav" | "aside" | "button" | "strong" | "em" | "b" | "i" | "small"
            | "label" | "input" | "img" | "br" | "hr")
    {
        return Err(LegacyReason::Element.into());
    }
    let mut attributes = std::vec::Vec::new();
    let mut names = FxHashSet::default();
    for value in values {
        if value.target.is_some() || value.region.is_some() {
            return Err(LegacyReason::Structure.into());
        }
        match value.role {
            Role::Tag | Role::Namespace if value.name.is_none() => {}
            Role::Attribute => {
                let name = value.name.ok_or(LegacyReason::Binding)?;
                if !attribute_name(name) || !names.insert(name) {
                    return Err(LegacyReason::Binding.into());
                }
                let text = match value.value.kind {
                    ValueKind::Absent => None,
                    ValueKind::Literal if !value.value.text.contains('&') => Some(value.value.text),
                    _ => return Err(LegacyReason::ExpressionOrEncoding.into()),
                };
                attributes.push((name, text));
            }
            _ => return Err(LegacyReason::Structure.into()),
        }
    }
    Ok(Content::Element {
        tag: tag.value.text,
        attributes,
    })
}

fn binding<'a>(values: &[&Operand<'a>], kind: OpKind) -> Result<(OpId, Binding<'a>)> {
    let binding = one(values, Role::BindingKind)?;
    let event = kind == OpKind::SetEvent;
    // SetProp is also the generic model/sync op. Select its semantic family
    // before requiring the narrower bind/on operand schema.
    if binding.value.kind != ValueKind::Literal
        || binding.value.text != if event { "on" } else { "bind" }
    {
        return Err(LegacyReason::Binding.into());
    }
    let name = one(values, Role::Name)?;
    let value = one(values, Role::Value)?;
    if values.len() != 3 {
        return Err(LegacyReason::Binding.into());
    }
    let target = binding.target.ok_or(LegacyReason::Structure)?;
    if values
        .iter()
        .any(|v| v.target != Some(target) || v.region.is_some() || v.name.is_some())
        || name.value.kind != ValueKind::Literal
        || if event {
            name.value.text != "click"
        } else {
            !attribute_name(name.value.text)
        }
    {
        return Err(LegacyReason::Binding.into());
    }
    if value.value.kind != ValueKind::Js || !reference(value.value.text) {
        return Err(LegacyReason::ExpressionOrEncoding.into());
    }
    Ok((
        target,
        Binding {
            name: name.value.text,
            value: value.value.text.trim(),
            event,
        },
    ))
}

fn one<'b, 'a>(values: &'b [&Operand<'a>], role: Role) -> Result<&'b Operand<'a>> {
    let mut found = values.iter().filter(|v| v.role == role);
    let value = found
        .next()
        .ok_or(AdmissionFailure::Invalid("missing required native operand"))?;
    if found.next().is_some() {
        return Err(AdmissionFailure::Invalid("duplicate native operand role"));
    }
    Ok(value)
}

fn attribute_name(name: &str) -> bool {
    !matches!(
        name,
        "key" | "ref" | "ref_for" | "ref_key" | "is" | "innerHTML" | "textContent"
    ) && !name.starts_with("on")
        && name.starts_with(|c: char| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn reference(value: &str) -> bool {
    // A deliberately narrower grammar than JavaScript. The S3 producer has
    // already classified it as JS; no reparsing or opaque reinterpretation.
    let value = value.trim();
    !matches!(
        value.split('.').next(),
        Some("this" | "true" | "false" | "null" | "$event")
    ) && value.split('.').all(|part| {
        part.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '$')
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    })
}

fn check_order(
    program: &Program<'_>,
    regions: &FxHashMap<RegionId, std::vec::Vec<OpId>>,
    bindings: &FxHashMap<OpId, std::vec::Vec<OpId>>,
) -> Result<()> {
    let mut expected = FxHashSet::default();
    for ops in regions.values().chain(bindings.values()) {
        expected.extend(
            ops.windows(2)
                .map(|pair| (pair[0], pair[1], EdgeKind::DomOrder)),
        );
    }
    let dynamic: std::vec::Vec<_> = program
        .ops
        .iter()
        .filter(|op| op.effect.is_some())
        .collect();
    expected.extend(
        dynamic
            .windows(2)
            .map(|pair| (pair[0].id, pair[1].id, EdgeKind::EffectOrder)),
    );
    if program.edges.len() != expected.len()
        || program
            .edges
            .iter()
            .any(|edge| !expected.contains(&(edge.from, edge.to, edge.kind)))
    {
        return Err(AdmissionFailure::Invalid(
            "native order differs from S3 ordering edges",
        ));
    }
    Ok(())
}
