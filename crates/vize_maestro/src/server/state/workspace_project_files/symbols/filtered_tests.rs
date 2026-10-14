//! Disabled matches cannot consume the session's global result limit.
use super::{FxHashMap, Url, collect_sources_filtered};
use vize_l0::cstr;

#[test]
fn filter_precedes_ranking_limit_and_imports_only_sources_with_matches() {
    let enabled = Url::parse("file:///workspace/b/enabled.ts").unwrap();
    let unmatched = Url::parse("file:///workspace/idle/unmatched.ts").unwrap();
    let mut sources = FxHashMap::default();
    for index in 0..150 {
        let uri = Url::parse(&cstr!("file:///workspace/a/disabled-{index}.ts")).unwrap();
        sources.insert(uri, "export const target = 1;".to_owned());
    }
    sources.insert(
        enabled.clone(),
        "export const targetEnabled = 1;".to_owned(),
    );
    sources.insert(unmatched.clone(), "export const unrelated = 1;".to_owned());
    let visited = std::sync::Mutex::new(Vec::new());
    let symbols = collect_sources_filtered(
        Vec::new(),
        &sources,
        &[],
        "target",
        |_| None,
        |uri| {
            visited.lock().unwrap().push(uri.clone());
            *uri == enabled
        },
    );
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].location.uri, enabled);
    assert_eq!(symbols[0].name, "targetEnabled");
    let visited = visited.into_inner().unwrap();
    assert_eq!(visited.len(), 151);
    assert!(!visited.contains(&unmatched));
}
