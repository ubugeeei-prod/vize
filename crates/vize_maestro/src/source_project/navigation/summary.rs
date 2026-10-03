//! Project only validated same-File bindings into private response rows.
#![expect(
    clippy::disallowed_types,
    reason = "consumer summaries retain the actual host snapshot Arc"
)]

use std::sync::Arc;
use tower_lsp::lsp_types::{Location, Position};
use vize_l0::{Allocator, FxHashMap, SourceRoot, Span};
use vize_l1::embed::{
    EmbedSource, Lang,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l2::{
    file::ReferenceTarget,
    lang::js::{FileProducer, ProgramInput, ProgramScope},
};

use super::{NavigationRefusal, SourceSnapshot, coordinates};

pub(super) struct NavigationSummary {
    snapshot: Arc<SourceSnapshot>,
    declarations: Vec<Span>,
    references: Vec<(Span, usize)>,
    lines: Vec<usize>,
}

impl NavigationSummary {
    pub(super) fn build(snapshot: Arc<SourceSnapshot>) -> Result<Self, NavigationRefusal> {
        let lang = match snapshot.language_id() {
            "javascript" => Lang::Js,
            "typescript" => Lang::Ts,
            _ => return Err(NavigationRefusal::Language),
        };
        let arena = Allocator::default();
        let root = SourceRoot::new(snapshot.source()).map_err(|_| NavigationRefusal::Projection)?;
        let block = root.whole_block();
        let input = EmbedSource::authored(snapshot.source(), block.span())
            .map_err(|_| NavigationRefusal::Projection)?;
        let syntax = parse_program_once(&arena, input, ProgramOptions::module(lang));
        let admitted = syntax.admitted_program().ok_or(NavigationRefusal::Syntax)?;
        let program =
            ProgramInput::checked(admitted, block, 0).map_err(|_| NavigationRefusal::Projection)?;
        let mut producer = FileProducer::new(&arena, snapshot.source())
            .map_err(|_| NavigationRefusal::Projection)?;
        producer
            .program(program, ProgramScope::Module)
            .map_err(|_| NavigationRefusal::Projection)?;
        let file = producer.finish().map_err(|refused| {
            let mut issues = refused.issues().to_vec();
            issues.extend(refused.interrupted_programs());
            NavigationRefusal::Producer(issues)
        })?;
        // Structural artifact construction also retains incomplete observations.
        // Require the actual script-family completion before querying bindings;
        // this condition grants no Vue or product admission.
        if !file.is_complete() {
            let mut issues = file.issues().to_vec();
            issues.extend(file.interrupted_programs());
            return Err(NavigationRefusal::Producer(issues));
        }
        let bindings = file.bindings().collect::<Vec<_>>();
        let indices = bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| (binding.id(), index))
            .collect::<FxHashMap<_, _>>();
        let lines = vize_l0::line_index::LineBreaks::Lsp
            .line_starts(snapshot.source())
            .collect::<Vec<_>>();
        let declarations = bindings
            .iter()
            .map(|binding| {
                let declaration = binding.declaration().ok_or(NavigationRefusal::Projection)?;
                coordinates::range(snapshot.source(), &lines, declaration.span)?;
                Ok(declaration.span)
            })
            .collect::<Result<Vec<_>, NavigationRefusal>>()?;
        let references = file
            .references()
            .iter()
            .map(|reference| {
                let ReferenceTarget::Resolved(id) = reference.target else {
                    return Err(NavigationRefusal::Projection);
                };
                let binding = file.binding(id).ok_or(NavigationRefusal::Projection)?;
                let index = *indices
                    .get(&binding.id())
                    .ok_or(NavigationRefusal::Projection)?;
                if !bindings
                    .get(index)
                    .is_some_and(|candidate| candidate.same_owner(binding))
                {
                    return Err(NavigationRefusal::Projection);
                }
                coordinates::range(snapshot.source(), &lines, reference.span)?;
                Ok((reference.span, index))
            })
            .collect::<Result<Vec<_>, NavigationRefusal>>()?;
        // No Program, raw BindingId or File capability escapes its real owner.
        drop(bindings);
        drop(file);
        drop(syntax);
        Ok(Self {
            snapshot,
            declarations,
            references,
            lines,
        })
    }

    #[cfg(test)]
    pub(super) fn belongs_to(&self, snapshot: &Arc<SourceSnapshot>) -> bool {
        Arc::ptr_eq(&self.snapshot, snapshot)
    }

    fn binding_at(&self, position: Position) -> Result<Option<usize>, NavigationRefusal> {
        let offset = coordinates::offset(self.snapshot.source(), &self.lines, position)?;
        let contains = |span: Span| span.start as usize <= offset && offset < span.end as usize;
        Ok(self
            .declarations
            .iter()
            .position(|span| contains(*span))
            .or_else(|| {
                self.references
                    .iter()
                    .find(|(span, _)| contains(*span))
                    .map(|(_, index)| *index)
            }))
    }

    fn location(&self, span: Span) -> Result<Location, NavigationRefusal> {
        Ok(Location::new(
            self.snapshot.uri().clone(),
            coordinates::range(self.snapshot.source(), &self.lines, span)?,
        ))
    }

    pub(super) fn definition(
        &self,
        position: Position,
    ) -> Result<Option<Location>, NavigationRefusal> {
        self.binding_at(position)?
            .map(|index| {
                self.location(
                    *self
                        .declarations
                        .get(index)
                        .ok_or(NavigationRefusal::Projection)?,
                )
            })
            .transpose()
    }

    pub(super) fn references(
        &self,
        position: Position,
        include_declaration: bool,
    ) -> Result<Vec<Location>, NavigationRefusal> {
        let Some(index) = self.binding_at(position)? else {
            return Ok(Vec::new());
        };
        let mut spans = self
            .references
            .iter()
            .filter(|(_, target)| *target == index)
            .map(|(span, _)| *span)
            .collect::<Vec<_>>();
        if include_declaration {
            spans.push(
                *self
                    .declarations
                    .get(index)
                    .ok_or(NavigationRefusal::Projection)?,
            );
        }
        spans.sort_unstable_by_key(|span| (span.start, span.end));
        spans.into_iter().map(|span| self.location(span)).collect()
    }
}
