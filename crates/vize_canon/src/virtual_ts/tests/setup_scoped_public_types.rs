//! Public `Emits`/`Exposed` aliases live at module scope. Macro type
//! arguments naming setup values or non-hoisted aliases must resolve through
//! the setup return instead (misskey `WidgetComponentEmits<WidgetProps>`).

use super::generate_virtual_ts_with_offsets;
use vize_croquis::{Analyzer, AnalyzerOptions};

fn generate(script: &str, generic: Option<&str>) -> vize_carton::String {
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    match generic {
        Some(generic) => analyzer.analyze_script_setup_with_generic(script, Some(generic)),
        None => analyzer.analyze_script_setup(script),
    };
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets(&summary, Some(script), None, 0, 0, &Default::default()).code
}

fn module_tail(code: &str) -> &str {
    code.split_once("// Invoke setup to verify types")
        .expect("module tail present")
        .1
}

const WIDGET: &str = r#"import type { WidgetEmits, WidgetExpose } from './types';
const def = { size: 1 };
type WidgetProps = typeof def;
const emit = defineEmits<WidgetEmits<WidgetProps>>();
defineExpose<WidgetExpose<WidgetProps>>({ current: () => def });
emit('update', def);
"#;

#[test]
fn setup_local_emits_and_expose_types_resolve_through_setup() {
    let code = generate(WIDGET, None);
    let tail = module_tail(&code);
    assert!(!tail.contains("WidgetProps"), "{code}");
    assert!(
        tail.contains(
            "export type Emits = Awaited<ReturnType<typeof __setup>>[\"__vize_typed_emits\"];"
        ),
        "{code}"
    );
    assert!(
        tail.contains(
            "export type Exposed = Awaited<ReturnType<typeof __setup>>[\"__vize_typed_exposed\"];"
        ),
        "{code}"
    );
    assert!(
        code.contains(
            "const __vize_typed_emits = undefined as unknown as WidgetEmits<WidgetProps>;"
        ),
        "{code}"
    );
    assert!(
        code.contains(
            "const __vize_typed_exposed = undefined as unknown as WidgetExpose<WidgetProps>;"
        ),
        "{code}"
    );
}

#[test]
fn setup_value_query_in_emits_resolves_through_setup() {
    let script = r#"const def = { size: 1 };
const emit = defineEmits<{ update: [value: typeof def] }>();
"#;
    let code = generate(script, None);
    let tail = module_tail(&code);
    assert!(!tail.contains("typeof def"), "{code}");
    assert!(tail.contains("[\"__vize_typed_emits\"]"), "{code}");
}

#[test]
fn generic_setup_local_expose_type_redeclares_parameters() {
    let script = r#"const props = defineProps<{ item: T }>();
type Local = { item: T };
defineExpose<{ current: () => Local }>({ current: () => ({ item: props.item }) });
"#;
    let code = generate(script, Some("T extends string"));
    let tail = module_tail(&code);
    assert!(
        tail.contains(
            "export type Exposed<T extends string = any> = Awaited<ReturnType<typeof __setup<T>>>[\"__vize_typed_exposed\"];"
        ),
        "{code}"
    );
}

#[test]
fn module_visible_emits_and_expose_types_stay_module_aliases() {
    let script = r#"import type { Api } from './types';
interface Events { change: [value: string] }
defineEmits<Events>();
defineExpose<Api>();
"#;
    let code = generate(script, None);
    let tail = module_tail(&code);
    assert!(tail.contains("export type Emits = Events;"), "{code}");
    assert!(tail.contains("export type Exposed = Api;"), "{code}");
    assert!(!code.contains("__vize_typed_emits"), "{code}");
    assert!(!code.contains("__vize_typed_exposed"), "{code}");
}
