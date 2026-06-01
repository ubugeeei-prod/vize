//! Virtual code generator that transforms SFC into virtual documents.
//!
//! Uses arena allocation from vize_carton for optimal performance.
#![allow(clippy::disallowed_methods)]

use vize_atelier_sfc::SfcDescriptor;
use vize_carton::Bump;
use vize_carton::cstr;

use super::{
    ScriptCodeGenerator, StyleCodeGenerator, TemplateCodeGenerator, VirtualDocument,
    VirtualDocuments, VirtualLanguage,
    script_code::extract_simple_bindings,
    template_code::{TemplateExpression, extract_expressions},
};

/// Virtual code generator for SFC files.
///
/// This generator transforms Vue SFC files into virtual documents for each
/// embedded language (template, script, style). It uses arena allocation
/// for temporary parsing data to minimize allocations.
pub struct VirtualCodeGenerator {
    /// Template code generator (reusable)
    template_gen: TemplateCodeGenerator,
    /// Script code generator (reusable)
    script_gen: ScriptCodeGenerator,
    /// Style code generator (reusable)
    style_gen: StyleCodeGenerator,
}

impl VirtualCodeGenerator {
    /// Create a new virtual code generator.
    #[inline]
    pub fn new() -> Self {
        Self {
            template_gen: TemplateCodeGenerator::new(),
            script_gen: ScriptCodeGenerator::new(),
            style_gen: StyleCodeGenerator::new(),
        }
    }

    /// Generate virtual documents from an SFC descriptor.
    ///
    /// Uses the provided arena allocator for temporary parsing data,
    /// minimizing heap allocations during generation.
    pub fn generate<'a>(
        &mut self,
        descriptor: &SfcDescriptor<'a>,
        base_uri: &str,
    ) -> VirtualDocuments {
        // Create arena for temporary parsing data
        let allocator = Bump::new();

        let mut docs = VirtualDocuments::new();

        // Generate template virtual code
        let mut template_expressions = Vec::new();
        if let Some(ref template) = descriptor.template {
            let template_content = template.content.as_ref();

            // Parse template with arena allocation
            let (ast, _errors) = vize_armature::parse(&allocator, template_content);
            template_expressions = extract_expressions(&ast);

            // Set block offset for source mapping
            self.template_gen
                .set_block_offset(template.loc.start as u32);

            // Generate virtual TypeScript
            let mut template_doc = self.template_gen.generate(&ast, template_content);
            template_doc.uri = cstr!("{base_uri}.__template.ts").to_string();

            docs.template = Some(template_doc);
        }

        // Generate script virtual code
        if let Some(ref script) = descriptor.script {
            let mut script_doc = self.script_gen.generate(script, false);
            script_doc.uri = cstr!("{base_uri}.__script.ts").to_string();
            docs.script = Some(script_doc);
        }

        // Generate script setup virtual code
        if let Some(ref script_setup) = descriptor.script_setup {
            let template_bindings =
                template_used_script_bindings(script_setup.content.as_ref(), &template_expressions);
            let mut script_doc =
                self.script_gen
                    .generate_with_exports(script_setup, true, &template_bindings);
            script_doc.uri = cstr!("{base_uri}.__script_setup.ts").to_string();
            docs.script_setup = Some(script_doc);
        }

        // Generate style virtual codes
        for (i, style) in descriptor.styles.iter().enumerate() {
            let mut style_doc = self.style_gen.generate(style, i);
            let ext = style.lang.as_ref().map(|l| l.as_ref()).unwrap_or("css");
            style_doc.uri = cstr!("{base_uri}.__style_{i}.{ext}").to_string();
            docs.styles.push(style_doc);
        }

        // Arena is dropped here, freeing all temporary allocations

        docs
    }

    /// Generate virtual documents with explicit allocator.
    ///
    /// Use this when you want to control the allocator lifetime,
    /// for example when processing multiple files in a batch.
    pub fn generate_with_allocator<'a, 'alloc>(
        &mut self,
        descriptor: &SfcDescriptor<'a>,
        base_uri: &str,
        allocator: &'alloc Bump,
    ) -> VirtualDocuments {
        let mut docs = VirtualDocuments::new();

        // Generate template virtual code
        let mut template_expressions = Vec::new();
        if let Some(ref template) = descriptor.template {
            let template_content = template.content.as_ref();

            // Parse template with provided allocator
            let (ast, _errors) = vize_armature::parse(allocator, template_content);
            template_expressions = extract_expressions(&ast);

            self.template_gen
                .set_block_offset(template.loc.start as u32);
            let mut template_doc = self.template_gen.generate(&ast, template_content);
            template_doc.uri = cstr!("{base_uri}.__template.ts").to_string();

            docs.template = Some(template_doc);
        }

        // Generate script virtual code
        if let Some(ref script) = descriptor.script {
            let mut script_doc = self.script_gen.generate(script, false);
            script_doc.uri = cstr!("{base_uri}.__script.ts").to_string();
            docs.script = Some(script_doc);
        }

        // Generate script setup virtual code
        if let Some(ref script_setup) = descriptor.script_setup {
            let template_bindings =
                template_used_script_bindings(script_setup.content.as_ref(), &template_expressions);
            let mut script_doc =
                self.script_gen
                    .generate_with_exports(script_setup, true, &template_bindings);
            script_doc.uri = cstr!("{base_uri}.__script_setup.ts").to_string();
            docs.script_setup = Some(script_doc);
        }

        // Generate style virtual codes
        for (i, style) in descriptor.styles.iter().enumerate() {
            let mut style_doc = self.style_gen.generate(style, i);
            let ext = style.lang.as_ref().map(|l| l.as_ref()).unwrap_or("css");
            style_doc.uri = cstr!("{base_uri}.__style_{i}.{ext}").to_string();
            docs.styles.push(style_doc);
        }

        docs
    }

    /// Quick generation for a single template string.
    ///
    /// Useful for testing and single-file scenarios.
    #[inline]
    pub fn generate_template_only(&mut self, template_content: &str) -> Option<VirtualDocument> {
        let allocator = Bump::new();
        let (ast, _) = vize_armature::parse(&allocator, template_content);

        let mut doc = self.template_gen.generate(&ast, template_content);
        doc.uri = "__inline.__template.ts".to_string();

        Some(doc)
    }
}

