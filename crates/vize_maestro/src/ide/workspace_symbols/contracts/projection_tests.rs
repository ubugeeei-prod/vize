//! Differential controls for the narrow closed-file event/slot demand.

use super::{MacroTracker, Url, WorkspaceSymbolsService};

pub(super) fn assert_projection_packet_parity() {
    let scripts = [
        "const [emit] = defineEmits(['array-event']);",
        "const [slots] = defineSlots<{ 'array-slot'(): any }>();",
        "const [emit] = (defineEmits(['array-event']) as unknown)!;",
        "const emit = (defineEmits(['wrapped-event']) satisfies unknown)!;",
        "(defineEmits(['ignored-event']));",
        "defineEmits(['ignored-event']) as unknown;",
        "const emit = <unknown>defineEmits(['ignored-event']);",
        "const [emit] = <unknown>defineEmits(['ignored-event']);",
        "const { emit } = defineEmits(['ignored-event']);",
        "const { emit } = withDefaults(defineEmits(['ignored-event']), {});",
        "const emit = withDefaults(withDefaults(defineEmits(['wrapped-event']), {}), {});",
        "withDefaults((defineEmits(['ignored-event'])), {});",
        "withDefaults(defineSlots<{ 'wrapped-slot'(): any }>(), {});",
        "const [slots] = withDefaults(defineSlots<{ 'array-slot'(): any }>(), {});",
        "function setup() { defineEmits(['ignored-event']); }",
        "{ defineSlots<{ 'ignored-slot'(): any }>(); }",
        "if (true) { const emit = defineEmits(['ignored-event']); }",
        "const emit = () => defineEmits(['ignored-event']);",
        "run(() => defineSlots<{ 'ignored-slot'(): any }>());",
        "export const emit = defineEmits(['ignored-event']);",
        "object.defineEmits(['ignored-event']);",
        "import { defineEmits as events } from 'vue'; const emit = events(['ignored-event']);",
        "type Events = { 'reference-event': [] }; defineEmits<Events>();",
        "interface Slots { 'reference-slot'(): any } defineSlots<Slots>();",
        "import type { Events } from './events'; defineEmits<Events>();",
        "import type { Slots } from './slots'; defineSlots<Slots>();",
        "defineEmits<{ first: [] } & { second: [] }>();",
        "defineSlots<({ first(): any } & ({ second(): any; first(): any }))>();",
        "let events = { before: null }; events = { after: null }; defineEmits(events);",
        "var events = { before: null }; events.before = 1; delete events.before; defineEmits(events);",
        "const events = { before: null }; mutate(events); defineEmits(events);",
        "const events = { before: null }; const copy = events; events.after = null; defineEmits(copy);",
        "const copy = events; const events = { after: null }; defineEmits(copy);",
        "const events = { before: null }; { const events = { after: null }; } defineEmits(events);",
        "const events = { before: null }; const copy = { ...events, after: null }; defineEmits(copy);",
        "const events = { before: (value: number) => true }; defineEmits({ ...events, before: null });",
        "const events = { before: null } as Record<string, unknown>; defineEmits(events);",
        "const events = { before: null } as const; defineEmits(events);",
        "const events = { before: null } satisfies Record<string, unknown>; defineEmits(events);",
        r"const [emit] = define\u0045mits(['雪🌸']); defineS\u006cots<{ '雪🌸'(): any }>();",
        "const emit = defineEmits(['before']); const value = api?.\ndefineSlots<{ after(): any }>();",
        "const emit = defineEmits(['before']); const value = api(\nconst slots = defineSlots<{ after(): any }>();",
    ];
    let uri = Url::parse("file:///workspace/array-card.vue").unwrap();
    let queries = [
        "",
        "array",
        "wrapped",
        "before",
        "after",
        "雪",
        "arraycard",
        "missing",
    ];
    let prefixes = ["", "<!-- 雪🌸 -->\n", "<!-- shifted -->\r\n"];
    let mut allocator = vize_l0::Allocator::default();
    let mut pairs = 0;
    for script in scripts {
        for prefix in prefixes {
            let source = format!(
                "{prefix}<script setup lang='ts'>\n{script}\n</script><style>.panel {{}}</style>"
            );
            for query in queries {
                super::assert_packet_matches_drawer(&uri, &source, query, script, &mut allocator);
                pairs += 1;
            }
        }
    }
    assert_eq!(pairs, 1008);
    assert_admission_matrix(&uri, &mut allocator);
    assert_registration_controls(&mut allocator);
    assert_owned_projection_survives_reuse(&uri, allocator);
}

