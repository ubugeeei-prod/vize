use oxc_allocator::Allocator;
use oxc_parser::Parser;

use super::*;

fn names(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, super::super::script_source_type()).parse();
    setup_macros(&parsed.program)
        .iter()
        .map(|found| match found.name {
            "defineModel" => format!("model:{}", DefineModel::of(found.call).name),
            name => name.to_owned(),
        })
        .collect()
}

#[test]
fn recognizes_only_top_level_identifier_calls() {
    assert_eq!(
        names(
            "const a = defineModel('a'); defineModel(); const p = withDefaults(defineProps<{ x?: 1 }>(), {});\n\
             form.defineModel(); const c = defineModel() as any; function f() { defineModel('n') }\n\
             const t = defineModel(`t`); const s = defineModel(('s' as string))"
        ),
        [
            "model:a",
            "model:modelValue",
            "defineProps",
            "model:modelValue",
            "model:s"
        ]
    );
}
