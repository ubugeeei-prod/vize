#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use super::WorkspaceSymbolsService;
use tower_lsp::lsp_types::{Position, Range, SymbolKind};

#[test]
fn test_to_pascal_case() {
    assert_eq!(
        WorkspaceSymbolsService::to_pascal_case("hello-world"),
        "HelloWorld"
    );
    assert_eq!(
        WorkspaceSymbolsService::to_pascal_case("my_component"),
        "MyComponent"
    );
    assert_eq!(WorkspaceSymbolsService::to_pascal_case("Button"), "Button");
}

#[test]
fn test_extract_identifier() {
    assert_eq!(
        WorkspaceSymbolsService::extract_identifier("count = 0"),
        Some("count".to_string())
    );
    assert_eq!(
        WorkspaceSymbolsService::extract_identifier("MyClass extends Base"),
        Some("MyClass".to_string())
    );
    assert_eq!(
        WorkspaceSymbolsService::extract_identifier("{ a, b } = obj"),
        None
    );
}

#[test]
fn css_selector_names_read_classes_and_ids_from_preludes() {
    let names: Vec<_> =
        super::style::css_selector_names("#app .container .item-active { color: #fff }")
            .into_iter()
            .map(|(offset, marker, name)| (offset, marker, name.to_owned()))
            .collect();
    assert_eq!(
        names,
        vec![
            (1, '#', "app".to_owned()),
            (6, '.', "container".to_owned()),
            (17, '.', "item-active".to_owned()),
        ]
    );
}

#[test]
fn test_parse_declaration() {
    let (name, kind) = WorkspaceSymbolsService::parse_declaration("count = ref(0)").unwrap();
    assert_eq!(name, "count");
    assert_eq!(kind, SymbolKind::VARIABLE);

    let (name, kind) =
        WorkspaceSymbolsService::parse_declaration("handleClick = () => {}").unwrap();
    assert_eq!(name, "handleClick");
    assert_eq!(kind, SymbolKind::FUNCTION);
}

#[test]
fn macro_declaration_range_targets_quoted_name_content() {
    let source = "<script setup>\nconst emit = defineEmits(['save-item'])\n</script>\n";
    let script_start = source.find("const emit").unwrap();
    let script_end = source.find("\n</script>").unwrap();
    let script = &source[script_start..script_end];
    let quoted_start = script.find("'save-item'").unwrap();
    let quoted_end = quoted_start + "'save-item'".len();
    let name_start = source.find("save-item").unwrap();
    let name_end = name_start + "save-item".len();
    let fallback = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    };

    let range = WorkspaceSymbolsService::macro_declaration_range(
        source,
        script,
        script_start,
        Some((quoted_start as u32, quoted_end as u32)),
        fallback,
    );

    assert_eq!(
        range.start,
        WorkspaceSymbolsService::offset_to_position(source, name_start)
    );
    assert_eq!(
        range.end,
        WorkspaceSymbolsService::offset_to_position(source, name_end)
    );
}

/// `(name, line)` of every style symbol that `search` returns for `query`.
fn style_symbols(source: &str, query: &str) -> Vec<(String, u32)> {
    let state = crate::server::ServerState::new();
    let uri = tower_lsp::lsp_types::Url::parse("file:///workspace/Card.vue").unwrap();
    state
        .documents
        .open(uri, source.to_owned(), 1, "vue".to_owned());
    let mut symbols: Vec<_> = WorkspaceSymbolsService::search(&state, query)
        .into_iter()
        .filter(|symbol| {
            symbol
                .container_name
                .as_deref()
                .is_some_and(|container| container.starts_with("style"))
        })
        .map(|symbol| (symbol.name, symbol.location.range.start.line))
        .collect();
    symbols.sort();
    symbols
}

#[test]
fn style_symbols_come_from_selectors_not_declaration_values() {
    let source = r#"<template><div /></template>
<style scoped>
/* .commented { } #nope */
.card, #main > .card__title:hover {
  color: #fff;
  transition: opacity .5s, transform 1.5s;
  background: url(./img/bg.png) no-repeat;
  font-family: "Font.Name";
}
a[href$=".pdf"] .file-link { opacity: .8 }
@media (min-width: 600px) {
  .wide { margin: 0 .25rem }
}
.parent { padding: 0; .nested { gap: .5em } }
</style>
"#;
    assert_eq!(
        style_symbols(source, ""),
        vec![
            ("#main".to_owned(), 3),
            (".card".to_owned(), 3),
            (".card__title".to_owned(), 3),
            (".file-link".to_owned(), 9),
            (".nested".to_owned(), 13),
            (".parent".to_owned(), 13),
            (".wide".to_owned(), 11),
        ]
    );
}

#[test]
fn script_and_component_contract_symbols_point_at_their_declarations() {
    let source = r#"<script setup lang="ts">
import { ref, computed } from 'vue'
const props = defineProps<{ title: string }>()
const emit = defineEmits<{ (e: 'save-item', id: number): void; (e: 'close'): void }>()
defineSlots<{ header(): any; default(props: { item: string }): any }>()
const count = ref(0)
const doubled = computed(() => count.value * 2)
function reset() { count.value = 0 }
const onSave = (id: number) => emit('save-item', id)
let { a, b } = { a: 1, b: 2 }
class Store {}
export interface CardProps { title: string }
</script>
<template><div /></template>
"#;
    let state = crate::server::ServerState::new();
    let uri = tower_lsp::lsp_types::Url::parse("file:///workspace/user-card.vue").unwrap();
    state
        .documents
        .open(uri, source.to_owned(), 1, "vue".to_owned());
    let mut symbols: Vec<_> = WorkspaceSymbolsService::search(&state, "")
        .into_iter()
        .map(|symbol| {
            let start = symbol.location.range.start;
            (
                symbol.name,
                symbol.kind,
                symbol.container_name,
                start.line,
                start.character,
            )
        })
        .collect();
    symbols.sort_by(|left, right| left.0.cmp(&right.0));
    let card = || Some("UserCard".to_owned());
    let setup = || Some("script setup".to_owned());
    assert_eq!(
        symbols,
        vec![
            ("Store".to_owned(), SymbolKind::CLASS, setup(), 10, 0),
            ("UserCard".to_owned(), SymbolKind::CLASS, None, 0, 0),
            ("close".to_owned(), SymbolKind::EVENT, card(), 3, 68),
            ("count".to_owned(), SymbolKind::VARIABLE, setup(), 5, 0),
            // Slot keys, not the start of `<script setup>`.
            ("default".to_owned(), SymbolKind::INTERFACE, card(), 4, 29),
            ("doubled".to_owned(), SymbolKind::VARIABLE, setup(), 6, 0),
            ("emit".to_owned(), SymbolKind::CONSTANT, setup(), 3, 0),
            ("header".to_owned(), SymbolKind::INTERFACE, card(), 4, 14),
            ("onSave".to_owned(), SymbolKind::FUNCTION, setup(), 8, 0),
            ("props".to_owned(), SymbolKind::CONSTANT, setup(), 2, 0),
            ("reset".to_owned(), SymbolKind::FUNCTION, setup(), 7, 0),
            ("save-item".to_owned(), SymbolKind::EVENT, card(), 3, 32),
        ]
    );
}
