//! Format one provenance record in the caller's sink, preserving lazy construction.

macro_rules! record {
    ($provenance:ident, $id:ident, $owner:ident, $span:ident, $fact:ident) => {
        $provenance.record(|| {
            let level = match $fact.level {
                $crate::analysis::static_facts::StaticLevel::NotStatic => "not-static",
                $crate::analysis::static_facts::StaticLevel::HasDynamicText => "dynamic-text",
                $crate::analysis::static_facts::StaticLevel::FullyStatic => "fully-static",
            };
            $crate::provenance::ProvenanceRecord {
                rule: vize_l0::String::from("pass.hoist-static.fact"),
                node: Some($id),
                before: vize_l0::String::from($owner),
                after: vize_l0::cstr!(
                    "level={level} props={} nested={} native={}",
                    $fact.props_hoistable,
                    $fact.nested_static,
                    $fact.native_descendants
                ),
                span: $span,
            }
        })
    };
}
pub(super) use record;
