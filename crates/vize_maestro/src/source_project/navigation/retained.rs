//! Original native owners remain ordinary locals for the entire mailbox loop.
#![expect(
    clippy::disallowed_types,
    reason = "the worker owns the original snapshot Arc"
)]

use std::sync::{Arc, mpsc::Receiver};

use tower_lsp::lsp_types::{Location, Position};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::{
    EmbedSource,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l2::{
    file::{BindingRef, FileArtifact, ReferenceTarget},
    lang::js::{FileProducer, ProgramInput, ProgramScope},
};

pub(super) mod vue;

use super::{
    NavigationRefusal, SourceSnapshot, coordinates,
    worker::{Command, Control, refused},
};

pub(super) fn run(
    snapshot: Arc<SourceSnapshot>,
    options: ProgramOptions,
    receiver: Receiver<Command>,
    control: Arc<Control>,
) {
    if control.retired() {
        return;
    }
    let arena = Allocator::default();
    let Ok(root) = SourceRoot::new(snapshot.source()) else {
        return refused(receiver, &control, NavigationRefusal::Projection);
    };
    let block = root.whole_block();
    let Ok(input) = EmbedSource::authored(snapshot.source(), block.span()) else {
        return refused(receiver, &control, NavigationRefusal::Projection);
    };
    // One authentic parser call per physical snapshot worker, never per request.
    #[cfg(test)]
    control
        .parses
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let syntax = parse_program_once(&arena, input, options);
    if control.retired() {
        return;
    }
    let Some(admitted) = syntax.admitted_program() else {
        return refused(receiver, &control, NavigationRefusal::Syntax);
    };
    #[cfg(test)]
    let original = {
        let program = admitted.program();
        Original {
            program: program.body.as_ptr() as usize,
            statements: program.body.len(),
            sfc: None,
        }
    };
    let Ok(program) = ProgramInput::checked(admitted, block, 0) else {
        return refused(receiver, &control, NavigationRefusal::Projection);
    };
    let Ok(mut producer) = FileProducer::new(&arena, snapshot.source()) else {
        return refused(receiver, &control, NavigationRefusal::Projection);
    };
    if producer.program(program, ProgramScope::Module).is_err() {
        return refused(receiver, &control, NavigationRefusal::Projection);
    }
    let file = match producer.finish() {
        Ok(file) => file,
        Err(original) => {
            let mut issues = original.issues().to_vec();
            issues.extend(original.interrupted_programs());
            return refused(receiver, &control, NavigationRefusal::Producer(issues));
        }
    };
    if !file.is_complete() {
        let mut issues = file.issues().to_vec();
        issues.extend(file.interrupted_programs());
        return refused(receiver, &control, NavigationRefusal::Producer(issues));
    }
    let lines = vize_l0::line_index::LineBreaks::Lsp
        .line_starts(snapshot.source())
        .collect::<Vec<_>>();
    let query = RetainedNavigation {
        snapshot: &snapshot,
        file: &file,
        lines: &lines,
        native: None,
        #[cfg(test)]
        original,
    };
    serve(&query, receiver, &control);
    // Explicit order: borrowed response work, then File, original syntax, arena.
    drop(file);
    drop(syntax);
}

fn serve(query: &RetainedNavigation<'_, '_>, receiver: Receiver<Command>, control: &Control) {
    while !control.retired() {
        let Ok(command) = receiver.recv() else { break };
        if control.retired() {
            break;
        }
        match command {
            Command::Definition(position, reply) => {
                if !reply.is_canceled() {
                    #[cfg(test)]
                    control
                        .queries
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let _ = reply.send(query.definition(position));
                }
            }
            Command::References(position, include_declaration, reply) => {
                if !reply.is_canceled() {
                    #[cfg(test)]
                    control
                        .queries
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let _ = reply.send(query.references(position, include_declaration));
                }
            }
            Command::Stop => break,
            #[cfg(test)]
            Command::Inspect(position, reply) => {
                if !reply.is_canceled() {
                    let result = query.offset(position).and_then(|offset| {
                        let binding = query.binding(offset)?;
                        Ok(super::worker::Inspection {
                            file: core::ptr::from_ref(query.file) as usize,
                            program: query.original.program,
                            statements: query.original.statements,
                            declaration: binding
                                .and_then(|row| row.declaration())
                                .map(|row| row.span),
                            parses: control.parses.load(std::sync::atomic::Ordering::Relaxed),
                            sfc: query.original.sfc.clone(),
                        })
                    });
                    let _ = reply.send(result);
                }
            }
            #[cfg(test)]
            Command::Pause(entered, resumed) => {
                let _ = entered.send(());
                let _ = resumed.recv();
            }
            #[cfg(test)]
            Command::Panic => panic!("actual native worker unwind"),
        }
    }
}

#[cfg(test)]
struct Original {
    program: usize,
    statements: usize,
    sfc: Option<super::worker::SfcInspection>,
}

struct RetainedNavigation<'file, 'arena> {
    snapshot: &'file SourceSnapshot,
    file: &'file FileArtifact<'arena>,
    lines: &'file [usize],
    native: Option<vize_l1_to_l2::native_file::NativeSfc<'file, 'arena>>,
    #[cfg(test)]
    original: Original,
}

impl<'file, 'arena> RetainedNavigation<'file, 'arena> {
    fn offset(&self, position: Position) -> Result<u32, NavigationRefusal> {
        let offset = coordinates::offset(self.snapshot.source(), self.lines, position)?;
        u32::try_from(offset).map_err(|_| NavigationRefusal::Position)
    }

    fn location(&self, span: Span) -> Result<Location, NavigationRefusal> {
        Ok(Location::new(
            self.snapshot.uri().clone(),
            coordinates::range(self.snapshot.source(), self.lines, span)?,
        ))
    }

    fn binding(&self, offset: u32) -> Result<Option<BindingRef<'file, 'arena>>, NavigationRefusal> {
        if let Some(native) = &self.native {
            return native
                .binding_at_offset(offset)
                .map_err(NavigationRefusal::NativeQuery);
        }
        self.file
            .binding_at_offset(offset)
            .map_err(NavigationRefusal::Query)
    }

    fn definition(&self, position: Position) -> Result<Option<Location>, NavigationRefusal> {
        self.binding(self.offset(position)?)?
            .map(|binding| {
                let declaration = binding.declaration().ok_or(NavigationRefusal::Projection)?;
                self.location(declaration.span)
            })
            .transpose()
    }

    fn references(
        &self,
        position: Position,
        include_declaration: bool,
    ) -> Result<Vec<Location>, NavigationRefusal> {
        let Some(binding) = self.binding(self.offset(position)?)? else {
            return Ok(Vec::new());
        };
        let mut spans = Vec::new();
        if let Some(native) = &self.native {
            native
                .for_each_reference_to(binding, |reference| spans.push(reference.span()))
                .map_err(NavigationRefusal::NativeQuery)?;
        } else {
            spans.extend(
                self.file
                    .references()
                    .iter()
                    .filter(|reference| reference.target == ReferenceTarget::Resolved(binding.id()))
                    .map(|reference| reference.span),
            );
        }
        if include_declaration {
            spans.push(
                binding
                    .declaration()
                    .ok_or(NavigationRefusal::Projection)?
                    .span,
            );
        }
        spans.sort_unstable_by_key(|span| (span.start, span.end));
        spans.into_iter().map(|span| self.location(span)).collect()
    }
}
