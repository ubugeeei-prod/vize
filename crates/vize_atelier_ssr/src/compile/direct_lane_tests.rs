//! The direct SSR lane against the full path.
//!
//! The direct lane runs no legacy parser and no legacy transform, so it may
//! only take a template those two pass without a diagnostic, and it must emit
//! the module the full path emits. Debug builds assert both inside the lane;
//! these tests pin them in every profile and pin which templates decline.

use vize_atelier_core::options::{CustomElementMatcher, TemplateSyntaxMode};
use vize_l0::Allocator;

use super::{
    SsrLane, compile_ssr_on_lane_with_runtime, compile_ssr_sfc_template, emit_on_direct_lane,
};
use crate::{SsrCompilerExperimentalOptions, SsrCompilerOptions, SsrSfcContext};

/// Templates the legacy parser and transform pass without a word.
const CLEAN: &[&str] = &[
    "<div>hello</div>",
    "<p class=\"a\" :class=\"b\" :style=\"s\">{{ msg }} and {{ other }}</p>",
    "<ul><li v-for=\"item in items\" :key=\"item.id\">{{ item.name }}</li></ul>",
    "<section><p v-if=\"ok\">yes</p><p v-else-if=\"maybe\">maybe</p><p v-else>no</p></section>",
    "<input v-model=\"text\"><textarea v-model=\"body\"></textarea>",
    "<button :disabled=\"busy\" @click=\"save\">Save</button>",
    "<Layout><template #header=\"{ title }\"><h1>{{ title }}</h1></template><p>body</p></Layout>",
    "<Comp v-slot=\"{ row }\" :rows=\"rows\">{{ row.label }}</Comp>",
    "<div v-show=\"open\" v-html=\"markup\"></div>",
    "<template v-if=\"ready\"><span>a</span><span>b</span></template>",
];

/// Templates the lane must leave to the full path, with why.
const DECLINED: &[(&str, &str)] = &[
    (
        "a frozen subtree needs the parser's custody",
        "<div v-pre>{{ raw }}</div>",
    ),
    (
        "a v-model argument on an element is an error",
        "<input v-model:foo=\"bar\">",
    ),
    (
        "malformed slot props are an error",
        "<Comp v-slot=\"(dep(, file)\">{{ file }}</Comp>",
    ),
    (
        "mixed slot usage is an error",
        "<Layout #default=\"p\"><p>{{ p.title }}</p><template #header><h1>T</h1></template></Layout>",
    ),
    (
        "an empty directive modifier is an error",
        "<div :key..prop=\"a\"></div>",
    ),
    (
        "a directive on a slot outlet is an error",
        "<slot v-example=\"payload\"></slot>",
    ),
    ("a stray end tag is a parse error", "<div>text</div></p>"),
];

struct Compiled {
    diagnostics: std::vec::Vec<std::string::String>,
    code: std::string::String,
    preamble: std::string::String,
}

fn direct_lane(source: &str) -> Option<Compiled> {
    let allocator = Allocator::new();
    emit_on_direct_lane::<vize_atelier_core::codegen::PlainRenderMap>(
        &allocator,
        source,
        &SsrCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
        &CustomElementMatcher::default(),
        &SsrCompilerExperimentalOptions::default(),
        SsrSfcContext::new(false),
        "vue",
    )
    .map(|result| Compiled {
        diagnostics: std::vec::Vec::new(),
        code: result.code.as_str().into(),
        preamble: result.preamble.as_str().into(),
    })
}

fn full_path(source: &str) -> Compiled {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr_on_lane_with_runtime::<vize_atelier_core::codegen::PlainRenderMap>(
        &allocator,
        source,
        SsrCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        SsrCompilerExperimentalOptions::default(),
        SsrSfcContext::new(false),
        SsrLane::Selected,
        "vue",
    );
    Compiled {
        diagnostics: errors
            .iter()
            .map(|error| std::format!("{:?}: {}", error.code, error.message))
            .collect(),
        code: result.code.as_str().into(),
        preamble: result.preamble.as_str().into(),
    }
}

fn sfc_entry(source: &str, direct: bool) -> Compiled {
    let allocator = Allocator::new();
    let (errors, result) = compile_ssr_sfc_template(
        &allocator,
        source,
        SsrCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        SsrCompilerExperimentalOptions::default(),
        SsrSfcContext::new(false),
        &vize_atelier_core::CodegenOptions::default(),
        direct,
    );
    Compiled {
        diagnostics: errors
            .iter()
            .map(|error| std::format!("{:?}: {}", error.code, error.message))
            .collect(),
        code: result.code.as_str().into(),
        preamble: result.preamble.as_str().into(),
    }
}

#[test]
fn clean_templates_take_the_direct_lane_and_emit_the_full_path_module() {
    for source in CLEAN {
        let direct = direct_lane(source);
        let full = full_path(source);
        assert_eq!(
            full.diagnostics,
            std::vec::Vec::<std::string::String>::new(),
            "{source}"
        );
        let Some(direct) = direct else {
            panic!("the direct lane declined a clean template: {source}");
        };
        assert_eq!(direct.code, full.code, "{source}");
        assert_eq!(direct.preamble, full.preamble, "{source}");
        assert_ne!(direct.code, "", "{source}");
    }
}

#[test]
fn diagnosed_and_frozen_templates_stay_on_the_full_path() {
    for (why, source) in DECLINED {
        assert!(direct_lane(source).is_none(), "{why}: {source}");
    }
    // Every decline but the frozen one is a template the full path diagnoses,
    // so emitting it directly would have dropped a diagnostic.
    for (why, source) in DECLINED.iter().skip(1) {
        assert_ne!(
            full_path(source).diagnostics,
            std::vec::Vec::<std::string::String>::new(),
            "{why}: {source}"
        );
    }
}

#[test]
fn the_sfc_entry_returns_the_same_result_with_and_without_the_direct_lane() {
    for source in CLEAN
        .iter()
        .copied()
        .chain(DECLINED.iter().map(|(_, source)| *source))
    {
        let direct = sfc_entry(source, true);
        let full = sfc_entry(source, false);
        assert_eq!(direct.diagnostics, full.diagnostics, "{source}");
        assert_eq!(direct.code, full.code, "{source}");
        assert_eq!(direct.preamble, full.preamble, "{source}");
    }
}
