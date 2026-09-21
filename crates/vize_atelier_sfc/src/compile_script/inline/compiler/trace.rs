//! Source-map provenance through the inline `<script setup>` compiler
//! (Davinci P3-9).
//!
//! Every stage below the sectioning keeps its text in `.vue` offsets: the
//! sections are composed with the block's provenance as soon as they are cut,
//! so emission only appends runs at the output position it writes to.

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

use crate::module_map::Runs;

use super::super::super::statement_sections::{SectionTrace, extract_script_sections_traced};

type Sections = (Vec<String>, Vec<String>, Vec<String>);

/// Provenance of one inline compile, in `.vue` offsets.
#[derive(Debug, Default)]
pub(crate) struct SetupTrace {
    /// The compiled `<script setup>` content → `.vue`. Empty when the content
    /// was rewritten before compiling, so nothing below it is attributed.
    pub(crate) content: Runs,
    /// The extracted normal `<script>` content → `.vue`, when it is present.
    pub(crate) normal: Option<Runs>,
    /// The compiled module → `.vue`: the result of the compile.
    pub(crate) output: Runs,
    /// Per user import: its text's provenance and its statement's start.
    pub(super) imports: Vec<(Runs, Option<usize>)>,
    pub(super) ts_declarations: Vec<Runs>,
    /// `setup_lines.join("\n")` → `.vue`.
    pub(super) setup_code: Runs,
    /// Per emitted setup-body segment, parallel to the segments.
    pub(super) body: Vec<Runs>,
    /// Content span of each dropped macro statement and its `.vue` start.
    macros: Vec<(usize, usize, Option<usize>)>,
}

impl SetupTrace {
    pub(crate) fn new(content: Runs, normal: Option<Runs>) -> Self {
        Self {
            content,
            normal,
            ..Self::default()
        }
    }

    /// Record the sections' provenance, composed into `.vue` offsets.
    pub(super) fn record_sections(&mut self, sections: SectionTrace, setup_lines: &[String]) {
        let content = &self.content;
        self.imports = sections
            .imports
            .into_iter()
            .map(|(runs, statement)| (runs.compose(content), content.lookup(statement)))
            .collect();
        self.ts_declarations = sections
            .ts_declarations
            .iter()
            .map(|runs| runs.compose(content))
            .collect();
        let mut offset = 0;
        for (index, (line, runs)) in setup_lines.iter().zip(&sections.setup_lines).enumerate() {
            offset += usize::from(index > 0);
            self.setup_code.append(offset, &runs.compose(content));
            offset += line.len();
        }
        self.macros = sections
            .macros
            .into_iter()
            .map(|(start, end)| (start, end, content.lookup(start)))
            .collect();
    }

    /// The `.vue` start of the macro statement containing content offset `at`.
    fn macro_statement(&self, at: usize) -> Option<usize> {
        self.macros
            .iter()
            .find(|(start, end, _)| (*start..*end).contains(&at))
            .and_then(|&(_, _, origin)| origin)
    }
}

/// Emission-side view of an optional trace: every method is a no-op when the
/// compile did not ask for a map.
pub(super) struct Tracer<'a>(pub(super) Option<&'a mut SetupTrace>);

impl Tracer<'_> {
    /// Text written at output offset `at` was copied as `runs` describe.
    pub(super) fn copy(&mut self, at: usize, runs: Option<&Runs>) {
        if let (Some(trace), Some(runs)) = (self.0.as_deref_mut(), runs) {
            trace.output.append(at, runs);
        }
    }

    /// Text written at `at` was synthesized from the macro call at content
    /// offset `call`: anchor it at that macro's statement.
    pub(super) fn macro_binding(&mut self, at: usize, call: usize) {
        if let Some(trace) = self.0.as_deref_mut()
            && let Some(origin) = trace.macro_statement(call)
        {
            trace.output.point(at, origin);
        }
    }

    /// Text written at `at` was rebuilt from the construct at `.vue` `origin`.
    pub(super) fn point(&mut self, at: usize, origin: Option<usize>) {
        if let (Some(trace), Some(origin)) = (self.0.as_deref_mut(), origin) {
            trace.output.point(at, origin);
        }
    }

    pub(super) fn trace(&mut self) -> Option<&mut SetupTrace> {
        self.0.as_deref_mut()
    }
}

/// The sections of `content` with their provenance, or `None` when the
/// AST-based extraction does not apply and the caller must fall back to the
/// untraced line scanner.
pub(super) fn traced_sections(
    content: &str,
    is_ts: bool,
    program: Option<&Program<'_>>,
    preserve_runtime_erased_macros: bool,
) -> Option<(Sections, SectionTrace)> {
    let mut trace = SectionTrace::default();
    let sections = match program {
        Some(program) => extract_script_sections_traced(
            program,
            content,
            is_ts,
            preserve_runtime_erased_macros,
            &mut trace,
        ),
        None => {
            let allocator = Allocator::default();
            let source_type = SourceType::from_path("script.ts").unwrap_or_default();
            let ret = Parser::new(&allocator, content, source_type).parse();
            if ret.panicked {
                return None;
            }
            extract_script_sections_traced(
                &ret.program,
                content,
                is_ts,
                preserve_runtime_erased_macros,
                &mut trace,
            )
        }
    }?;
    Some((sections, trace))
}
