//! The JS plugin host's pure half, pinned by exact equality (P4-16).
#![expect(clippy::string_slice, reason = "tests assert by panicking")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]

use vize_davinci::fact::{Demand, FactManager, FactProducer};

use super::batch::{PluginSpec, build_batch, diagnostics};
use super::document::PluginDocument;
use super::error::HostError;
use super::facts::{REGISTRY, TemplateScopes, resolve_demands};
use super::plugin_cache::{
    PluginCache, PluginCacheInput, content_key_for_build, validate_cache_inputs,
};

const TODOS: &str = "<template>\n  <ul>\n    <li v-for=\"(todo, i) in todos\" :key=\"i\">{{ todo.title }}</li>\n  </ul>\n</template>\n";

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

fn spec<'p>(visit: Option<&'p [String]>, demands: &'p [String]) -> PluginSpec<'p> {
    PluginSpec {
        name: "team",
        version: "1.0.0",
        fingerprint: "f",
        visit,
        demands,
    }
}

fn content_key(source: &str, filename: &str, spec: &PluginSpec<'_>) -> String {
    super::plugin_cache::content_key(source, filename, spec, &[]).unwrap()
}

#[test]
fn positions_clamp_mid_character_and_past_end_offsets() {
    let document = PluginDocument {
        filename: "Offsets.vue".to_owned(),
        source: "a\né🦀".to_owned(),
        nodes: Vec::new(),
        scopes: Vec::new(),
    };

    assert_eq!(document.position(3), (2, 1)); // Inside the two-byte é.
    assert_eq!(document.position(4), (2, 2)); // After é.
    assert_eq!(document.position(7), (2, 2)); // Inside the four-byte crab.
    assert_eq!(document.position(8), (2, 4)); // After its two UTF-16 units.
    assert_eq!(document.position(u32::MAX), (2, 4));
}

