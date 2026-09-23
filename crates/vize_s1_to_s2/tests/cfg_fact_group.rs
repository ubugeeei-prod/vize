//! TS-35 for the template-complexity fact group (P4-9a on the P4-1a API):
//! the group computes once per artifact however many consumers read it,
//! a declared consumer reads it without tripping the detector, an
//! undeclared one gets the exact refusal, and the table equals the pass
//! run directly — shifted into its file, the file-absolute lowering.

#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use vize_davinci::fact::{
    Demand, FactConsumer, FactError, FactGroup, FactManager, produced_count, undeclared_accesses,
};
use vize_s0::{Allocator, SourceRoot};
use vize_s1_to_s2::pass::cfg::{self, TEMPLATE_FACTS, TemplateComplexityGroup};
use vize_s1_to_s2::{lower, lower_source_block};

struct Rule;
impl FactConsumer for Rule {
    const NAME: &'static str = "test/rule";
    const DEMAND: Demand = Demand::NONE.with(TemplateComplexityGroup::ID);
}

struct Report;
impl FactConsumer for Report {
    const NAME: &'static str = "test/report";
    const DEMAND: Demand = Demand::NONE.with(TemplateComplexityGroup::ID);
}

struct Stranger;
impl FactConsumer for Stranger {
    const NAME: &'static str = "test/stranger";
    const DEMAND: Demand = Demand::NONE;
}

const TEMPLATE: &str = r#"<p v-if="a && b">x</p><li v-for="x in xs">{{ x ? 1 : 2 }}</li>"#;

/// One test body: the counters are process-global, so every assertion that
/// reads them runs in this binary's single test.
#[test]
fn the_group_is_computed_once_read_under_demand_and_equals_the_pass() {
    let produced = produced_count(TemplateComplexityGroup::ID);
    let undeclared = undeclared_accesses();

    let mut manager = FactManager::new(&TEMPLATE_FACTS);
    let rule_facts = manager
        .prepare::<Rule>(TEMPLATE)
        .unwrap()
        .get::<TemplateComplexityGroup>()
        .unwrap()
        .get(&())
        .cloned()
        .unwrap();
    let report_facts = manager
        .prepare::<Report>(TEMPLATE)
        .unwrap()
        .get::<TemplateComplexityGroup>()
        .unwrap()
        .get(&())
        .cloned()
        .unwrap();
    assert_eq!(rule_facts, report_facts);
    // Two consumers, one artifact, one production.
    assert_eq!(produced_count(TemplateComplexityGroup::ID), produced + 1);
    assert_eq!(undeclared_accesses(), undeclared);

    // The table is the pass's product over the same template.
    let allocator = Allocator::new();
    let (tree, errors) = vize_s1::parse(&allocator, TEMPLATE);
    assert_eq!(rule_facts, cfg::run(&lower(&allocator, &tree, &errors)));
    assert_eq!((rule_facts.cyclomatic, rule_facts.cognitive), (5, 5));

    // An undeclared reader gets the exact refusal (debug detector).
    #[cfg(debug_assertions)]
    {
        let refused = manager
            .view::<Stranger>()
            .get::<TemplateComplexityGroup>()
            .err();
        assert_eq!(
            refused,
            Some(FactError::Undeclared {
                consumer: "test/stranger",
                group: TemplateComplexityGroup::ID,
            })
        );
        assert_eq!(undeclared_accesses(), undeclared + 1);
    }

    // Placed in a file, the facts equal the file-absolute lowering.
    let source =
        "<script setup>\nconst a = 1\n</script>\n<template>X</template>\n".replace('X', TEMPLATE);
    let start = source.find("<template>").unwrap() + "<template>".len();
    let end = start + TEMPLATE.len();
    let placed = cfg::template_facts::<Rule>(&source, start as u32, end as u32).unwrap();
    let root = SourceRoot::new(&source).unwrap();
    let block = root.block(&source[start..end], start as u32).unwrap();
    let (tree, errors) = vize_s1::parse(&allocator, &source[start..end]);
    assert_eq!(
        placed,
        cfg::run(&lower_source_block(&allocator, &tree, &errors, block))
    );
    assert_eq!(produced_count(TemplateComplexityGroup::ID), produced + 2);
}
