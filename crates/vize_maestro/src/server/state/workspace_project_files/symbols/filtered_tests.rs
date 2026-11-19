//! Disabled matches cannot consume the session's global result limit.
use std::sync::Arc;

use super::{SymbolCacheSnapshot, Url, collect_sources_filtered};
use vize_l0::cstr;

#[test]
fn filter_precedes_ranking_limit_and_imports_only_sources_with_matches() {
    let enabled = Url::parse("file:///workspace/b/enabled.ts").unwrap();
    let unmatched = Url::parse("file:///workspace/idle/unmatched.ts").unwrap();
    let mut sources = Vec::new();
    for index in 0..150 {
        let uri = Url::parse(&cstr!("file:///workspace/a/disabled-{index}.ts")).unwrap();
        sources.push((uri, Arc::new("export const target = 1;".to_owned())));
    }
    sources.push((
        enabled.clone(),
        Arc::new("export const targetEnabled = 1;".to_owned()),
    ));
    sources.push((
        unmatched.clone(),
        Arc::new("export const unrelated = 1;".to_owned()),
    ));
    let expected_visited = sources
        .iter()
        .take(151)
        .map(|(uri, _)| uri.clone())
        .collect::<Vec<_>>();
    let visited = std::sync::Mutex::new(Vec::new());
    let symbols = collect_sources_filtered(
        &[],
        &sources,
        &[],
        "target",
        |_| Option::<String>::None,
        |uri| {
            visited.lock().unwrap().push(uri.clone());
            *uri == enabled
        },
        None,
    );
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].location.uri, enabled);
    assert_eq!(symbols[0].name, "targetEnabled");
    let visited = visited.into_inner().unwrap();
    assert_eq!(visited, expected_visited);

    let cache = dashmap::DashMap::new();
    let cached_visited = std::sync::Mutex::new(Vec::new());
    let cached = collect_sources_filtered(
        &[],
        &sources,
        &[],
        "target",
        |_| Option::<String>::None,
        |uri| {
            cached_visited.lock().unwrap().push(uri.clone());
            *uri == enabled
        },
        Some(SymbolCacheSnapshot {
            documents: &cache,
            disk_generation: None,
        }),
    );
    assert_eq!(cached, symbols);
    let cached_visited = cached_visited.into_inner().unwrap();
    assert_eq!(cached_visited, expected_visited);
}
