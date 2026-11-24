use std::borrow::Cow;

use crate::types::{BindingMetadata, BlockLocation, SfcTemplateBlock, TemplateCompileOptions};

use super::extraction::{
    extract_template_parts, extract_template_parts_full, render_fn_runs, slice_template_parts,
    slice_template_parts_full, slice_template_parts_runs,
};
use super::{TemplateBlockCompileContext, TemplateBlockCompileResult, compile_template_block};

const TEMPLATES: [&str; 9] = [
    // Plain element + interpolation
    "<div>{{ msg }}</div>",
    // Static-only content (hoistable, may need no assets)
    "<div><img style=\"position: absolute; top: 0\" alt=\"x\"></div>",
    // Unresolved component + directive => asset preamble lines
    "<MyWidget v-focus>{{ count + 1 }}</MyWidget>",
    // Root-level v-if/v-else => multi-line ternary return
    "<div v-if=\"shown\">a</div>\n<span v-else>b</span>",
    // Multi-root fragment
    "<header>h</header>\n<footer>f</footer>",
    // v-for with nested interpolation
    "<ul><li v-for=\"item in items\" :key=\"item.id\">{{ item.name }}</li></ul>",
    // Slot outlet root
    "<slot name=\"body\" :row=\"row\" />",
    // Event handlers (cached) + v-model
    "<input v-model=\"text\" @keyup.enter=\"submit($event)\">",
    // Plain text root
    "hello",
];

fn compile(source: &str, inline: bool, hoist_static: bool) -> TemplateBlockCompileResult {
    let template = SfcTemplateBlock {
        content: Cow::Borrowed(source),
        loc: BlockLocation {
            start: 0,
            end: 0,
            tag_start: 0,
            tag_end: 0,
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 1,
        },
        lang: None,
        src: None,
        attrs: Default::default(),
    };
    let bindings = BindingMetadata::default();
    let template_allocator = vize_l0::Allocator::new();
    let codegen_options = vize_atelier_core::CodegenOptions {
        source_map: true,
        ..Default::default()
    };
    let mut template_options = TemplateCompileOptions::default();
    if !hoist_static {
        let compiler_options = vize_atelier_dom::DomCompilerOptions {
            hoist_static: false,
            ..Default::default()
        };
        template_options.compiler_options = Some(compiler_options);
    }
    compile_template_block(
        &template_allocator,
        &template,
        &template_options,
        &vize_atelier_core::options::CustomElementMatcher::default(),
        TemplateBlockCompileContext {
            scope_id: "abc123",
            apply_scope_id: false,
            has_scoped: true,
            slotted: true,
            is_ts: false,
            inline,
            component_name: Some("TestComp"),
            experimental_self_component: false,
            bindings: Some(&bindings),
            script_tables: None,
            authored_source: None,
            pug_source_map: None,
        },
        vize_atelier_core::TemplateSyntaxMode::Standard,
        &codegen_options,
    )
    .expect("template should compile")
}

/// The bytes a part's provenance accounts for.
fn copied(runs: &crate::module_map::Runs) -> usize {
    runs.runs().iter().map(|run| run.len).sum()
}

/// The emission-recorded section offsets must reproduce exactly what the
/// line scanner extracts, for every shape of generated render module the
/// SFC DOM path can produce.
#[test]
fn test_slice_template_parts_matches_line_scanner() {
    for inline in [true, false] {
        for source in TEMPLATES {
            let result = compile(source, inline, inline);
            let sections = result
                .sections
                .expect("DOM lane must record section offsets");
            let sliced = slice_template_parts(&result.code, &sections);
            let scanned = extract_template_parts(&result.code);

            assert_eq!(
                sliced, scanned,
                "sliced sections must match the line scanner for inline={inline} template:\n{source}\n\ngenerated code:\n{}",
                result.code
            );

            // The provenance the source map is composed through names
            // exactly the bytes each part was cut from (#13547).
            let (_, _, preamble, render_body, _) = sliced;
            let (preamble_runs, body_runs) = slice_template_parts_runs(&result.code, &sections);
            assert_eq!(
                (
                    preamble_runs.clone().verified(&preamble, &result.code),
                    body_runs.clone().verified(&render_body, &result.code),
                    copied(&preamble_runs),
                    copied(&body_runs),
                ),
                (preamble_runs, body_runs, preamble.len(), render_body.len()),
                "inline={inline} template:\n{source}"
            );
        }
    }
}

/// Separate-template assembly slices the recorded render function, so it must
/// equal the brace-counting line scanner, including bodies whose strings,
/// template literals and nested functions carry braces.
#[test]
fn test_slice_template_parts_full_matches_line_scanner() {
    let braces = [
        "<p :title=\"`a${b}{`\">{{ `}${c}` }}</p>",
        "<button @click=\"() => { open = !open; log('}') }\">{{ { a: 1 }.a }}</button>",
        "<div :style=\"{ color: 'red', '--x': `{${y}}` }\"><span v-if=\"z\">/* } */</span></div>",
    ];
    for hoist_static in [true, false] {
        for source in TEMPLATES.into_iter().chain(braces) {
            let result = compile(source, false, hoist_static);
            let sections = result
                .sections
                .expect("DOM lane must record section offsets");
            let sliced = slice_template_parts_full(&result.code, &sections);
            let scanned = extract_template_parts_full(&result.code);

            assert_eq!(
                sliced, scanned,
                "sliced sections must match the line scanner for hoist_static={hoist_static} template:\n{source}\n\ngenerated code:\n{}",
                result.code
            );

            // The render function's provenance covers every byte of it.
            let render_fn = &sliced.2;
            assert_eq!(
                copied(&render_fn_runs(&result, render_fn)),
                render_fn.len(),
                "hoist_static={hoist_static} template:\n{source}"
            );
        }
    }
}
