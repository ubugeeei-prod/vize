//! Publish the same owned provenance with a static spelling for each lattice value.
use super::StaticAnalysis;
use crate::analysis::static_facts::StaticLevel;
use crate::provenance::{ProvenanceRecord, ProvenanceSink};
use vize_l0::{Span, String, id::NodeId};

// The flags are indexed in the order they appear in the provenance record.
// A long CompactString constructed from a static literal borrows its bytes
// for the life of the program, including when records are cloned or moved.
macro_rules! descriptions {
    ($level:literal) => {
        [
            concat!("level=", $level, " props=false nested=false native=false"),
            concat!("level=", $level, " props=false nested=false native=true"),
            concat!("level=", $level, " props=false nested=true native=false"),
            concat!("level=", $level, " props=false nested=true native=true"),
            concat!("level=", $level, " props=true nested=false native=false"),
            concat!("level=", $level, " props=true nested=false native=true"),
            concat!("level=", $level, " props=true nested=true native=false"),
            concat!("level=", $level, " props=true nested=true native=true"),
        ]
    };
}

fn description(fact: StaticAnalysis) -> String {
    let spellings = match fact.level {
        StaticLevel::NotStatic => &descriptions!("not-static"),
        StaticLevel::HasDynamicText => &descriptions!("dynamic-text"),
        StaticLevel::FullyStatic => &descriptions!("fully-static"),
    };
    let flags = (usize::from(fact.props_hoistable) << 2)
        | (usize::from(fact.nested_static) << 1)
        | usize::from(fact.native_descendants);
    #[expect(
        clippy::indexing_slicing,
        reason = "three flag bits index eight spellings"
    )]
    String::const_new(spellings[flags])
}

pub(super) fn record(
    provenance: &mut impl ProvenanceSink,
    id: NodeId,
    owner: &str,
    span: Span,
    fact: StaticAnalysis,
) {
    provenance.record(|| ProvenanceRecord {
        rule: vize_l0::String::from("pass.hoist-static.fact"),
        node: Some(id),
        before: vize_l0::String::from(owner),
        after: description(fact),
        span,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_description_preserves_the_original_bytes_and_uses_static_storage() {
        for (level, name) in [
            (StaticLevel::NotStatic, "not-static"),
            (StaticLevel::HasDynamicText, "dynamic-text"),
            (StaticLevel::FullyStatic, "fully-static"),
        ] {
            for flags in 0..16 {
                let fact = StaticAnalysis {
                    level,
                    props_hoistable: flags & 4 != 0,
                    nested_static: flags & 2 != 0,
                    native_descendants: flags & 1 != 0,
                    foreign: flags & 8 != 0,
                };
                let expected = vize_l0::cstr!(
                    "level={name} props={} nested={} native={}",
                    fact.props_hoistable,
                    fact.nested_static,
                    fact.native_descendants
                );
                assert_eq!(description(fact).as_static_str(), Some(expected.as_str()));
            }
        }
    }
}
