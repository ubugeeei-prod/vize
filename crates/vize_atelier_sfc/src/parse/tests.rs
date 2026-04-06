use super::parse_sfc;
use std::borrow::Cow;

#[test]
fn test_parse_empty_sfc() {
    let result = parse_sfc("", Default::default()).unwrap();
    assert!(result.template.is_none());
    assert!(result.script.is_none());
    assert!(result.styles.is_empty());
}

#[test]
fn test_parse_template_only() {
    let source = "<template><div>Hello</div></template>";
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.template.is_some());
    let template = result.template.unwrap();
    assert_eq!(template.content, "<div>Hello</div>");
}

#[test]
fn test_parse_with_lang_attr() {
    let source = r#"<script lang="ts">const x: number = 1</script>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script.is_some());
    let script = result.script.unwrap();
    assert_eq!(script.lang.as_deref(), Some("ts"));
}

#[test]
fn test_parse_multiple_styles() {
    let source = r#"
<style>.a {}</style>
<style scoped>.b {}</style>
<style lang="scss">.c {}</style>
"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert_eq!(result.styles.len(), 3);
    assert!(!result.styles[0].scoped);
    assert!(result.styles[1].scoped);
    assert_eq!(result.styles[2].lang.as_deref(), Some("scss"));
}

#[test]
fn test_parse_custom_block() {
    let source = r#"
<template><div></div></template>
<i18n>{"en": {"hello": "Hello"}}</i18n>
"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert_eq!(result.custom_blocks.len(), 1);
    assert_eq!(result.custom_blocks[0].block_type, "i18n");
}

#[test]
fn test_parse_script_setup() {
    let source = r#"
<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0)
</script>
"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    assert!(script.setup);
    assert_eq!(script.lang.as_deref(), Some("ts"));
}

#[test]
fn test_zero_copy_content() {
    let source = "<template><div>Hello World</div></template>";
    let result = parse_sfc(source, Default::default()).unwrap();

    // Verify that content is borrowed (Cow::Borrowed)
    let template = result.template.unwrap();
    match &template.content {
        Cow::Borrowed(s) => {
            // The string should be a slice of the original source
            let ptr = s.as_ptr();
            let source_ptr = source.as_ptr();
            assert!(ptr >= source_ptr && ptr < unsafe { source_ptr.add(source.len()) });
        }
        Cow::Owned(_) => panic!("Expected Cow::Borrowed, got Cow::Owned"),
    }
}

#[test]
fn test_closing_template_tag_with_whitespace() {
    // Test that closing </template> tag with whitespace before '>' is handled correctly
    let source = r#"<script setup>
const x = 1
</script>

<template
  ><div>Hello</div></template
>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.template.is_some());
    let template = result.template.unwrap();
    assert_eq!(template.content, "<div>Hello</div>");
}

#[test]
fn test_closing_template_tag_with_newline() {
    // Test that closing </template> tag with newline before '>' is handled correctly
    // This pattern appears in some Vue files with specific formatting
    let source = r#"<template>
  <div>Content</div>
</template
>

<style>
.foo {}
</style>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.template.is_some());
    let template = result.template.unwrap();
    assert!(template.content.contains("<div>Content</div>"));

    assert_eq!(result.styles.len(), 1);
}

#[test]
fn test_nested_template_in_string_literal() {
    // Test that embedded <template> tags inside string literals don't confuse the parser
    let source = r#"<script setup lang="ts">
const code = `<template>
  <div>Nested</div>
</template>`
</script>

<template>
  <div>{{ code }}</div>
</template>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    assert!(result.template.is_some());

    let template = result.template.unwrap();
    // The main template should be the one at depth 0, not the one in the string
    assert!(template.content.contains("{{ code }}"));
}

#[test]
fn test_template_with_v_slot_syntax() {
    // Test that <template v-slot> and <template #name> are handled as nested, not root
    let source = r#"<template>
  <MyComponent>
    <template #header>Header</template>
    <template v-slot:footer>Footer</template>
  </MyComponent>
</template>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.template.is_some());
    let template = result.template.unwrap();
    // Should contain the nested templates as content
    assert!(template.content.contains("<template #header>"));
    assert!(template.content.contains("<template v-slot:footer>"));
}

#[test]
fn test_multiline_closing_tag_complex() {
    // Test complex case with multiple blocks and multiline closing tags
    let source = r#"<script setup>
const x = `</template>`  // embedded in string
</script>

<template
><div class="container">
    <template v-if="show">
      Content
    </template
    ><template v-else>
      Other
    </template>
  </div></template
>

<style scoped>
.container {}
</style>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    assert!(result.template.is_some());
    assert_eq!(result.styles.len(), 1);
    assert!(result.styles[0].scoped);

    let template = result.template.unwrap();
    assert!(template.content.contains("<div class=\"container\">"));
    assert!(template.content.contains("<template v-if=\"show\">"));
    assert!(template.content.contains("<template v-else>"));
}

#[test]
fn test_script_with_embedded_closing_tag_in_template_literal() {
    // Test that </script> inside a template literal doesn't end the script block
    let source = r#"<script setup lang="ts">
const code = `<script setup>
console.log('hello')
</script>`
const x = 1
</script>

<template>
  <div>{{ code }}</div>
</template>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    // The script content should include everything up to the real </script>
    assert!(script.content.contains("const code = `<script setup>"));
    assert!(script.content.contains("</script>`"));
    assert!(script.content.contains("const x = 1"));
}

