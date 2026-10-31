//! Real L1 lowering publishes the canonical L2 type through every old alias.

use crate::support;
use vize_l0::{Span, String, side_table::SideTable};
use vize_l1_to_l2::{lower, pass};
use vize_l2::analysis::if_facts::{BranchKey, BranchKeyKind, IfFacts};
use vize_l2::dump::Op;

fn span(source: &str, token: &str) -> Result<Span, String> {
    let at = source
        .find(token)
        .ok_or_else(|| String::from("actual source token"))?;
    let at = u32::try_from(at).map_err(|_| String::from("fixture offset"))?;
    let len = u32::try_from(token.len()).map_err(|_| String::from("fixture length"))?;
    Ok(Span::new(at, at + len))
}

#[test]
fn real_lowering_and_pass_aliases_borrow_the_canonical_fact_type() -> Result<(), String> {
    let source =
        "<p v-if='ready' :title='label' :key='id' key='late'>x</p><slot v-else key='fallback'/>";
    support::with_transformed(source, |lowered, folio, facts, _| {
        let canonical: &SideTable<IfFacts> = &lowered.if_facts;
        let old_lower: &SideTable<lower::IfFacts> = canonical;
        let old_pass: &SideTable<pass::IfFacts> = canonical;
        let old_vif: &SideTable<pass::vif::IfFacts> = canonical;
        assert!(core::ptr::eq(canonical, old_lower));
        assert!(core::ptr::eq(canonical, old_pass));
        assert!(core::ptr::eq(canonical, old_vif));
        let mirrored: &SideTable<IfFacts> = &facts.if_facts;
        assert_eq!(mirrored, canonical);
        let entries = canonical.sorted_entries();
        let [(node, row)] = entries.as_slice() else {
            return Err(String::from("one real If fact"));
        };
        assert_eq!(node.index(), 0);
        assert_eq!(
            **row,
            IfFacts {
                branches: vec![
                    Some(BranchKey {
                        kind: BranchKeyKind::Dynamic {
                            source: "id".into(),
                            bind_index: Some(1)
                        },
                        span: span(source, ":key='id'")?,
                    }),
                    Some(BranchKey {
                        kind: BranchKeyKind::Static(Some("fallback".into())),
                        span: span(source, "key='fallback'")?,
                    }),
                ]
            }
        );
        let [Some(dynamic), Some(_)] = row.branches.as_slice() else {
            return Err(String::from("two ordered authored keys"));
        };
        let lower_key: &lower::BranchKey = dynamic;
        let pass_key: &pass::BranchKey = dynamic;
        let lower_kind: &lower::BranchKeyKind = &dynamic.kind;
        let pass_kind: &pass::BranchKeyKind = &dynamic.kind;
        assert!(core::ptr::eq(dynamic, lower_key));
        assert!(core::ptr::eq(dynamic, pass_key));
        assert!(core::ptr::eq(&dynamic.kind, lower_kind));
        assert!(core::ptr::eq(&dynamic.kind, pass_kind));
        let [Op::If(owner)] = folio.ops.as_slice() else {
            return Err(String::from("actual If folio"));
        };
        let [first, last] = owner.branches.as_slice() else {
            return Err(String::from("actual two branches"));
        };
        let [Op::Element(element)] = first.ops.as_slice() else {
            return Err(String::from("actual Element carrier"));
        };
        let [attribute] = element.attributes.as_slice() else {
            return Err(String::from("later static key remains"));
        };
        assert_eq!(attribute.name, "key");
        assert_eq!(attribute.value.as_deref(), Some("late"));
        assert_eq!(attribute.span, span(source, "key='late'")?);
        assert_eq!(element.bindings.len(), 2);
        let [Op::Slot(slot)] = last.ops.as_slice() else {
            return Err(String::from("actual Slot carrier"));
        };
        assert_eq!(slot.attributes, vec![]);
        assert_eq!(lowered.diagnostics, vec![]);
        Ok(())
    })
}