impl Default for VirtualCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn template_used_script_bindings(
    script_content: &str,
    expressions: &[TemplateExpression],
) -> Vec<String> {
    let script_bindings = extract_simple_bindings(script_content, true)
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    if script_bindings.is_empty() {
        return Vec::new();
    }

    let mut used = std::collections::BTreeSet::new();
    for expression in expressions {
        collect_expression_identifiers(&expression.text, &script_bindings, &mut used);
    }

    used.into_iter().collect()
}

fn collect_expression_identifiers(
    expression: &str,
    script_bindings: &std::collections::BTreeSet<String>,
    used: &mut std::collections::BTreeSet<String>,
) {
    let bytes = expression.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if !is_identifier_start(byte) {
            index += 1;
            continue;
        }

        let start = index;
        index += 1;
        while index < bytes.len() && is_identifier_continue(bytes[index]) {
            index += 1;
        }

        if is_property_access(expression, start) {
            continue;
        }

        let name = &expression[start..index];
        if !is_js_keyword(name) && script_bindings.contains(name) {
            used.insert(name.to_string());
        }
    }
}

#[inline]
fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$'
}

#[inline]
fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

fn is_property_access(expression: &str, start: usize) -> bool {
    expression
        .as_bytes()
        .get(..start)
        .unwrap_or_default()
        .iter()
        .rev()
        .find(|byte| !byte.is_ascii_whitespace())
        .is_some_and(|byte| *byte == b'.')
}

