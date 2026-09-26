use super::{
    Result,
    schema::{Edit, Node, Reply, SCHEMA},
    walk,
};
use std::collections::BTreeSet;
use vize_davinci::id::NodeId;
use vize_l0::Allocator;
use vize_l1_to_l2::Lowered;
use vize_l2::op::Op;
use vize_l2::provenance::ProvenanceRecord;

pub(super) fn decode(json: &str) -> Result<Reply> {
    if json.len() > 1_048_576 {
        return Err("transform response exceeds 1 MiB".into());
    }
    let reply: Reply =
        serde_json::from_str(json).map_err(|e| format!("invalid transform response: {e}"))?;
    if reply.schema != SCHEMA {
        return Err("unsupported transform response schema".into());
    }
    if reply.edits.len() > 256 {
        return Err("transform exceeds 256 edits".into());
    }
    Ok(reply)
}

pub(super) fn validate(reply: &Reply, nodes: &[Node]) -> Result<()> {
    if reply.schema != SCHEMA || reply.edits.len() > 256 {
        return Err("cached transform has an unsupported schema or exceeds 256 edits".into());
    }
    let mut touched = BTreeSet::new();
    for Edit::Replace { node, name, value } in &reply.edits {
        if !allowed(name) {
            return Err(format!("transform attribute `{name}` is not supported"));
        }
        if !touched.insert((*node, name.as_str())) {
            return Err("duplicate transform attribute edit".into());
        }
        let owner = nodes
            .iter()
            .find(|owner| owner.id == *node)
            .ok_or_else(|| format!("transform node {node} is not a native element"))?;
        if owner.namespace != "html" {
            return Err("transform requires the HTML namespace".into());
        }
        if owner.tag == "template" {
            return Err("transform cannot edit a template carrier".into());
        }
        if owner.attrs.iter().filter(|attr| attr.name == *name).count() != 1 {
            return Err(format!(
                "transform attribute `{name}` must exist exactly once on node {node}"
            ));
        }
        if value
            .as_ref()
            .is_some_and(|v| v.len() > 4096 || v.chars().any(|c| c == '\0'))
        {
            return Err("transform value exceeds 4096 bytes or contains NUL".into());
        }
    }
    Ok(())
}

fn allowed(name: &str) -> bool {
    let simple = matches!(name, "class" | "id" | "title" | "role" | "alt");
    let prefixed = name
        .strip_prefix("data-")
        .or_else(|| name.strip_prefix("aria-"))
        .is_some_and(|tail| {
            !tail.is_empty()
                && tail
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        });
    simple || prefixed
}

/// Allocate semantic strings as authored entity-encoded L2 attribute values.
/// The canonical consumer performs its ordinary one decoding step.
fn encoded<'a>(allocator: &'a Allocator, value: &str) -> &'a str {
    let encoded = value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    allocator.alloc_str(&encoded)
}

#[derive(Debug)]
pub(super) struct Applied {
    pub owner_start: u32,
    pub name: String,
    pub value: Option<String>,
}

/// Called only after all edits validate, so a rejected reply leaves L2 intact.
pub(super) fn apply(lowered: &mut Lowered<'_>, reply: &Reply, plugin: &str) -> Vec<Applied> {
    let allocator = lowered.allocator;
    let mut applied = Vec::new();
    let mut records = Vec::new();
    let mut count = 0;
    walk::visit(&mut lowered.root, &mut count, &mut |id, op| {
        let Op::Element(owner) = op else {
            return;
        };
        for Edit::Replace { node, name, value } in &reply.edits {
            if *node != id {
                continue;
            }
            if let Some(attr) = owner.attributes.iter_mut().find(|attr| attr.name == name) {
                let before = format!("{}={:?}", attr.name, attr.value);
                attr.value = value.as_deref().map(|value| encoded(allocator, value));
                records.push(ProvenanceRecord {
                    rule: format!("plugin.transform:{plugin}").into(),
                    node: NodeId::from_index(id),
                    before: before.into(),
                    after: format!("{}={:?}", attr.name, attr.value).into(),
                    span: attr.span,
                });
                applied.push(Applied {
                    owner_start: owner.span.start,
                    name: name.clone(),
                    value: value.clone(),
                });
            }
        }
    });
    assert_eq!(
        count, lowered.op_count,
        "transform reader's page numbering diverged"
    );
    lowered.provenance.extend(records);
    applied
}
