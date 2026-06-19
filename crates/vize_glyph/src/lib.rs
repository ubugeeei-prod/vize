//! # vize_glyph
//!
//! Glyph - The beautiful letterforms for Vize.
//! High-performance formatter for Vue.js Single File Components.
//!
//! ## Name Origin
//!
//! **Glyph** (/ɡlɪf/) refers to the visual representation of a character -
//! the elegant form that gives meaning to written symbols. In typography and
//! calligraphy, glyphs are carefully crafted to be both beautiful and legible.
//! `vize_glyph` shapes Vue SFC code into its most readable and consistent form.
//!
//! ## Performance
//!
//! This crate is designed for maximum performance:
//! - Arena allocation via `vize_carton::Allocator` for minimal heap allocations
//! - Zero-copy parsing where possible
//! - SIMD-accelerated string operations via `memchr`
//! - Efficient buffer management with pre-allocated capacity
//!
//! ## Example
//!
//! ```ignore
//! use vize_glyph::{format_sfc, FormatOptions};
//!
//! let source = r#"
//! <script setup>
//! import {ref} from 'vue'
//! const count=ref(0)
//! </script>
//! <template>
//!   <button @click="count++">{{count}}</button>
//! </template>
//! "#;
//!
//! let options = FormatOptions::default();
//! let result = format_sfc(source, &options).unwrap();
//! println!("{}", result.code);
//! ```

mod error;
mod formatter;
mod options;
mod script;
mod style;
mod template;

pub use error::*;
pub use formatter::*;
pub use options::*;

// Re-export allocator for external use
pub use vize_carton::Allocator;
use vize_carton::String;

/// Format a Vue SFC source string
///
/// This is the main entry point for formatting Vue Single File Components.
/// Uses arena allocation for efficient memory management.
#[inline]
pub fn format_sfc(source: &str, options: &FormatOptions) -> Result<FormatResult, FormatError> {
    let allocator = Allocator::with_capacity(source.len() * 2);
    format_sfc_with_allocator(source, options, &allocator)
}

/// Format a Vue SFC source string with a provided allocator
///
/// Use this when you want to reuse an allocator across multiple format operations.
#[inline]
pub fn format_sfc_with_allocator(
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
) -> Result<FormatResult, FormatError> {
    let formatter = GlyphFormatter::new(options, allocator);
    formatter.format(source)
}

/// Format only the script/TypeScript content
#[inline]
pub fn format_script(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let allocator = Allocator::with_capacity(source.len() * 2);
    script::format_ts_script_content_stable(source, options, &allocator)
}

/// Format only script content with an explicit JavaScript/TypeScript source type.
#[inline]
pub fn format_script_with_source_type(
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
    source_type: oxc_span::SourceType,
) -> Result<String, FormatError> {
    script::format_script_content_stable(source, options, allocator, source_type)
}

/// Format only the template content
#[inline]
pub fn format_template(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    template::format_template_content(source, options)
}

/// Format only the CSS/style content
#[inline]
pub fn format_style(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    style::format_style_content(source, options)
}

#[cfg(test)]
mod tests {
    use super::{Allocator, FormatOptions, format_script, format_sfc, format_sfc_with_allocator};