fn is_js_keyword(name: &str) -> bool {
    matches!(
        name,
        "as" | "async"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "from"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "let"
            | "new"
            | "null"
            | "return"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "undefined"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

/// Batch generator for processing multiple SFC files efficiently.
///
/// Reuses a single arena allocator across multiple files to minimize
/// allocation overhead.
pub struct BatchVirtualCodeGenerator {
    /// Underlying generator
    generator: VirtualCodeGenerator,
    /// Shared allocator for batch processing
    allocator: Bump,
}

impl BatchVirtualCodeGenerator {
    /// Create a new batch generator.
    #[inline]
    pub fn new() -> Self {
        Self {
            generator: VirtualCodeGenerator::new(),
            allocator: Bump::new(),
        }
    }

    /// Create with pre-allocated capacity.
    ///
    /// Use this when you know approximately how much memory will be needed.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            generator: VirtualCodeGenerator::new(),
            allocator: Bump::with_capacity(capacity),
        }
    }

    /// Generate virtual documents for a single file.
    ///
    /// The allocator is reused but reset between calls.
    pub fn generate<'a>(
        &mut self,
        descriptor: &SfcDescriptor<'a>,
        base_uri: &str,
    ) -> VirtualDocuments {
        // Reset allocator for new file
        self.allocator.reset();

        self.generator
            .generate_with_allocator(descriptor, base_uri, &self.allocator)
    }

    /// Process multiple files in batch.
    ///
    /// More efficient than calling generate() repeatedly as it
    /// minimizes allocator resets.
    pub fn generate_batch<'a>(
        &mut self,
        files: &[(&SfcDescriptor<'a>, &str)],
    ) -> Vec<VirtualDocuments> {
        files
            .iter()
            .map(|(descriptor, uri)| {
                self.allocator.reset();
                self.generator
                    .generate_with_allocator(descriptor, uri, &self.allocator)
            })
            .collect()
    }

    /// Get memory usage statistics.
    #[inline]
    pub fn allocated_bytes(&self) -> usize {
        self.allocator.allocated_bytes()
    }
}

impl Default for BatchVirtualCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about cursor position within an art variant template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtVariantInfo {
    /// Index of the variant in the art descriptor
    pub variant_index: usize,
    /// Byte offset where the variant template content starts in the art file
    pub template_start: usize,
    /// Byte offset where the variant template content ends in the art file
    pub template_end: usize,
    /// Cursor offset relative to the start of the variant template content
    pub relative_offset: usize,
}

/// A source-backed slice of an art `<script setup>` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtScriptChunk {
    /// Absolute byte offset where the chunk starts in the art file.
    pub source_start: usize,
    /// Absolute byte offset where the chunk ends in the art file.
    pub source_end: usize,
    /// Chunk source text.
    pub text: String,
}

/// Script setup decomposition for `.art.vue`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtScriptSetupParts {
    /// The inferred component tag from `defineArt(...)`, if present.
    pub component_name: Option<String>,
    /// Whether the setup body should be isolated per variant.
    pub isolate: bool,
    /// Imports that are safe to keep at module level for generated variant setup functions.
    pub shared_imports: Vec<ArtScriptChunk>,
    /// Top-level setup code used as variant-local state.
    pub isolated_body: Vec<ArtScriptChunk>,
}

impl ArtScriptSetupParts {
    /// Concatenate isolated body chunks.
    pub fn isolated_body_text(&self) -> String {
        let mut text = String::new();
        for chunk in &self.isolated_body {
            text.push_str(&chunk.text);
            if !text.ends_with('\n') {
                text.push('\n');
            }
        }
        text
    }
}

/// Where the cursor is within an art block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtCursorPosition {
    /// In `<art ...>` tag attributes
    ArtTag,
    /// In `<variant ...>` tag attributes (variant index)
    VariantTag(usize),
    /// Inside variant template content
    VariantTemplate(ArtVariantInfo),
    /// Between variants (art content area)
    ArtContent,
}

/// Helper to determine the virtual language from a block position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Template,
    Script,
    ScriptSetup,
    Style(usize),
    Art(ArtCursorPosition),
}