#[test]
fn the_page_is_flattened_in_s2_id_order() {
    let document = PluginDocument::build(TODOS, "Todos.vue").expect("splits");
    let rows: Vec<_> = document
        .nodes
        .iter()
        .map(|n| {
            (
                n.id,
                n.parent,
                n.kind,
                n.start,
                n.end,
                n.name.as_deref(),
                n.value.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            (0, None, "ui.element", 13, 91, Some("ul"), None),
            (1, Some(0), "ui.for", 22, 83, None, Some("todos")),
            (2, Some(1), "ui.element", 22, 83, Some("li"), None),
            (3, Some(2), "ui.bind", 53, 61, Some("key"), Some("i")),
            (
                4,
                Some(2),
                "ui.interpolation",
                62,
                78,
                None,
                Some("todo.title")
            ),
        ]
    );
    assert_eq!(document.scopes, [(1, strings(&["todo", "i"]))]);
    assert_eq!(&TODOS[53..61], ":key=\"i\"");
}

#[test]
fn a_batch_carries_the_visited_kinds_all_parents_and_only_demanded_facts() {
    let document = PluginDocument::build(TODOS, "Todos.vue").expect("splits");
    let mut manager = FactManager::new(&REGISTRY);
    let (visit, demands) = (
        strings(&["ui.for", "ui.bind"]),
        strings(&["templateScopes"]),
    );
    let batch =
        build_batch(&document, &spec(Some(&visit), &demands), &mut manager).expect("builds");
    assert_eq!(batch.nodes, 2);
    assert_eq!(
        batch.json,
        concat!(
            r#"{"schema":1,"plugin":"team","file":"Todos.vue","parents":[-1,0,1,2,2],"nodes":["#,
            r#"{"id":1,"parent":0,"kind":"ui.for","value":"todos","alias":{"value":"todo","key":"i","index":null}},"#,
            r#"{"id":3,"parent":2,"kind":"ui.bind","name":"key","value":"i"}],"#,
            r#""facts":{"templateScopes":[[1,[{"name":"todo","position":"value"},{"name":"i","position":"key"}]]]}}"#,
        )
    );
    let bare = build_batch(&document, &spec(Some(&visit), &[]), &mut manager).expect("builds");
    let (_, facts) = bare.json.split_once(r#""facts":"#).expect("facts member");
    assert_eq!(facts, "{}}");
}

#[test]
fn a_demand_is_computed_once_per_document_however_many_plugins_declare_it() {
    let document = PluginDocument::build(TODOS, "Todos.vue").expect("splits");
    let demands = strings(&["templateScopes"]);
    let demand = resolve_demands("team", &demands).expect("known");
    let mut manager = FactManager::new(&REGISTRY);
    for _ in 0..3 {
        build_batch(&document, &spec(None, &demands), &mut manager).expect("builds");
    }
    // Already computed: a further demand produces nothing.
    assert_eq!(manager.computed(), demand);
    assert_eq!(manager.compute(&document, demand), Ok(Demand::NONE));
}

#[test]
fn destructured_and_slot_scopes_bind_exactly_their_names() {
    let source = "<template><Card v-for=\"({ id }, key, n) in rows\" v-slot=\"props\">{{ id }}</Card></template>";
    let document = PluginDocument::build(source, "Card.vue").expect("splits");
    let view_manager = &mut FactManager::new(&REGISTRY);
    let table = TemplateScopes::produce(
        &document,
        &view_manager.view::<super::facts::JsPluginHost>(),
    );
    let rows: Vec<_> = table
        .iter()
        .map(|(id, entries)| {
            (
                *id,
                entries
                    .iter()
                    .map(|e| (e.name.as_str(), e.position))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let slot = document
        .nodes
        .iter()
        .find(|n| n.kind == "ui.slot-content")
        .expect("slot")
        .id;
    assert_eq!(
        rows,
        [
            (0, vec![("key", "key"), ("n", "index")]),
            (slot, vec![("props", "slot")])
        ]
    );
}

#[test]
fn a_manifest_naming_an_unknown_group_or_kind_is_refused_exactly() {
    let document = PluginDocument::build(TODOS, "Todos.vue").expect("splits");
    let mut manager = FactManager::new(&REGISTRY);
    let bindings = strings(&["bindings"]);
    let error =
        build_batch(&document, &spec(None, &bindings), &mut manager).expect_err("unknown fact");
    assert_eq!(
        error.to_string(),
        "team: fact group `bindings` is not available to JS plugins (available: templateScopes)"
    );
    let visit = strings(&["ui.div"]);
    let error =
        build_batch(&document, &spec(Some(&visit), &[]), &mut manager).expect_err("unknown kind");
    assert_eq!(
        error,
        HostError::UnknownKind {
            plugin: "team".to_owned(),
            kind: "ui.div".to_owned()
        }
    );
}

#[test]
fn the_content_key_covers_the_plugin_code_version_and_file() {
    let demands = strings(&["templateScopes"]);
    let base = spec(None, &demands);
    let key = content_key(TODOS, "Todos.vue", &base);
    assert_eq!(key.len(), 38);
    assert_eq!(&key[..6], "s0.v1:");
    assert_eq!(content_key(TODOS, "Todos.vue", &base), key);
    let variants = [
        content_key(TODOS, "Other.vue", &base),
        content_key(&TODOS.replace("todos", "items"), "Todos.vue", &base),
        content_key(
            TODOS,
            "Todos.vue",
            &PluginSpec {
                version: "1.0.1",
                ..base
            },
        ),
        content_key(
            TODOS,
            "Todos.vue",
            &PluginSpec {
                fingerprint: "g",
                ..base
            },
        ),
        content_key(TODOS, "Todos.vue", &spec(None, &[])),
        content_key(
            TODOS,
            "Todos.vue",
            &PluginSpec {
                name: "other",
                ..base
            },
        ),
        content_key(
            TODOS,
            "Todos.vue",
            &PluginSpec {
                visit: Some(&strings(&["ui.for"])),
                ..base
            },
        ),
    ];
    assert_eq!(
        variants.iter().filter(|variant| **variant == key).count(),
        0
    );
}

#[test]
fn cached_diagnostics_survive_a_new_cache_instance_and_reject_bad_disk_entries() {
    let dir = tempfile::tempdir().expect("cache dir");
    let key = content_key(TODOS, "Todos.vue", &spec(None, &[]));
    let document = PluginDocument::build(TODOS, "Todos.vue").expect("document");
    let expected = diagnostics(
        &document,
        "team",
        r#"[{"rule":"index-key","node":3,"message":"use an id"}]"#,
    )
    .expect("diagnostic");
    let mut first = PluginCache::default();
    assert_eq!(first.get(&key, Some(dir.path())), None);
    first.put(&key, expected.clone(), Some(dir.path()));
    let mut second = PluginCache::default();
    assert_eq!(second.get(&key, Some(dir.path())), Some(expected.clone()));

    let entry = std::fs::read_dir(dir.path())
        .expect("cache dir")
        .next()
        .expect("cache entry")
        .expect("cache path")
        .path();
    std::fs::write(&entry, b"{\"schema\":999}").expect("corrupt entry");
    assert_eq!(PluginCache::default().get(&key, Some(dir.path())), None);
    std::fs::write(&entry, b"unfinished").expect("tear entry");
    assert_eq!(PluginCache::default().get(&key, Some(dir.path())), None);
}

#[test]
fn plugin_inputs_and_same_version_builds_invalidate_the_key_independently() {
    let base = spec(None, &[]);
    let one = [PluginCacheInput {
        name: "threshold",
        value: "one",
    }];
    let two = [PluginCacheInput {
        name: "threshold",
        value: "two",
    }];
    let first = content_key_for_build(TODOS, "Todos.vue", &base, &one, "revision-a").unwrap();
    assert_eq!(
        first,
        content_key_for_build(TODOS, "Todos.vue", &base, &one, "revision-a").unwrap()
    );
    assert_ne!(
        first,
        content_key_for_build(TODOS, "Todos.vue", &base, &two, "revision-a").unwrap()
    );
    assert_ne!(
        first,
        content_key_for_build(TODOS, "Todos.vue", &base, &one, "revision-b").unwrap()
    );
    let reordered = [
        PluginCacheInput {
            name: "flag",
            value: "yes",
        },
        one[0],
    ];
    let opposite = [one[0], reordered[0]];
    assert_eq!(
        content_key_for_build(TODOS, "Todos.vue", &base, &reordered, "revision-a").unwrap(),
        content_key_for_build(TODOS, "Todos.vue", &base, &opposite, "revision-a").unwrap()
    );
    assert_eq!(
        validate_cache_inputs("team", false, &[]),
        Err(HostError::InvalidCacheInputs {
            plugin: "team".to_owned(),
            detail: "declare cacheInputs, even when it is empty".to_owned(),
        })
    );
    assert_eq!(validate_cache_inputs("team", true, &one), Ok(()));
    assert_eq!(
        validate_cache_inputs("team", true, &[one[0], two[0]]),
        Err(HostError::InvalidCacheInputs {
            plugin: "team".to_owned(),
            detail: "input name `threshold` is duplicated".to_owned(),
        })
    );
}

#[test]
fn reports_anchor_on_the_host_span_in_one_order() {
    let document = PluginDocument::build(TODOS, "Todos.vue").expect("splits");
    let reports = r#"[{"rule":"b","node":4,"message":"m"},{"rule":"a","node":3,"message":"k"}]"#;
    let found = diagnostics(&document, "team", reports).expect("maps");
    let rows: Vec<_> = found
        .iter()
        .map(|d| {
            (
                d.rule_id.as_str(),
                d.start,
                d.end,
                d.line,
                d.column,
                d.end_line,
                d.end_column,
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("team/a", 53, 61, 3, 36, 3, 44),
            ("team/b", 62, 78, 3, 45, 3, 61)
        ]
    );
    let error = diagnostics(
        &document,
        "team",
        r#"[{"rule":"a","node":9,"message":"m"}]"#,
    )
    .expect_err("node");
    assert_eq!(
        error.to_string(),
        "team/a: reported node 9, which the document does not have"
    );
    let error = diagnostics(&document, "team", "{}").expect_err("shape");
    assert_eq!(
        error.to_string(),
        "team: run() must return a JSON report array (invalid type: map, expected a sequence at line 1 column 0)"
    );
}