fn assert_admission_matrix(uri: &Url, allocator: &mut vize_l0::Allocator) {
    let shapes = [
        "$CALL;",
        "const value = $CALL;",
        "let value = ($CALL as unknown)!;",
        "var [value] = $CALL;",
        "const { value } = $CALL;",
        "export const value = $CALL;",
        "export declare const value = $CALL;",
        "export default $CALL;",
        "declare const value = $CALL;",
        "const value = <unknown>$CALL;",
        "const [value] = <unknown>$CALL;",
        "($CALL);",
        "$CALL as unknown;",
        "if (true) { $CALL; }",
        "{ $CALL; }",
        "function setup() { const value = $CALL; }",
        "const factory = () => $CALL;",
        "run(() => $CALL);",
        "const value = flag ? $CALL : null;",
        "flag ? $CALL : null;",
        "const [value] = $CALL satisfies unknown;",
        "const value = $CALL, next = $CALL;",
        "const defineEmits = () => null; const value = $CALL;",
        "const value = ;\nconst next = $CALL;",
    ];
    let calls = [
        "defineEmits(['shape-event'])",
        "defineSlots<{ 'shape-slot'(): any }>()",
        "withDefaults(defineEmits(['shape-event']), {})",
    ];
    let mut pairs = 0;
    for shape in shapes {
        for call in calls {
            let script = shape.replace("$CALL", call);
            for prefix in ["", "<!-- 雪🌸 -->\r\n"] {
                let source = format!("{prefix}<script setup lang='ts'>\n{script}\n</script>");
                for query in ["", "shape", "missing"] {
                    super::assert_packet_matches_drawer(uri, &source, query, &script, allocator);
                    pairs += 1;
                }
            }
        }
    }
    assert_eq!(pairs, 432);
}

fn assert_registration_controls(allocator: &mut vize_l0::Allocator) {
    let controls: &[(&str, &[&str], &[&str])] = &[
        (
            "const [emit] = defineEmits(['array-event']);",
            &["array-event"],
            &[],
        ),
        (
            "const [slots] = defineSlots<{ 'array-slot'(): any }>();",
            &[],
            &["array-slot"],
        ),
        (
            "const emit = (defineEmits(['wrapped-event']) as unknown)!;",
            &["wrapped-event"],
            &[],
        ),
        ("(defineEmits(['ignored-event']));", &[], &[]),
        (
            "const emit = <unknown>defineEmits(['ignored-event']);",
            &[],
            &[],
        ),
        (
            "const { emit } = withDefaults(defineEmits(['ignored-event']), {});",
            &[],
            &[],
        ),
        (
            "withDefaults(defineEmits(['wrapped-event']), {});",
            &["wrapped-event"],
            &[],
        ),
        (
            "withDefaults((defineEmits(['ignored-event'])), {});",
            &[],
            &[],
        ),
        (
            "type Events = { event: [] }; defineEmits<Events>();",
            &[],
            &[],
        ),
        (
            "interface Slots { slot(): any } defineSlots<Slots>();",
            &[],
            &[],
        ),
        (
            "let events = { before: null }; events = { after: null }; defineEmits(events);",
            &["before"],
            &[],
        ),
        (
            "const events = { before: null }; mutate(events); defineEmits(events);",
            &["before"],
            &[],
        ),
        (
            "const copy = events; const events = { after: null }; defineEmits(copy);",
            &[],
            &[],
        ),
        (
            "function setup() { defineEmits(['ignored-event']); }",
            &[],
            &[],
        ),
    ];
    for (source, emits, slots) in controls {
        let actual = parse(allocator, source);
        assert_eq!(
            actual
                .emits()
                .iter()
                .map(|emit| emit.name.as_str())
                .collect::<Vec<_>>(),
            *emits,
            "emits: {source}"
        );
        assert_eq!(
            actual
                .slots()
                .iter()
                .map(|slot| slot.name.as_str())
                .collect::<Vec<_>>(),
            *slots,
            "slots: {source}"
        );
        assert_projection_matches_drawer(source, &actual);
        allocator.reset();
        assert_eq!(allocator.used_bytes(), 0);
    }
}

