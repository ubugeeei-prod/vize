//! Avoid full setup analysis only when none of its contract producers can occur.

use tower_lsp::lsp_types::{Location, Range, SymbolInformation, SymbolKind, Url};

use super::WorkspaceSymbolsService;

/// Setup contract extraction recognizes decoded direct `defineEmits` and
/// `defineSlots` identifiers. Backslashes may encode either identifier, so any
/// escape keeps the original analysis. Comments, strings and longer names are
/// deliberately conservative false positives; no source is lexically stripped.
fn may_have_contract_symbols(source: &str) -> bool {
    source.as_bytes().contains(&b'\\')
        || source.contains(vize_croquis::macros::DEFINE_EMITS)
        || source.contains(vize_croquis::macros::DEFINE_SLOTS)
}

impl WorkspaceSymbolsService {
    /// Surface Vue-specific entities (emits, slots) discovered by Croquis.
    #[expect(
        deprecated,
        reason = "SymbolInformation.deprecated is deprecated in favor of tags"
    )]
    pub(super) fn collect_vue_specific_symbols<const FILTER_CONTRACTS: bool>(
        uri: &Url,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
        query: &str,
        symbols: &mut Vec<SymbolInformation>,
    ) {
        let Some(ref script_setup) = descriptor.script_setup else {
            return;
        };

        if FILTER_CONTRACTS && !may_have_contract_symbols(&script_setup.content) {
            return;
        }

        let mut analyzer = vize_croquis::Drawer::with_options(vize_croquis::DrawerOptions {
            analyze_script: true,
            ..Default::default()
        });
        analyzer.analyze_script_setup(&script_setup.content);
        let croquis = analyzer.finish();

        let container = Self::extract_component_name(uri);

        // Fallback for Croquis symbols that do not yet carry a declaration
        // span, on top of the line-0-character-0 placeholder from #715.
        let script_setup_position =
            Self::offset_to_position(descriptor.source.as_ref(), script_setup.loc.start);
        let placeholder_range = Range {
            start: script_setup_position,
            end: script_setup_position,
        };

        // Emits declared via defineEmits<{...}>(). The macro tracker keeps the
        // raw names; expose each as an EVENT symbol so `@symbol` searches
        // discover "update:modelValue", "submit", etc.
        for emit in croquis.macros.emits() {
            let name = emit.name.as_str();
            if !query.is_empty() && !name.to_lowercase().contains(query) {
                continue;
            }
            let range = Self::macro_declaration_range(
                descriptor.source.as_ref(),
                script_setup.content.as_ref(),
                script_setup.loc.start,
                croquis.macros.emit_declaration(name),
                placeholder_range,
            );
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: uri.clone(),
                    range,
                },
                container_name: container.clone(),
            });
        }

        // Slots declared via defineSlots<{...}>(). Expose each as an
        // INTERFACE symbol — slots define the parent-child contract, much
        // like a TypeScript interface.
        for (index, slot) in croquis.macros.slots().iter().enumerate() {
            let name = slot.name.as_str();
            if !query.is_empty() && !name.to_lowercase().contains(query) {
                continue;
            }
            let range = Self::macro_declaration_range(
                descriptor.source.as_ref(),
                script_setup.content.as_ref(),
                script_setup.loc.start,
                croquis.macros.slot_declaration(index),
                placeholder_range,
            );
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::INTERFACE,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: uri.clone(),
                    range,
                },
                container_name: container.clone(),
            });
        }
    }
}

