//! Request-local arena reuse for owned Vue contract facts.

use tower_lsp::lsp_types::{Location, Range, SymbolInformation, SymbolKind, Url};
use vize_croquis::macros::MacroTracker;

use super::WorkspaceSymbolsService;

#[cfg(test)]
mod projection_tests;

impl WorkspaceSymbolsService {
    pub(super) fn collect_vue_specific_symbols(
        uri: &Url,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
        query: &str,
        symbols: &mut Vec<SymbolInformation>,
        allocator: Option<&mut vize_l0::Allocator>,
    ) {
        let Some(ref script_setup) = descriptor.script_setup else {
            return;
        };
        match allocator {
            Some(allocator) => {
                let macros = vize_croquis::script_parser::parse_workspace_symbol_contracts_in(
                    allocator,
                    &script_setup.content,
                );
                // The result owns its facts; every parser/AST borrow has ended.
                allocator.reset();
                Self::append_contract_symbols(uri, descriptor, query, symbols, &macros);
            }
            None => {
                let mut analyzer =
                    vize_croquis::Drawer::with_options(vize_croquis::DrawerOptions {
                        analyze_script: true,
                        ..Default::default()
                    });
                analyzer.analyze_script_setup(&script_setup.content);
                let croquis = analyzer.finish();
                // Preserve the original full Croquis through presentation.
                Self::append_contract_symbols(uri, descriptor, query, symbols, &croquis.macros);
            }
        }
    }

