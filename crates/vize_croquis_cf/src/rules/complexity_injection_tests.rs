use super::ProvideInjectMatch;
use super::complexity::summarize_complexity;
use crate::analyzer::CrossFileResult;
use crate::registry::ModuleRegistry;
use vize_carton::CompactString;
use vize_croquis::Croquis;

#[test]
fn plain_provide_inject_matches_do_not_count_as_reactive_edges() {
    let mut registry = ModuleRegistry::new();
    let (provider, _) = registry.register("Provider.vue", "", Croquis::new());
    let (consumer, _) = registry.register("Consumer.vue", "", Croquis::new());
    let result = CrossFileResult {
        provide_inject_matches: vec![ProvideInjectMatch {
            provider,
            consumer,
            key: CompactString::new("theme"),
            key_identity: CompactString::new("string:theme"),
            path: vec![provider, consumer],
            type_match: None,
            provide_offset: 10,
            inject_offset: 20,
        }],
        ..CrossFileResult::default()
    };

    let report = summarize_complexity(&registry, &result);

    assert_eq!(report.input.provide_inject_reference_count, 1);
    assert_eq!(report.input.reactive_edge_count, 0);
    assert_eq!(report.dimensions.reactive_graph, 0);
    assert_eq!(report.dimensions.provide_inject, 1);
}