/// Run from the existing contract declaration test so the normal native test
/// inventory executes the same corpus without adding a second discovery path.
#[cfg(test)]
pub(super) fn assert_whole_packet_parity() {
    let scripts = [
        ("bindings", "const ordinary = 1; function reset() {}", false),
        (
            "model only",
            "const model = defineModel<string>('title');",
            false,
        ),
        (
            "defineOptions",
            "defineOptions({ name: 'Card', emits: ['options-event'] });",
            false,
        ),
        (
            "plain options in setup",
            "export default { emits: ['plain-event'] };",
            false,
        ),
        (
            "class decorator in setup",
            "class Card { @Emit('decorated-event') save() {} }",
            false,
        ),
        (
            "unrecognized alias",
            "const events = factory; const emit = events(['alias-event']);",
            false,
        ),
        (
            "import alias",
            "import { defineEmits as events } from 'vue'; const emit = events(['alias-event']);",
            true,
        ),
        (
            "runtime array",
            "const emit = defineEmits(['save-item', 'update:modelValue']);",
            true,
        ),
        (
            "runtime object",
            "const emit = defineEmits({ save: (value: number) => true, cancel: null });",
            true,
        ),
        (
            "runtime alias and spread",
            "const events = ['save']; const emit = defineEmits([...events, 'cancel']);",
            true,
        ),
        (
            "typed calls",
            "const emit = defineEmits<{ (event: 'save-item', id: number): void; (event: 'close'): void }>();",
            true,
        ),
        (
            "typed tuples",
            "const emit = defineEmits<{ save: [id: number]; close: [] }>();",
            true,
        ),
        (
            "slots",
            "defineSlots<{ header(): any; default(props: { item: string }): any }>();",
            true,
        ),
        (
            "escaped emits identifier",
            r"const emit = define\u0045mits(['save-item']);",
            true,
        ),
        (
            "escaped first identifier character",
            r"const emit = \u0064efineEmits(['save-item']);",
            true,
        ),
        (
            "escaped slots identifier",
            r"defineS\u006cots<{ header(): any }>();",
            true,
        ),
        (
            "unicode event and binding",
            "const emit = defineEmits(['雪🌸']); const Klass = 1;",
            true,
        ),
        (
            "unicode binding only",
            "const Klass = 1; const 雪 = 2;",
            false,
        ),
        ("unrelated escape", r"const newline = '\n';", true),
        (
            "comment false positive",
            "// defineSlots\nconst ordinary = 1;",
            true,
        ),
        (
            "string false positive",
            "const text = 'defineEmits'; const ordinary = 1;",
            true,
        ),
        (
            "long identifier false positive",
            "const defineEmitsLater = () => 1;",
            true,
        ),
        (
            "case sensitive identifier",
            "const defineemits = () => 1;",
            false,
        ),
        (
            "invalid source with producer",
            "const broken: = 1; const emit = defineEmits(['save-item']);",
            true,
        ),
        ("invalid source without producer", "const broken = ;", false),
        (
            "split identifier",
            "const emit = define/**/Emits(['save-item']);",
            false,
        ),
        (
            "wrapped producer",
            "const emit = withDefaults(defineEmits<{ save: [] }>(), {});",
            true,
        ),
        (
            "nested producer",
            "const make = () => defineEmits(['nested-event']);",
            true,
        ),
    ];
    let uri = Url::parse("file:///workspace/user-card.vue").unwrap();
    let queries = [
        "", "save", "update:", "header", "usercard", ".panel", "雪", "missing",
    ];
    let prefixes = ["", "<!-- 雪🌸 -->\n\n", "<!-- edited offset -->\r\n"];

    for (label, script, expected_analysis) in scripts {
        assert_eq!(
            may_have_contract_symbols(script),
            expected_analysis,
            "{label}"
        );
        for prefix in prefixes {
            let source = format!(
                "{prefix}<script lang=\"ts\">const plainBinding = 1;</script>\n\
                 <script setup lang=\"ts\">\n{script}\n</script>\n\
                 <template><slot name=\"template-only\" /></template>\n\
                 <style>.panel {{ color: #fff; }}</style>\n"
            );
            for query in queries {
                assert_packet_matches_unguarded(&uri, &source, query, label);
            }
        }
    }

    for source in [
        "<template><slot name='template-only' /></template><style>.panel {}</style>",
        "<script>export default { emits: ['plain-only'] }</script>",
        "<script setup lang='ts'>const ordinary = 1;</script>",
        "<script setup>const unterminated = ",
    ] {
        for query in queries {
            assert_packet_matches_unguarded(&uri, source, query, "SFC boundary");
        }
    }

    // These are real producers, not only conservative classifier matches.
    for (script, name, kind) in [
        (
            "defineEmits(['save-item']);",
            "save-item",
            SymbolKind::EVENT,
        ),
        (
            r"define\u0045mits(['save-item']);",
            "save-item",
            SymbolKind::EVENT,
        ),
        (
            r"\u0064efineEmits(['save-item']);",
            "save-item",
            SymbolKind::EVENT,
        ),
        (
            "defineSlots<{ header(): any }>();",
            "header",
            SymbolKind::INTERFACE,
        ),
        (
            r"defineS\u006cots<{ header(): any }>();",
            "header",
            SymbolKind::INTERFACE,
        ),
    ] {
        let source = format!("<script setup lang='ts'>\n{script}\n</script>");
        let mut symbols = Vec::new();
        WorkspaceSymbolsService::collect_symbols_from_document(&uri, &source, "", &mut symbols);
        assert!(
            symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.kind == kind),
            "{script}"
        );
    }
}

#[cfg(test)]
fn assert_packet_matches_unguarded(uri: &Url, source: &str, query: &str, label: &str) {
    let query = query.to_lowercase();
    let mut actual = Vec::new();
    WorkspaceSymbolsService::collect_symbols_from_document(uri, source, &query, &mut actual);
    let mut unguarded = Vec::new();
    WorkspaceSymbolsService::collect_symbols_from_document_with_contract_filter::<false>(
        uri,
        source,
        &query,
        &mut unguarded,
    );
    // Compare every field, including order, URI, kind, container and full range,
    // before ranking/truncation can hide an omitted or additional contract.
    assert_eq!(actual, unguarded, "{label}: {query}");
    assert_eq!(
        super::rank::sort(actual, &query),
        super::rank::sort(unguarded, &query),
        "ranked {label}: {query}",
    );
}