impl BlockType {
    /// Get the virtual language for this block type.
    #[inline]
    pub fn language(&self) -> VirtualLanguage {
        match self {
            BlockType::Template => VirtualLanguage::Template,
            BlockType::Script => VirtualLanguage::Script,
            BlockType::ScriptSetup => VirtualLanguage::ScriptSetup,
            BlockType::Style(_) => VirtualLanguage::Style,
            BlockType::Art(_) => VirtualLanguage::Template,
        }
    }
}

/// Find which block contains the given offset in an SFC.
pub fn find_block_at_offset(descriptor: &SfcDescriptor, offset: usize) -> Option<BlockType> {
    // Check template
    if let Some(ref template) = descriptor.template
        && offset >= template.loc.start
        && offset < template.loc.end
    {
        return Some(BlockType::Template);
    }

    // Check script
    if let Some(ref script) = descriptor.script
        && offset >= script.loc.start
        && offset < script.loc.end
    {
        return Some(BlockType::Script);
    }

    // Check script setup
    if let Some(ref script_setup) = descriptor.script_setup
        && offset >= script_setup.loc.start
        && offset < script_setup.loc.end
    {
        return Some(BlockType::ScriptSetup);
    }

    // Check styles
    for (i, style) in descriptor.styles.iter().enumerate() {
        if offset >= style.loc.start && offset < style.loc.end {
            return Some(BlockType::Style(i));
        }
    }

    // Check custom blocks (art, i18n, etc.)
    for custom in descriptor.custom_blocks.iter() {
        if custom.block_type == "art" && offset >= custom.loc.start && offset < custom.loc.end {
            return Some(BlockType::Art(ArtCursorPosition::ArtContent));
        }
    }

    None
}

/// Find which block contains the given offset in an art file (*.art.vue).
///
/// Uses `vize_musea::parse_art()` to determine cursor position within art variant templates.
pub fn find_art_block_at_offset(source: &str, offset: usize) -> Option<BlockType> {
    // First check SFC blocks (script, style)
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: Default::default(),
        ..Default::default()
    };

    if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(source, options) {
        // Check script/script_setup/style blocks
        if let Some(ref script) = descriptor.script
            && offset >= script.loc.start
            && offset < script.loc.end
        {
            return Some(BlockType::Script);
        }
        if let Some(ref script_setup) = descriptor.script_setup
            && offset >= script_setup.loc.start
            && offset < script_setup.loc.end
        {
            return Some(BlockType::ScriptSetup);
        }
        for (i, style) in descriptor.styles.iter().enumerate() {
            if offset >= style.loc.start && offset < style.loc.end {
                return Some(BlockType::Style(i));
            }
        }
    }

    // Parse as art file to determine variant position
    let allocator = vize_carton::Bump::new();
    let Ok(art_desc) =
        vize_musea::parse_art(&allocator, source, vize_musea::ArtParseOptions::default())
    else {
        return None;
    };

    for (i, variant) in art_desc.variants.iter().enumerate() {
        if let Some(ref loc) = variant.loc {
            let variant_start = loc.start as usize;
            let variant_end = loc.end as usize;

            if offset >= variant_start && offset < variant_end {
                let template_ptr = variant.template.as_ptr() as usize;
                let source_ptr = source.as_ptr() as usize;
                let trimmed_template_start = if variant.template.is_empty() {
                    find_variant_template_body_range(source, variant_start, variant_end)
                        .map(|(body_start, _)| body_start)
                        .unwrap_or(variant_start)
                } else {
                    template_ptr.saturating_sub(source_ptr)
                };
                let trimmed_template_end = trimmed_template_start + variant.template.len();
                let (body_start, body_end) =
                    find_variant_template_body_range(source, variant_start, variant_end)
                        .unwrap_or((trimmed_template_start, trimmed_template_end));

                if offset >= body_start && offset < body_end {
                    let relative_offset = if offset <= trimmed_template_start {
                        0
                    } else if offset >= trimmed_template_end {
                        variant.template.len()
                    } else {
                        offset - trimmed_template_start
                    };

                    return Some(BlockType::Art(ArtCursorPosition::VariantTemplate(
                        ArtVariantInfo {
                            variant_index: i,
                            template_start: trimmed_template_start,
                            template_end: trimmed_template_end,
                            relative_offset,
                        },
                    )));
                }

                return Some(BlockType::Art(ArtCursorPosition::VariantTag(i)));
            }
        }
    }

    Some(BlockType::Art(ArtCursorPosition::ArtContent))
}

