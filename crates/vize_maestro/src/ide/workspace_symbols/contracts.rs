//! Consume owned setup macro facts without retaining a full Croquis.

use tower_lsp::lsp_types::{Location, Range, SymbolInformation, SymbolKind, Url};

use super::WorkspaceSymbolsService;

impl WorkspaceSymbolsService {
    /// Surface Vue-specific entities (emits, slots) discovered by Croquis.
    #[expect(
        deprecated,
        reason = "SymbolInformation.deprecated is deprecated in favor of tags"
    )]
    pub(super) fn collect_vue_specific_symbols<const DIRECT_MACROS: bool>(
        uri: &Url,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
        query: &str,
        symbols: &mut Vec<SymbolInformation>,
    ) {
        let Some(ref script_setup) = descriptor.script_setup else {
            return;
        };

        let macros = if DIRECT_MACROS {
            vize_croquis::script_parser::parse_script_setup_macros(&script_setup.content)
        } else {
            // Preserve the original Drawer as the complete packet oracle.
            let mut analyzer = vize_croquis::Drawer::with_options(vize_croquis::DrawerOptions {
                analyze_script: true,
                ..Default::default()
            });
            analyzer.analyze_script_setup(&script_setup.content);
            analyzer.finish().macros
        };

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
        for emit in macros.emits() {
            let name = emit.name.as_str();
            if !query.is_empty() && !name.to_lowercase().contains(query) {
                continue;
            }
            let range = Self::macro_declaration_range(
                descriptor.source.as_ref(),
                script_setup.content.as_ref(),
                script_setup.loc.start,
                macros.emit_declaration(name),
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
        for (index, slot) in macros.slots().iter().enumerate() {
            let name = slot.name.as_str();
            if !query.is_empty() && !name.to_lowercase().contains(query) {
                continue;
            }
            let range = Self::macro_declaration_range(
                descriptor.source.as_ref(),
                script_setup.content.as_ref(),
                script_setup.loc.start,
                macros.slot_declaration(index),
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
        ("bindings", "const ordinary = 1; function reset() {}"),
        ("model only", "const model = defineModel<string>('title');"),
        (
            "defineOptions",
            "defineOptions({ name: 'Card', emits: ['options-event'] });",
        ),
        (
            "plain options in setup",
            "export default { emits: ['plain-event'] };",
        ),
        (
            "class decorator in setup",
            "class Card { @Emit('decorated-event') save() {} }",
        ),
        (
            "unrecognized alias",
            "const events = factory; const emit = events(['alias-event']);",
        ),
        (
            "import alias",
            "import { defineEmits as events } from 'vue'; const emit = events(['alias-event']);",
        ),
        (
            "runtime array",
            "const emit = defineEmits(['save-item', 'update:modelValue']);",
        ),
        (
            "runtime object",
            "const emit = defineEmits({ save: (value: number) => true, cancel: null });",
        ),
        (
            "runtime alias and spread",
            "const events = ['save']; const emit = defineEmits([...events, 'cancel']);",
        ),
        (
            "runtime object alias",
            "const events = { save: (value: number) => true }; const copied = { ...events }; const emit = defineEmits(copied);",
        ),
        (
            "runtime object spread",
            "const events = { save: (value: number) => true }; const copied = { ...events }; const emit = defineEmits({ ...copied, cancel: null });",
        ),
        (
            "typed method",
            "const emit = defineEmits<{ save(id: number): void; 'close-item'(): void }>();",
        ),
        (
            "slot intersection and property",
            "defineSlots<({ header(): any } & { 'item-row': (props: { item: number }) => any; header(): any })>();",
        ),
        (
            "incomplete member with producer",
            "defineSlots<{ header(): any }>(); const slots = useSlots(); slots?.",
        ),
        (
            "incomplete member without producer",
            "const slots = useSlots(); slots?.",
        ),
        (
            "typed calls",
            "const emit = defineEmits<{ (event: 'save-item', id: number): void; (event: 'close'): void }>();",
        ),
        (
            "typed tuples",
            "const emit = defineEmits<{ save: [id: number]; close: [] }>();",
        ),
        (
            "slots",
            "defineSlots<{ header(): any; default(props: { item: string }): any }>();",
        ),
        (
            "escaped emits identifier",
            r"const emit = define\u0045mits(['save-item']);",
        ),
        (
            "escaped first identifier character",
            r"const emit = \u0064efineEmits(['save-item']);",
        ),
        (
            "escaped slots identifier",
            r"defineS\u006cots<{ header(): any }>();",
        ),
        (
            "unicode event and binding",
            "const emit = defineEmits(['雪🌸']); const Klass = 1;",
        ),
        ("unicode binding only", "const Klass = 1; const 雪 = 2;"),
        ("unrelated escape", r"const newline = '\n';"),
        (
            "macro name in comment",
            "// defineSlots\nconst ordinary = 1;",
        ),
        (
            "macro name in string",
            "const text = 'defineEmits'; const ordinary = 1;",
        ),
        ("long identifier", "const defineEmitsLater = () => 1;"),
        ("case sensitive identifier", "const defineemits = () => 1;"),
        (
            "invalid source with producer",
            "const broken: = 1; const emit = defineEmits(['save-item']);",
        ),
        ("invalid source without producer", "const broken = ;"),
        (
            "split identifier",
            "const emit = define/**/Emits(['save-item']);",
        ),
        (
            "wrapped producer",
            "const emit = withDefaults(defineEmits<{ save: [] }>(), {});",
        ),
        (
            "nested producer",
            "const make = () => defineEmits(['nested-event']);",
        ),
    ];
    let uri = Url::parse("file:///workspace/user-card.vue").unwrap();
    let queries = [
        "", "save", "update:", "header", "usercard", ".panel", "雪", "missing",
    ];
    let prefixes = ["", "<!-- 雪🌸 -->\n\n", "<!-- edited offset -->\r\n"];

    for (label, script) in scripts {
        for prefix in prefixes {
            let source = format!(
                "{prefix}<script lang=\"ts\">const plainBinding = 1;</script>\n\
                 <script setup lang=\"ts\">\n{script}\n</script>\n\
                 <template><slot name=\"template-only\" /></template>\n\
                 <style>.panel {{ color: #fff; }}</style>\n"
            );
            for query in queries {
                assert_packet_matches_drawer(&uri, &source, query, label);
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
            assert_packet_matches_drawer(&uri, source, query, "SFC boundary");
        }
    }

    // Require actual produced symbols, beyond equality of two empty packets.
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
            "const events = { save: (value: number) => true }; const copied = { ...events }; defineEmits(copied);",
            "save",
            SymbolKind::EVENT,
        ),
        (
            "const events = { save: (value: number) => true }; const copied = { ...events }; defineEmits({ ...copied, cancel: null });",
            "save",
            SymbolKind::EVENT,
        ),
        (
            "defineEmits<{ save(id: number): void }>();",
            "save",
            SymbolKind::EVENT,
        ),
        (
            "defineSlots<({ header(): any } & { 'item-row': () => any })>();",
            "item-row",
            SymbolKind::INTERFACE,
        ),
        (
            "defineSlots<{ header(): any }>(); const slots = useSlots(); slots?.",
            "header",
            SymbolKind::INTERFACE,
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
fn assert_packet_matches_drawer(uri: &Url, source: &str, query: &str, label: &str) {
    let query = query.to_lowercase();
    let mut actual = Vec::new();
    WorkspaceSymbolsService::collect_symbols_from_document(uri, source, &query, &mut actual);
    let mut original = Vec::new();
    WorkspaceSymbolsService::collect_symbols_from_document_with_macro_source::<false>(
        uri,
        source,
        &query,
        &mut original,
    );
    // Compare every field, including order, URI, kind, container and full range,
    // before ranking/truncation can hide an omitted or additional contract.
    assert_eq!(actual, original, "{label}: {query}");
    assert_eq!(
        super::rank::sort(actual, &query),
        super::rank::sort(original, &query),
        "ranked {label}: {query}",
    );
}