#[test]
fn test_script_with_embedded_closing_tag_in_single_quote() {
    // Test that </script> inside a single-quoted string doesn't end the script block
    let source = r#"<script setup>
const tag = '</script>'
const y = 2
</script>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    assert!(script.content.contains("const tag = '</script>'"));
    assert!(script.content.contains("const y = 2"));
}

#[test]
fn test_script_with_embedded_closing_tag_in_double_quote() {
    // Test that </script> inside a double-quoted string doesn't end the script block
    let source = r#"<script setup>
const tag = "</script>"
const z = 3
</script>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    assert!(script.content.contains(r#"const tag = "</script>""#));
    assert!(script.content.contains("const z = 3"));
}

#[test]
fn test_script_with_embedded_closing_tag_in_comment() {
    // Test that </script> inside comments doesn't end the script block
    let source = r#"<script setup>
// This is a comment: </script>
const a = 1
/* Multi-line comment
   </script>
*/
const b = 2
</script>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    assert!(script.content.contains("// This is a comment: </script>"));
    assert!(script.content.contains("const a = 1"));
    assert!(script.content.contains("</script>"));
    assert!(script.content.contains("const b = 2"));
}

#[test]
fn test_script_with_template_literal_expression() {
    // Test that template literal with ${} expressions is handled correctly
    let source = r#"<script setup>
const name = 'world'
const code = `Hello ${name}! </script> ${1 + 2}`
const c = 3
</script>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    assert!(script
        .content
        .contains("const code = `Hello ${name}! </script> ${1 + 2}`"));
    assert!(script.content.contains("const c = 3"));
}

#[test]
fn test_script_with_escaped_quotes() {
    // Test that escaped quotes in strings are handled correctly
    let source = r#"<script setup>
const str1 = "He said \"</script>\""
const str2 = 'It\'s </script> here'
const str3 = `Template \` </script> \``
const d = 4
</script>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    assert!(script.content.contains("const d = 4"));
}

#[test]
fn test_script_with_regex_containing_quotes() {
    // Test that regex literals containing quotes don't confuse the parser
    // This is important for Monaco editor tokenizer patterns like: [/`[^`]*`/, "string"]
    let source = r#"<script setup>
const tokenizer = {
  root: [
    [/<script[^>]*>/, "tag"],
    [/"[^"]*"/, "string"],
    [/'[^']*'/, "string"],
    [/`[^`]*`/, "string"],
  ]
}
const e = 5
</script>

<template>
  <div>Test</div>
</template>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    assert!(script.content.contains("const tokenizer"));
    assert!(script.content.contains(r#"[/`[^`]*`/, "string"]"#));
    assert!(script.content.contains("const e = 5"));

    assert!(result.template.is_some());
    let template = result.template.unwrap();
    assert!(template.content.contains("<div>Test</div>"));
}

#[test]
fn test_script_with_division_operator() {
    // Test that division operator doesn't interfere with string detection
    let source = r#"<script setup>
const x = 10 / 2
const y = "test"
const z = x / y
</script>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    assert!(script.content.contains("const x = 10 / 2"));
    assert!(script.content.contains(r#"const y = "test""#));
}

#[test]
fn test_script_with_tagged_template_literal() {
    // Test that tagged template literals (e.g., html`...`) are handled correctly
    // The backtick after an identifier should still be treated as a string start
    let source = r#"<script setup>
const tag = html`<span style="color: red">Hello</span>`
const result = css`
  .container {
    color: blue;
  }
`
const x = 1
</script>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    assert!(script.content.contains("html`<span"));
    assert!(script.content.contains("css`"));
    assert!(script.content.contains("const x = 1"));
}

#[test]
fn test_script_with_keyword_template_literal() {
    // Test that template literals after keywords (return, throw) are handled correctly
    // This pattern is common: return `<span>${x}</span>`
    let source = r#"<script setup>
function render() {
  const x = 'test'
  return `<span>${x}</span>`
}

function throwError() {
  throw `Error: </script>`
}
const y = 2
</script>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    assert!(script.content.contains("return `<span>"));
    assert!(script.content.contains("throw `Error: </script>`"));
    assert!(script.content.contains("const y = 2"));
}

#[test]
fn test_script_with_method_call_template_literal() {
    // Test that template literals after method calls (e.g., foo()`...`) are handled
    let source = r#"<script setup>
const result = getTemplate()`<div>${content}</div>`
const arr = items.map((x) => `<li>${x}</li>`)
const z = 3
</script>"#;
    let result = parse_sfc(source, Default::default()).unwrap();

    assert!(result.script_setup.is_some());
    let script = result.script_setup.unwrap();
    assert!(script.content.contains(r#"getTemplate()`<div>"#));
    assert!(script.content.contains(r#"`<li>${x}</li>`"#));
    assert!(script.content.contains("const z = 3"));
}

#[test]
fn test_tags_in_code_comments() {
    // HTML comments containing pseudo-tags must not be parsed as SFC blocks
    let source = r#" <!-- <script> </script> -->
<!-- <script>
console.log("HI")
</script> -->
<script setup>
const x = 1
</script>
<template><div>{{ x }}</div></template>"#;

    let result = parse_sfc(source, Default::default()).unwrap();

    // The comment must not create a spurious script block
    assert!(result.script.is_none());
    assert!(result.template.is_some());
    assert!(result.script_setup.is_some());
    assert!(result.script_setup.unwrap().content.contains("const x = 1"));
}
