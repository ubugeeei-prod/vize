//! Authored regions that an embedded-language frontend owns in a script.
//! The original parser observation supplies both the source and the syntax.

use super::syntax::{ProgramGoal, ProgramOptions};
use alloc::vec::Vec;
use oxc_ast::ast::{JSXElement, JSXFragment};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::AdmittedProgram;
use vize_l0::{Lang, Span};

/// Outermost replacement regions and the starts of all nested surface nodes.
/// A host range containing such a start cannot be copied as ordinary script.
#[derive(Debug)]
pub struct ModuleRegions<'a> {
    roots: Vec<Span>,
    surface: ModuleSurface<'a>,
}

#[derive(Debug)]
pub struct ModuleSurface<'a> {
    source: &'a str,
    host: ProgramOptions,
    end: u32,
    starts: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionError {
    SourceTooLarge,
    SourceMismatch,
    Range,
}

impl<'a> ModuleRegions<'a> {
    /// Capture the parse profile and original markup sites without reparsing.
    pub fn from_program(input: &AdmittedProgram<'_, 'a>) -> Result<Self, RegionError> {
        let source = input.source();
        let program = input.program();
        // An unambiguous input has one resolved goal after the parser finishes.
        let profile = program.source_type;
        if !core::ptr::eq(source, program.source_text) {
            return Err(RegionError::SourceMismatch);
        }
        let end = u32::try_from(source.len()).map_err(|_| RegionError::SourceTooLarge)?;
        let mut sites = Sites::default();
        if profile.is_jsx() {
            sites.visit_program(program);
        }
        let mut previous = 0;
        for span in &sites.roots {
            if span.start < previous
                || span.is_empty()
                || source.get(span.start as usize..span.end as usize).is_none()
            {
                return Err(RegionError::Range);
            }
            previous = span.end;
        }
        sites.starts.sort_unstable();
        Ok(Self {
            roots: sites.roots,
            surface: ModuleSurface {
                source,
                end,
                host: ProgramOptions {
                    lang: if profile.is_typescript() {
                        Lang::Ts
                    } else {
                        Lang::Js
                    },
                    jsx: profile.is_jsx(),
                    goal: if profile.is_module() {
                        ProgramGoal::Module
                    } else {
                        ProgramGoal::Script
                    },
                },
                starts: sites.starts,
            },
        })
    }

    #[must_use]
    pub fn roots(&self) -> &[Span] {
        &self.roots
    }
    #[must_use]
    pub fn surface(&self) -> &ModuleSurface<'a> {
        &self.surface
    }
    /// Drop the temporary root spans after a lowering has checked its edits.
    #[must_use]
    pub fn into_surface(self) -> ModuleSurface<'a> {
        self.surface
    }
}

impl<'a> ModuleSurface<'a> {
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.source
    }
    #[must_use]
    pub fn end(&self) -> u32 {
        self.end
    }
    #[must_use]
    pub fn host(&self) -> ProgramOptions {
        self.host
    }
    /// Sorted positions including nodes nested inside host expressions.
    #[must_use]
    pub fn protected_starts(&self) -> &[u32] {
        &self.starts
    }
}

#[derive(Default)]
struct Sites {
    roots: Vec<Span>,
    starts: Vec<u32>,
    depth: usize,
}

impl Sites {
    fn enter(&mut self, span: oxc_span::Span) {
        if self.depth == 0 {
            self.roots.push(Span::new(span.start, span.end));
        }
        self.starts.push(span.start);
        self.depth += 1;
    }
}
impl<'a> Visit<'a> for Sites {
    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        self.enter(element.span);
        walk::walk_jsx_element(self, element);
        self.depth -= 1;
    }
    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) {
        self.enter(fragment.span);
        walk::walk_jsx_fragment(self, fragment);
        self.depth -= 1;
    }
}
