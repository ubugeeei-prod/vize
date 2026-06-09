//! Decomposition of art (`*.art.vue`) `<script setup>` blocks into compiler-macro
//! metadata, shared imports, and variant-local setup bodies.

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