fn assert_owned_projection_survives_reuse(uri: &Url, mut allocator: vize_l0::Allocator) {
    let large = (0..1024)
        .map(|index| format!("const value{index}: number = {index};\n"))
        .collect::<String>()
        + "const events = { 'large-event': (value: number) => true }; const copy = { ...events }; defineEmits(copy); defineSlots<{ header(): any; '雪🌸'(): any }>();";
    let mut retained = Vec::new();
    for script in [
        large.as_str(),
        "const [emit] = defineEmits(['tiny']);",
        "const broken = ;",
        "",
        r"const [emit] = define\u0045mits(['雪🌸']); defineS\u006cots<{ '雪🌸'(): any }>();",
        large.as_str(),
        large.as_str(),
    ] {
        let macros = parse(&allocator, script);
        assert_projection_matches_drawer(script, &macros);
        let facts = projection_fingerprint(&macros);
        let source = format!("<!-- 雪🌸 -->\r\n<script setup lang='ts'>\n{script}\n</script>");
        super::assert_packet_matches_drawer(
            uri,
            &source,
            "",
            "retained projection",
            &mut allocator,
        );
        retained.push((macros, facts, script.to_owned()));
        for (saved, facts, script) in &retained {
            assert_eq!(projection_fingerprint(saved), *facts);
            assert_projection_matches_drawer(script, saved);
        }
    }
    drop(allocator);
    drop(large);
    for (saved, facts, script) in retained {
        assert_eq!(projection_fingerprint(&saved), facts);
        assert_projection_matches_drawer(&script, &saved);
    }
}

fn parse(allocator: &vize_l0::Allocator, source: &str) -> MacroTracker {
    vize_croquis::script_parser::parse_workspace_symbol_contracts_in(allocator, source)
}

fn assert_projection_matches_drawer(source: &str, actual: &MacroTracker) {
    let mut drawer = vize_croquis::Drawer::with_options(vize_croquis::DrawerOptions {
        analyze_script: true,
        ..Default::default()
    });
    drawer.analyze_script_setup(source);
    assert_eq!(
        projection_fingerprint(actual),
        projection_fingerprint(&drawer.finish().macros),
        "{source}"
    );
}

fn projection_fingerprint(macros: &MacroTracker) -> String {
    let emits = macros
        .emits()
        .iter()
        .map(|emit| {
            (
                macros.emit_declaration(&emit.name),
                macros.emit_validator_signatures(&emit.name),
                macros.emit_validator_type_annotations(&emit.name),
            )
        })
        .collect::<Vec<_>>();
    let slots = (0..macros.slots().len())
        .map(|index| macros.slot_declaration(index))
        .collect::<Vec<_>>();
    format!(
        "{:?}|{:?}|{emits:?}|{slots:?}|{:?}|{:?}",
        macros.emits(),
        macros.slots(),
        macros.slot_type_argument_range(),
        macros.static_slot_declarations().collect::<Vec<_>>()
    )
}