fn find_variant_template_body_range(
    source: &str,
    variant_start: usize,
    variant_end: usize,
) -> Option<(usize, usize)> {
    if variant_start >= variant_end || variant_end > source.len() {
        return None;
    }

    let tag_end = source[variant_start..variant_end].find('>')? + variant_start;
    let body_start = tag_end + 1;
    let close_start = source[body_start..variant_end].rfind("</variant>")? + body_start;

    Some((body_start, close_start))
}

/// Split art `<script setup>` into compiler-macro metadata, shared imports, and
/// variant-local setup body.
pub fn analyze_art_script_setup(
    script: &str,
    source_start: usize,
    isolate: bool,
) -> ArtScriptSetupParts {
    let parsed = vize_croquis::script_parser::parse_script_setup(script);
    let component_name = parsed
        .macros
        .define_art()
        .map(|art| art.component_name.to_string());
    let define_art_range = parsed
        .macros
        .define_art_call()
        .map(|call| (call.start as usize, call.end as usize));
    let component_binding_range = component_name
        .as_ref()
        .and_then(|name| parsed.binding_spans.get(name.as_str()))
        .map(|(start, end)| (*start as usize, *end as usize));
    let statements = split_top_level_statements(script, source_start);
    let mut shared_imports = Vec::new();
    let mut isolated_body = Vec::new();

    for statement in statements {
        let relative_start = statement.source_start.saturating_sub(source_start);
        let relative_end = statement.source_end.saturating_sub(source_start);
        let trimmed = statement.text.trim();
        if trimmed.is_empty()
            || define_art_range
                .is_some_and(|range| ranges_overlap(range, (relative_start, relative_end)))
        {
            continue;
        }

        if trimmed.starts_with("import ") {
            if isolate
                && component_binding_range.is_some_and(|range| {
                    parsed.import_statements.iter().any(|import| {
                        let import_range = (import.start as usize, import.end as usize);
                        contains_range(import_range, range)
                            && ranges_overlap(import_range, (relative_start, relative_end))
                    })
                })
            {
                continue;
            }
            shared_imports.push(statement);
            continue;
        }

        isolated_body.push(statement);
    }

    ArtScriptSetupParts {
        component_name,
        isolate,
        shared_imports,
        isolated_body,
    }
}

/// Find the component tag inferred from `defineArt(...)`.
pub fn find_define_art_component_name(script: &str) -> Option<String> {
    vize_croquis::script_parser::parse_script_setup(script)
        .macros
        .define_art()
        .map(|art| art.component_name.to_string())
}

fn split_top_level_statements(script: &str, source_start: usize) -> Vec<ArtScriptChunk> {
    let mut statements = Vec::new();
    let mut start = 0usize;
    let mut state = StatementState::default();

    for (idx, ch) in script.char_indices() {
        state.accept(ch, script, idx);
        if state.is_boundary(ch) {
            let end = idx + ch.len_utf8();
            push_statement(&mut statements, script, source_start, start, end);
            start = end;
        }
    }

    push_statement(&mut statements, script, source_start, start, script.len());
    statements
}

fn push_statement(
    statements: &mut Vec<ArtScriptChunk>,
    script: &str,
    source_start: usize,
    start: usize,
    end: usize,
) {
    if start >= end {
        return;
    }
    let text = &script[start..end];
    if text.trim().is_empty() {
        return;
    }
    statements.push(ArtScriptChunk {
        source_start: source_start + start,
        source_end: source_start + end,
        text: text.to_string(),
    });
}

#[derive(Default)]
struct StatementState {
    brace_depth: i32,
    bracket_depth: i32,
    paren_depth: i32,
    quote: Option<char>,
}