    #[expect(
        deprecated,
        reason = "SymbolInformation.deprecated is part of the complete packet"
    )]
    fn append_contract_symbols(
        uri: &Url,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
        query: &str,
        symbols: &mut Vec<SymbolInformation>,
        macros: &MacroTracker,
    ) {
        let Some(ref script_setup) = descriptor.script_setup else {
            return;
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
pub(super) fn assert_contract_packet_parity() {
    projection_tests::assert_projection_packet_parity();
    let scripts = [
        "const ordinary = 1; function reset() {}",
        "const model = defineModel<string>('title');",
        "defineOptions({ name: 'Card', emits: ['options-event'] });",
        "export default { emits: ['plain-event'] };",
        "class Card { @Emit('decorated-event') save() {} }",
        "const events = factory; const emit = events(['alias-event']);",
        "import { defineEmits as events } from 'vue'; const emit = events(['alias-event']);",
        "const emit = defineEmits(['save-item', 'update:modelValue']);",
        "const emit = defineEmits({ save: (value: number) => true, cancel: null });",
        "const events = ['save']; const emit = defineEmits([...events, 'cancel']);",
        "const events = { save: (value: number) => true }; const copied = { ...events }; const emit = defineEmits(copied);",
        "const events = { save: (value: number) => true }; const copied = { ...events }; const emit = defineEmits({ ...copied, cancel: null });",
        "const emit = defineEmits<{ save(id: number): void; 'close-item'(): void }>();",
        "defineSlots<({ header(): any } & { 'item-row': (props: { item: number }) => any; header(): any })>();",
        "defineSlots<{ header(): any }>(); const slots = useSlots(); slots?.",
        "const slots = useSlots(); slots?.",
        "const emit = defineEmits<{ (event: 'save-item', id: number): void; (event: 'close'): void }>();",
        "const emit = defineEmits<{ save: [id: number]; close: [] }>();",
        "defineSlots<{ header(): any; default(props: { item: string }): any }>();",
        r"const emit = define\u0045mits(['save-item']);",
        r"const emit = \u0064efineEmits(['save-item']);",
        r"defineS\u006cots<{ header(): any }>();",
        "const emit = defineEmits(['雪🌸']); const Klass = 1;",
        "const Klass = 1; const 雪 = 2;",
        r"const newline = '\n';",
        "// defineSlots\nconst ordinary = 1;",
        "const text = 'defineEmits'; const ordinary = 1;",
        "const defineEmitsLater = () => 1;",
        "const defineemits = () => 1;",
        "const broken: = 1; const emit = defineEmits(['save-item']);",
        "const broken = ;",
        "const emit = define/**/Emits(['save-item']);",
        "const emit = withDefaults(defineEmits<{ save: [] }>(), {});",
        "const make = () => defineEmits(['nested-event']);",
    ];
    let uri = Url::parse("file:///workspace/user-card.vue").unwrap();
    let queries = [
        "", "save", "update:", "header", "usercard", ".panel", "雪", "missing",
    ];
    let prefixes = ["", "<!-- 雪🌸 -->\n\n", "<!-- edited offset -->\r\n"];

    let mut allocator = vize_l0::Allocator::default();
    assert_eq!(allocator.used_bytes(), 0);
    let mut packet_pairs = 0;
    for script in scripts {
        let label = script;
        for prefix in prefixes {
            let source = format!(
                "{prefix}<script lang=\"ts\">const plainBinding = 1;</script>\n\
                 <script setup lang=\"ts\">\n{script}\n</script>\n\
                 <template><slot name=\"template-only\" /></template>\n\
                 <style>.panel {{ color: #fff; }}</style>\n"
            );
            for query in queries {
                assert_packet_matches_drawer(&uri, &source, query, label, &mut allocator);
                packet_pairs += 1;
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
            assert_packet_matches_drawer(&uri, source, query, "SFC boundary", &mut allocator);
            packet_pairs += 1;
        }
    }
    assert_eq!(packet_pairs, 848);

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
        assert_packet_matches_drawer(&uri, &source, "", "producer anchor", &mut allocator);
        WorkspaceSymbolsService::collect_symbols_from_document_with_contract_allocator(
            &uri,
            &source,
            "",
            &mut symbols,
            Some(&mut allocator),
        );
        assert_eq!(allocator.used_bytes(), 0);
        assert_eq!(
            symbols
                .iter()
                .filter(|symbol| symbol.name == name && symbol.kind == kind)
                .count(),
            1,
            "{script}"
        );
    }
    assert_reuse_sequence(&uri);
}

#[cfg(test)]
fn assert_packet_matches_drawer(
    uri: &Url,
    source: &str,
    query: &str,
    label: &str,
    allocator: &mut vize_l0::Allocator,
) {
    let query = query.to_lowercase();
    let mut original = Vec::new();
    WorkspaceSymbolsService::collect_symbols_from_document_with_contract_allocator(
        uri,
        source,
        &query,
        &mut original,
        None,
    );
    let mut actual = Vec::new();
    WorkspaceSymbolsService::collect_symbols_from_document_with_contract_allocator(
        uri,
        source,
        &query,
        &mut actual,
        Some(&mut *allocator),
    );
    assert_eq!(allocator.used_bytes(), 0);
    assert_eq!(actual, original, "{label}: {query}");
    assert_eq!(
        super::rank::sort(actual, &query),
        super::rank::sort(original, &query),
        "ranked {label}: {query}"
    );
}

#[cfg(test)]
struct Retained {
    source: String,
    macros: MacroTracker,
    facts: String,
    contracts: Vec<SymbolInformation>,
    symbols: Vec<SymbolInformation>,
    original: Vec<SymbolInformation>,
}

#[cfg(test)]
fn assert_reuse_sequence(uri: &Url) {
    let bindings = (0..1024)
        .map(|index| format!("const value{index}: number = {index};\n"))
        .collect::<String>();
    let events = (0..128)
        .map(|index| format!("'event-{index}'"))
        .collect::<Vec<_>>()
        .join(",");
    let large = format!(
        "{bindings}defineEmits([{events}]); defineSlots<{{ header(): any; '雪🌸'(): any }}>(); const model = defineModel<string>('title'); defineOptions({{ name: 'ArenaCard' }}); const props = withDefaults(defineProps<{{ value?: string }}>(), {{ value: '雪🌸' }}); defineExpose({{ model }}); await Promise.resolve(1);"
    );
    let mut allocator = vize_l0::Allocator::default();
    let mut retained = Vec::new();
    for (index, script) in [
        large.as_str(),
        "defineEmits(['tiny']);",
        "const broken = ;",
        "",
        r"define\u0045mits(['雪🌸', '\u0061']); defineS\u006cots<{ '雪🌸'(): any }>();",
        large.as_str(),
        large.as_str(),
    ]
    .into_iter()
    .enumerate()
    {
        let source = format!(
            "<!-- 雪🌸 -->\r\n<script setup lang='ts'>\n{script}\n</script><style>.panel {{}}</style>"
        );
        let mut original = Vec::new();
        WorkspaceSymbolsService::collect_symbols_from_document_with_contract_allocator(
            uri,
            &source,
            "",
            &mut original,
            None,
        );
        let mut symbols = Vec::new();
        WorkspaceSymbolsService::collect_symbols_from_document_with_contract_allocator(
            uri,
            &source,
            "",
            &mut symbols,
            Some(&mut allocator),
        );
        assert_eq!(allocator.used_bytes(), 0);
        assert_eq!(symbols, original);
        let descriptor = super::descriptor::parse(uri.path(), &source).unwrap();
        let mut contracts = Vec::new();
        WorkspaceSymbolsService::collect_vue_specific_symbols(
            uri,
            &descriptor,
            "",
            &mut contracts,
            None,
        );
        let setup = descriptor.script_setup.as_ref().unwrap().content.as_ref();
        let mut drawer = vize_croquis::Drawer::with_options(vize_croquis::DrawerOptions {
            analyze_script: true,
            ..Default::default()
        });
        drawer.analyze_script_setup(setup);
        let facts = macro_fingerprint(&drawer.finish().macros);
        let macros = vize_croquis::script_parser::parse_workspace_contracts_in(&allocator, setup);
        drop(descriptor);
        assert_eq!(macro_fingerprint(&macros), facts);
        if index == 0 {
            assert!(allocator.used_bytes() > 0);
        }
        allocator.reset();
        assert_eq!(allocator.used_bytes(), 0);
        // reset clears usage; reserved arena capacity may remain nonzero.
        retained.push(Retained {
            source,
            macros,
            facts,
            contracts,
            symbols,
            original,
        });
        assert_retained(uri, &retained);
    }
    drop(allocator);
    drop(large);
    assert_retained(uri, &retained);
    // A new request owns a fresh empty handle, not the preceding request's arena.
    // This is an allocator API assertion, not a process-RSS reclamation claim.
    let next_request = vize_l0::Allocator::default();
    assert_eq!(next_request.used_bytes(), 0);
    assert_eq!(next_request.capacity(), 0);
}

#[cfg(test)]
fn assert_retained(uri: &Url, retained: &[Retained]) {
    for saved in retained {
        assert_eq!(macro_fingerprint(&saved.macros), saved.facts);
        let descriptor = super::descriptor::parse(uri.path(), &saved.source).unwrap();
        let mut contracts = Vec::new();
        WorkspaceSymbolsService::append_contract_symbols(
            uri,
            &descriptor,
            "",
            &mut contracts,
            &saved.macros,
        );
        assert_eq!(contracts, saved.contracts);
        assert_eq!(saved.symbols, saved.original);
        assert_eq!(
            super::rank::sort(saved.symbols.clone(), ""),
            super::rank::sort(saved.original.clone(), "")
        );
    }
}

#[cfg(test)]
fn macro_fingerprint(macros: &MacroTracker) -> String {
    let props = macros
        .props()
        .iter()
        .map(|prop| {
            (
                prop.name.as_str(),
                macros.prop_declaration(&prop.name),
                macros.with_defaults_key_ranges(&prop.name),
            )
        })
        .collect::<Vec<_>>();
    let emits = macros
        .emits()
        .iter()
        .map(|emit| {
            (
                emit.name.as_str(),
                macros.emit_declaration(&emit.name),
                macros.emit_validator_signatures(&emit.name),
                macros.emit_validator_type_annotations(&emit.name),
            )
        })
        .collect::<Vec<_>>();
    let models = macros
        .models()
        .iter()
        .map(|model| {
            (
                model.name.as_str(),
                macros.model_declaration(&model.name),
                macros.model_modifier_type(&model.name),
            )
        })
        .collect::<Vec<_>>();
    let slots = (0..macros.slots().len())
        .map(|index| macros.slot_declaration(index))
        .collect::<Vec<_>>();
    let mut defaults = macros.resolved_prop_defaults().collect::<Vec<_>>();
    defaults.sort_unstable();
    format!(
        "{macros:?}|{props:?}|{emits:?}|{models:?}|{slots:?}|{:?}",
        (
            macros.slot_type_argument_range(),
            macros.static_slot_declarations().collect::<Vec<_>>(),
            macros.define_options_name(),
            macros.expose_bindings(),
            macros.expose_is_complete(),
            macros.with_defaults_expression(),
            defaults
        )
    )
}
