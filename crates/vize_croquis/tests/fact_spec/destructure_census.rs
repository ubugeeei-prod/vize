//! Exact added relations exposed by authored destructuring identifier spans.

use vize_armature::Parser;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{Allocator, CompactString, cstr};
use vize_croquis::Croquis;
use vize_croquis::facts::{
    BindingFact, BindingKey, Bindings, BindingsTable, CroquisFacts, Demand, FactConsumer,
    FactGroup, UnusedBindings,
};

struct DestructureReader;

impl FactConsumer for DestructureReader {
    const NAME: &'static str = "ts-34/destructure-census";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID).with(UnusedBindings::ID);
}

fn analyze_battery(name: &str) -> Result<(Croquis, CompactString), CompactString> {
    let source = super::battery::SOURCES
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .ok_or_else(|| cstr!("missing unchanged battery {name}"))?
        .1;
    let descriptor = parse_sfc(source, SfcParseOptions::default())
        .map_err(|error| cstr!("{name}: {error:?}"))?;
    let allocator = Allocator::new();
    let template = descriptor
        .template
        .as_ref()
        .map(|block| {
            let (root, errors) = Parser::new(&allocator, block.content.as_ref()).parse();
            if errors.is_empty() {
                Ok(root)
            } else {
                Err(cstr!("{name}: invalid template: {errors:?}"))
            }
        })
        .transpose()?;
    let croquis = analyze_sfc_descriptor(
        &descriptor,
        template.as_ref(),
        SfcCroquisOptions::full().with_unused_bindings(),
    );
    let script = descriptor
        .script_setup
        .as_ref()
        .ok_or_else(|| cstr!("{name}: missing setup source"))?;
    Ok((croquis, CompactString::new(&script.content)))
}

fn leaf_span(
    script: &str,
    name: &str,
    anchor: &str,
    prefix: usize,
) -> Result<(u32, u32), CompactString> {
    let start = script
        .find(anchor)
        .ok_or_else(|| cstr!("missing authored declaration anchor {anchor}"))?
        + prefix;
    if script.get(start..start + name.len()) != Some(name) {
        return Err(cstr!(
            "{name}: declaration anchor does not select its identifier"
        ));
    }
    Ok((
        u32::try_from(start).map_err(|error| cstr!("{error}"))?,
        u32::try_from(start + name.len()).map_err(|error| cstr!("{error}"))?,
    ))
}

pub(super) fn assert_added_pattern_witnesses() -> Result<u64, CompactString> {
    let (croquis, script) = analyze_battery("patterns")?;
    let mut facts = CroquisFacts::new(&croquis);
    let view = facts.prepare::<DestructureReader>();
    let table = view
        .get::<UnusedBindings>()
        .map_err(|error| cstr!("{error:?}"))?;
    let observed = table
        .iter()
        .map(|(name, fact)| (name.as_str(), fact.span))
        .collect::<Vec<_>>();
    let mut expected = Vec::new();
    for (name, anchor, prefix) in [
        ("third", "third] = useBar()", 0),
        ("f", "{ f } = ()", 2),
        ("m", "[m, ...others]", 1),
        ("others", "...others", 3),
        ("modifiers", "modifiers] = defineModel()", 0),
    ] {
        expected.push((name, leaf_span(&script, name, anchor, prefix)?));
    }
    expected.sort_unstable();
    eprintln!("TS-34 added destructure unused witnesses: {observed:?}");
    if observed != expected {
        return Err(cstr!(
            "only these five pattern declarations supplement the old census: actual {observed:?}, expected {expected:?}"
        ));
    }
    u64::try_from(expected.len()).map_err(|error| cstr!("{error}"))
}

pub(super) fn assert_added_span_only_binding() -> Result<u64, CompactString> {
    let (croquis, script) = analyze_battery("props-destructure")?;
    let mut facts = CroquisFacts::new(&croquis);
    let view = facts.prepare::<DestructureReader>();
    let table = view.get::<Bindings>().map_err(|error| cstr!("{error:?}"))?;
    if !table.is_script_setup() {
        return Err(cstr!("the nested props local must belong to setup"));
    }
    let observed = table
        .iter()
        .filter_map(|(key, fact)| match key {
            BindingKey::Name(name) if fact.kind.is_none() => Some((name.as_str(), fact.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    let expected = [(
        "deep",
        BindingFact {
            kind: None,
            span: Some(leaf_span(&script, "deep", "nested: { deep }", 10)?),
            prop_key: None,
        },
    )];
    eprintln!("TS-34 added span-only binding witness: {observed:?}");
    if observed != expected {
        return Err(cstr!(
            "the nested props local owns only its span: actual {observed:?}, expected {expected:?}"
        ));
    }
    u64::try_from(observed.len()).map_err(|error| cstr!("{error}"))
}