impl StatementState {
    fn accept(&mut self, ch: char, source: &str, idx: usize) {
        if let Some(quote) = self.quote {
            if ch == quote && !is_escaped(source, idx) {
                self.quote = None;
            }
            return;
        }

        match ch {
            '"' | '\'' | '`' => self.quote = Some(ch),
            '{' => self.brace_depth += 1,
            '}' => self.brace_depth = self.brace_depth.saturating_sub(1),
            '[' => self.bracket_depth += 1,
            ']' => self.bracket_depth = self.bracket_depth.saturating_sub(1),
            '(' => self.paren_depth += 1,
            ')' => self.paren_depth = self.paren_depth.saturating_sub(1),
            _ => {}
        }
    }

    fn is_boundary(&self, ch: char) -> bool {
        self.quote.is_none()
            && self.brace_depth == 0
            && self.bracket_depth == 0
            && self.paren_depth == 0
            && (ch == ';' || ch == '\n')
    }
}

fn ranges_overlap(a: (usize, usize), b: (usize, usize)) -> bool {
    a.0 < b.1 && b.0 < a.1
}

fn contains_range(outer: (usize, usize), inner: (usize, usize)) -> bool {
    outer.0 <= inner.0 && inner.1 <= outer.1
}

fn is_escaped(source: &str, idx: usize) -> bool {
    let mut count = 0usize;
    let mut pos = idx;
    while pos > 0 && source.as_bytes()[pos - 1] == b'\\' {
        count += 1;
        pos -= 1;
    }
    count % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::{
        ArtCursorPosition, BatchVirtualCodeGenerator, BlockType, VirtualCodeGenerator,
        VirtualLanguage, find_art_block_at_offset, find_block_at_offset,
    };

    #[test]
    fn test_virtual_code_generator() {
        let source = r#"<template>
  <div>{{ message }}</div>
</template>

<script setup lang="ts">
const message = ref('hello')
</script>

<style scoped>
.container { color: red; }
</style>"#;

        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();

        let mut generator = VirtualCodeGenerator::new();
        let docs = generator.generate(&descriptor, "test.vue");

        assert!(docs.template.is_some());
        assert!(docs.script_setup.is_some());
        assert_eq!(docs.styles.len(), 1);

        // Check template virtual code
        let template = docs.template.unwrap();
        assert!(!template.source_map.is_empty());
        insta::assert_snapshot!(template.content.as_str());
    }

    #[test]
    fn test_script_setup_exports_template_used_bindings() {
        let source = r#"<script setup lang="ts">
const count = ref(0)
function handleClick() {
  count.value++
}
const double = computed(() => count.value * 2)
const unused = 1
</script>

<template>
  <button @click="handleClick">{{ count }}</button>
  <p>{{ double }}</p>
</template>"#;

        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
        let mut generator = VirtualCodeGenerator::new();
        let docs = generator.generate(&descriptor, "test.vue");
        let script_setup = docs.script_setup.unwrap();

        assert!(
            script_setup
                .content
                .contains("export { count, double, handleClick };")
        );
        assert!(
            !script_setup
                .content
                .contains("export { count, double, handleClick, unused };")
        );
    }

    #[test]
    fn test_batch_generator() {
        let source1 = "<template><div>{{ a }}</div></template>";
        let source2 = "<template><div>{{ b }}</div></template>";

        let desc1 = vize_atelier_sfc::parse_sfc(source1, Default::default()).unwrap();
        let desc2 = vize_atelier_sfc::parse_sfc(source2, Default::default()).unwrap();

        let mut batch = BatchVirtualCodeGenerator::new();
        let results = batch.generate_batch(&[(&desc1, "file1.vue"), (&desc2, "file2.vue")]);

        assert_eq!(results.len(), 2);
        assert!(results[0].template.is_some());
        assert!(results[1].template.is_some());
    }

    #[test]
    fn test_find_block_at_offset() {
        let source = r#"<template>
  <div>test</div>
</template>

<script setup>
const x = 1
</script>"#;

        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();

        // In template
        assert_eq!(
            find_block_at_offset(&descriptor, 15),
            Some(BlockType::Template)
        );

        // In script setup
        assert_eq!(
            find_block_at_offset(&descriptor, 60),
            Some(BlockType::ScriptSetup)
        );
    }

    #[test]
    fn test_find_block_at_offset_inline_art() {
        let source = r#"<template>
  <div>test</div>
</template>

<script setup>
const x = 1
</script>

<art title="Test" component="./Foo.vue">
  <variant name="Default" default>
    <Foo />
  </variant>
</art>"#;

        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();

        // Verify custom_blocks contains the art block
        assert_eq!(descriptor.custom_blocks.len(), 1);
        assert_eq!(descriptor.custom_blocks[0].block_type, "art");

        // Offset inside <art> content area
        let art_content_start = descriptor.custom_blocks[0].loc.start;
        assert_eq!(
            find_block_at_offset(&descriptor, art_content_start + 5),
            Some(BlockType::Art(ArtCursorPosition::ArtContent))
        );

        // In template - should still be Template
        assert_eq!(
            find_block_at_offset(&descriptor, 15),
            Some(BlockType::Template)
        );

        // Outside any block
        assert_eq!(find_block_at_offset(&descriptor, 0), None);
    }

    #[test]
    fn test_block_type_art_language() {
        assert_eq!(
            BlockType::Art(ArtCursorPosition::ArtContent).language(),
            VirtualLanguage::Template
        );
    }

    #[test]
    fn test_find_art_block_at_offset() {
        let source = r#"<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>Click me</Button>
  </variant>
</art>

<script setup lang="ts">
import Button from './Button.vue'
</script>"#;

        // In script setup
        let script_offset = source.find("import Button").unwrap();
        assert_eq!(
            find_art_block_at_offset(source, script_offset),
            Some(BlockType::ScriptSetup)
        );

        // In variant template content
        let template_offset = source.find("<Button>Click me</Button>").unwrap();
        let result = find_art_block_at_offset(source, template_offset);
        assert!(matches!(
            result,
            Some(BlockType::Art(ArtCursorPosition::VariantTemplate(_)))
        ));

        // In art content (between variants)
        let art_content_offset = source.find("\n  <variant").unwrap() + 1;
        // This offset is just before <variant, which is inside the <art> but before variant tag starts
        // It should be ArtContent
        assert!(matches!(
            find_art_block_at_offset(source, art_content_offset),
            Some(BlockType::Art(_))
        ));
    }

    #[test]
    fn test_find_art_block_at_offset_treats_variant_body_whitespace_as_template() {
        let source = r#"<art title="Button" component="./Button.vue">
  <variant name="Primary" default>

    <Button>Click me</Button>
  </variant>
</art>"#;

        let body_whitespace_offset = source.find("\n\n    <Button>").unwrap() + 1;
        let result = find_art_block_at_offset(source, body_whitespace_offset);

        let Some(BlockType::Art(ArtCursorPosition::VariantTemplate(info))) = result else {
            panic!("expected variant template, got {result:?}");
        };

        assert_eq!(info.relative_offset, 0);
        assert_eq!(info.template_start, source.find("<Button>").unwrap());
    }

    #[test]
    fn test_find_art_block_at_offset_treats_variant_body_as_template() {
        let source = r#"<script setup>
const count = ref(0)
</script>

<art title="Counter" component="./Counter.vue">
  <variant name="Interactive">
    <Counter :count="count" />
  </variant>
</art>"#;

        let offset = source.find("count = ref").unwrap();
        assert_eq!(
            find_art_block_at_offset(source, offset),
            Some(BlockType::ScriptSetup)
        );

        let template_offset = source.find(":count=\"count\"").unwrap();
        assert!(matches!(
            find_art_block_at_offset(source, template_offset),
            Some(BlockType::Art(ArtCursorPosition::VariantTemplate(_)))
        ));
    }
}
