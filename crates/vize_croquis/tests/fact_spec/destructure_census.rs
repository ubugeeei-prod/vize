//! The span repair exposes five previously unobservable unused declarations.

use vize_armature::Parser;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::Allocator;
use vize_croquis::facts::{CroquisFacts, Demand, FactConsumer, FactGroup, UnusedBindings};

struct UnusedReader;

impl FactConsumer for UnusedReader {
    const NAME: &'static str = "ts-34/destructure-census";
    const DEMAND: Demand = Demand::NONE.with(UnusedBindings::ID);
}

pub(super) fn assert_added_pattern_witnesses() -> u64 {
    let source = super::battery::SOURCES
        .iter()
        .find(|(name, _)| *name == "patterns")
        .expect("the unchanged committed pattern battery")
        .1;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("authored SFC");
    let allocator = Allocator::new();
    let template = descriptor.template.as_ref().map(|block| {
        let (root, errors) = Parser::new(&allocator, block.content.as_ref()).parse();
        assert!(errors.is_empty());
        root
    });
    let croquis = analyze_sfc_descriptor(
        &descriptor,
        template.as_ref(),
        SfcCroquisOptions::full().with_unused_bindings(),
    );
    let mut facts = CroquisFacts::new(&croquis);
    let table = facts
        .prepare::<UnusedReader>()
        .get::<UnusedBindings>()
        .expect("declared unused relation");
    let observed = table
        .iter()
        .map(|(name, fact)| (name.as_str(), fact.span))
        .collect::<Vec<_>>();
    let script = descriptor.script_setup.as_ref().expect("setup source");
    let mut expected = [
        ("third", "third] = useBar()", 0),
        ("f", "{ f } = ()", 2),
        ("m", "[m, ...others]", 1),
        ("others", "...others", 3),
        ("modifiers", "modifiers] = defineModel()", 0),
    ]
    .map(|(name, anchor, prefix)| {
        let start = script
            .content
            .find(anchor)
            .expect("exact declaration anchor")
            + prefix;
        let span = (start as u32, (start + name.len()) as u32);
        assert_eq!(script.content.get(start..start + name.len()), Some(name));
        (name, span)
    });
    expected.sort_unstable();
    eprintln!("TS-34 added destructure unused witnesses: {observed:?}");
    assert_eq!(
        observed, expected,
        "only these five pattern declarations supplement the old census"
    );
    u64::try_from(expected.len()).expect("finite witness count")
}