    #[test]
    fn test_format_simple_sfc() {
        let source = r#"<script setup>
import {ref} from 'vue'
const count=ref(0)
</script>

<template>
<div>{{ count }}</div>
</template>
"#;
        let options = FormatOptions::default();
        let result = format_sfc(source, &options).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_format_script_only() {
        let source = "const x=1;const y={a:1,b:2}";
        let options = FormatOptions::default();
        let result = format_script(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_sfc_preserves_structure() {
        let source = r#"<script setup lang="ts">
const msg = 'hello'
</script>

<template>
  <div>{{ msg }}</div>
</template>

<style scoped>
.container { color: red; }
</style>
"#;
        let options = FormatOptions::default();
        let result = format_sfc(source, &options).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_format_sfc_preserves_block_attrs() {
        let source = r#"<template functional>
  <div>{{ msg }}</div>
</template>

<script setup lang="ts" generic="T extends string">
const msg = 'hello'
</script>

<style scoped module lang="scss">
.container { color: red; }
</style>

<i18n global locale="en">
{"hello":"Hello"}
</i18n>
"#;
        let options = FormatOptions::default();
        let result = format_sfc(source, &options).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_format_sfc_tsx_script_block() {
        let source = r#"<script setup lang="tsx">
const Card=(props:{title:string;items:string[]})=><section class="card"><h2>{props.title}</h2>{props.items.map((item)=><span key={item}>{item}</span>)}</section>
</script>

<template><Card title="Docs" :items="['a']" /></template>
"#;
        let options = FormatOptions::default();
        let result = format_sfc(source, &options).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_format_sfc_jsx_script_block() {
        let source = r#"<script lang="jsx">
export default function App(){return <><button onClick={()=>emit('save')}>Save</button></>}
</script>
"#;
        let options = FormatOptions::default();
        let result = format_sfc(source, &options).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_format_sfc_preserves_leading_comment() {
        let source = r#"<!--
SPDX-FileCopyrightText: Example Author
SPDX-License-Identifier: MIT
-->

<template>
<div>Hello</div>
</template>

<script setup lang="ts">
const message = 'hello';
</script>
"#;
        let options = FormatOptions::default();
        let result = format_sfc(source, &options).unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn test_format_sfc_multiline_interpolation_is_idempotent() {
        // Regression for #957: a long inline interpolation whose JS
        // expression wraps onto multiple lines used to converge only on
        // the second `vize fmt` pass; the first pass mis-indented the
        // wrapped expression to column 0. The first pass must already
        // produce canonical multi-line shape.
        let source = "<template>\n  <div>\n    <span>{{ new Date(version.date).toLocaleDateString(\"en-US\", { year: \"numeric\", month: \"short\", day: \"numeric\" }) }}</span>\n  </div>\n</template>\n";
        let options = FormatOptions::default();
        let first = format_sfc(source, &options).unwrap();
        let second = format_sfc(&first.code, &options).unwrap();
        assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    }

    #[test]
    fn test_format_sfc_multiline_attribute_value_is_idempotent() {
        // Regression: lines inside a multi-line attribute value (e.g. a
        // wrapped `class` string) are emitted verbatim by the template
        // formatter, but the SFC layer stacked one extra indent level on
        // them per pass, so `fmt` never reached a fixed point.
        let source = "<template>\n  <button\n    class=\"\n      flex items-center gap-2\n      text-start text-lg\n    \"\n    @click=\"go\"\n  >\n    hi\n  </button>\n</template>\n";
        let options = FormatOptions::default();
        let first = format_sfc(source, &options).unwrap();
        let second = format_sfc(&first.code, &options).unwrap();
        let third = format_sfc(&second.code, &options).unwrap();
        assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
        assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
    }

    #[test]
    fn test_format_sfc_multiline_comment_is_idempotent() {
        // Regression: inner lines of a multi-line HTML comment are emitted
        // verbatim by the template formatter, but the SFC layer stacked one
        // extra indent level on them per pass.
        let source = "<template>\n  <div>\n    <!-- <div v-if=\"result.action\">\n      {{ result.action!.label }}\n    </div> -->\n    <span>hi</span>\n  </div>\n</template>\n";
        let options = FormatOptions::default();
        let first = format_sfc(source, &options).unwrap();
        let second = format_sfc(&first.code, &options).unwrap();
        let third = format_sfc(&second.code, &options).unwrap();
        assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
        assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
    }

    #[test]
    fn test_format_sfc_multiline_interpolation_with_trailing_text_is_idempotent() {
        let source = "<template>\n  <div>\n    <span>\n      {{ $t(\"compose.drafts\", nonEmptyDrafts.length, { named: { v: formatNumber(nonEmptyDrafts.length) } }) }}&#160;\n    </span>\n  </div>\n</template>\n";
        let options = FormatOptions::default();
        let first = format_sfc(source, &options).unwrap();
        let second = format_sfc(&first.code, &options).unwrap();
        assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    }

    #[test]
    fn test_format_sfc_text_between_interpolations_is_idempotent() {
        // Regression: a text segment between two block-form interpolations
        // kept its leading space, shifting the line one column per pass.
        let source = "<template>\n  <span>\n    {{ tsx.compressedToX({ x: bytes(item.compressedSize), yyyyyyyyyyyyyyyy: zzzzzzzzzzzzzz }) }} = {{ tsx.savedXPercent({ x: Math.round((1 - item.compressedSize / item.file.size) * 100) }) }}\n  </span>\n</template>\n";
        let options = FormatOptions::default();
        let first = format_sfc(source, &options).unwrap();
        let second = format_sfc(&first.code, &options).unwrap();
        let third = format_sfc(&second.code, &options).unwrap();
        assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
        assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
    }

    #[test]
    fn test_format_sfc_multiline_pre_open_tag_is_idempotent() {
        // Regression: a `<pre>` whose opening tag wraps attributes across
        // lines was not recognized as a raw region by the SFC layer, so the
        // verbatim content and closing tag gained one indent per pass.
        let source = "<template>\n  <div>\n    <pre\n      v-else-if=\"parsedJSON\"\n      class=\"overflow-auto max-h-96\"\n    >{{ formattedJSONString }}\n      </pre>\n  </div>\n</template>\n";
        let options = FormatOptions::default();
        let first = format_sfc(source, &options).unwrap();
        let second = format_sfc(&first.code, &options).unwrap();
        let third = format_sfc(&second.code, &options).unwrap();
        assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
        assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
    }

    #[test]
    fn test_format_sfc_class_component_preserves_decorators() {
        // #1391 PR8: a decorated class-component SFC (vue-property-decorator
        // style) must round-trip through the script formatter without losing
        // or mangling decorators, the class body, or member decorators. oxc
        // formats the class verbatim apart from quote/semicolon normalization.
        let source = r#"<script lang="ts">
import { Vue, Component, Prop, Emit } from 'vue-property-decorator'

@Component
export default class MyComponent extends Vue {
  @Prop({ default: 0 }) readonly value!: number
  count = 0
  get doubled(): number {
    return this.count * 2
  }
  increment(): void {
    this.count++
  }
  @Emit('change')
  onChange(): number {
    return this.count
  }
}
</script>

<template>
  <div>{{ doubled }}</div>
</template>
"#;
        let options = FormatOptions::default();
        let first = format_sfc(source, &options).unwrap();
        let code = first.code.as_str();

        // Decorators and class structure preserved.
        assert!(code.contains("@Component"), "@Component decorator dropped");
        assert!(
            code.contains("export default class MyComponent extends Vue"),
            "class declaration mangled"
        );
        assert!(
            code.contains("@Prop({ default: 0 }) readonly value!: number;"),
            "@Prop member decorator dropped or mangled"
        );
        assert!(
            code.contains("@Emit(\"change\")"),
            "@Emit member decorator dropped or mangled"
        );
        assert!(code.contains("get doubled(): number"), "getter dropped");
        assert!(code.contains("increment(): void"), "method dropped");

        // Idempotent: a second pass is a no-op.
        let second = format_sfc(code, &options).unwrap();
        assert_eq!(
            first.code, second.code,
            "fmt; fmt must be a no-op for class components"
        );
    }

    #[test]
    fn test_format_sfc_class_component_options_decorator() {
        // #1391 PR8: the vue-class-component v8 `@Options({...})` form is also
        // preserved, including the decorator-argument options object.
        let source = r#"<script lang="ts">
import { Vue, Options } from 'vue-class-component'

@Options({
  components: { Foo },
})
export default class MyComponent extends Vue {
  count = 0
}
</script>
"#;
        let options = FormatOptions::default();
        let first = format_sfc(source, &options).unwrap();
        let code = first.code.as_str();
        assert!(code.contains("@Options({"), "@Options decorator dropped");
        assert!(
            code.contains("components: { Foo },"),
            "decorator-argument options object mangled"
        );
        let second = format_sfc(code, &options).unwrap();
        assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    }

    #[test]
    fn test_format_sfc_script_parse_error_degrades_to_unchanged_script() {
        // #1391 PR8: an unparseable script body must NOT fail the whole-SFC
        // format. The script is left unchanged (trimmed) and the template is
        // still formatted, mirroring the style block's fallback behaviour.
        let source = r#"<script lang="ts">
const x = ;
@@@ not valid typescript
</script>

<template>
<div>{{ x }}</div>
</template>
"#;
        let options = FormatOptions::default();
        let result =
            format_sfc(source, &options).expect("parse error must not abort the SFC format");
        let code = result.code.as_str();
        // Unparseable script preserved verbatim (trimmed).
        assert!(
            code.contains("const x = ;"),
            "unparseable script body should be left unchanged"
        );
        assert!(
            code.contains("@@@ not valid typescript"),
            "unparseable script body should be left unchanged"
        );
        // Template still got formatted/indented (the interpolation expands
        // onto its own indented line instead of being left untouched).
        assert!(
            code.contains("  <div>\n    {{ x }}\n  </div>"),
            "template should still be formatted when script fails to parse, got:\n{code}"
        );
    }

    #[test]
    fn test_allocator_reuse() {
        let allocator = Allocator::with_capacity(4096);
        let options = FormatOptions::default();

        let source1 = "<script>const a = 1</script>";
        let source2 = "<script>const b = 2</script>";

        let result1 = format_sfc_with_allocator(source1, &options, &allocator).unwrap();
        let result2 = format_sfc_with_allocator(source2, &options, &allocator).unwrap();

        insta::assert_snapshot!(result1.code.as_str());
        insta::assert_snapshot!(result2.code.as_str());
    }
}
