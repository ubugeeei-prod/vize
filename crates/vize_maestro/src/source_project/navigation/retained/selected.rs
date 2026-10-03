//! Once-selected whole SFC, original File and body tables remain worker locals.
#![expect(
    clippy::disallowed_types,
    reason = "the original immutable host snapshot is an Arc"
)]

use std::sync::{Arc, mpsc::Receiver};
use tower_lsp::lsp_types::{Location, Position};
use vize_l0::{Allocator, Span};
use vize_l1_to_l2::native_file::lower_selected_sfc_native;
use vize_l2::file::{FileArtifact, TemplateSymbolRef};

use super::super::{
    NavigationRefusal, SourceSnapshot, coordinates,
    profile::VueConfiguration,
    worker::{Command, Control, refused},
};

mod highlights;
mod serve;

pub(in crate::source_project::navigation) fn run(
    snapshot: Arc<SourceSnapshot>,
    configuration: VueConfiguration,
    receiver: Receiver<Command>,
    control: Arc<Control>,
) {
    if control.retired() {
        return;
    }
    if !configuration.supported() {
        return refused(receiver, &control, NavigationRefusal::Configuration);
    }
    let arena = Allocator::default();
    #[cfg(test)]
    control
        .sfc_productions
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let observation = lower_selected_sfc_native(&arena, snapshot.source(), configuration.options());
    if control.retired() {
        return;
    }
    {
        let Some(native) = observation.admitted() else {
            return refused(
                receiver,
                &control,
                NavigationRefusal::SelectedSfcProducer(observation.issues().to_vec()),
            );
        };
        let template = native.into_template_view();
        let Some(file) = template.file() else {
            return refused(receiver, &control, NavigationRefusal::Projection);
        };
        let lines = vize_l0::line_index::LineBreaks::Lsp
            .line_starts(snapshot.source())
            .collect::<Vec<_>>();
        let query = SelectedNavigation {
            snapshot: &snapshot,
            file,
            lines: &lines,
            #[cfg(test)]
            original: super::super::worker::selected::Inspection {
                file: core::ptr::from_ref(file) as usize,
                observation: core::ptr::from_ref(&observation) as usize,
                descriptor: core::ptr::from_ref(observation.descriptor()) as usize,
                selected: core::ptr::from_ref(observation.template().unwrap().selected()) as usize,
                handler_body: None,
                productions: control
                    .sfc_productions
                    .load(std::sync::atomic::Ordering::Relaxed),
            },
        };
        serve::run(&query, receiver, &control);
    }
    // The consumed view/query end before the actual whole original owner;
    // observation drops before arena, which drops before snapshot/slot release.
    drop(observation);
}

struct SelectedNavigation<'f, 'a> {
    snapshot: &'f SourceSnapshot,
    file: &'f FileArtifact<'a>,
    lines: &'f [usize],
    #[cfg(test)]
    original: super::super::worker::selected::Inspection,
}

impl<'f, 'a> SelectedNavigation<'f, 'a> {
    fn symbol(
        &self,
        position: Position,
    ) -> Result<Option<TemplateSymbolRef<'f, 'a>>, NavigationRefusal> {
        let offset = coordinates::offset(self.snapshot.source(), self.lines, position)?;
        let offset = u32::try_from(offset).map_err(|_| NavigationRefusal::Position)?;
        self.file
            .template_symbol_at_offset(offset)
            .map(|site| site.map(|site| site.symbol()))
            .map_err(NavigationRefusal::TemplateQuery)
    }

    fn location(&self, span: Span) -> Result<Location, NavigationRefusal> {
        Ok(Location::new(
            self.snapshot.uri().clone(),
            coordinates::range(self.snapshot.source(), self.lines, span)?,
        ))
    }

    fn declarations(
        &self,
        symbol: TemplateSymbolRef<'f, 'a>,
    ) -> Result<Vec<Span>, NavigationRefusal> {
        let spans = match symbol {
            TemplateSymbolRef::HandlerLocal(local) => {
                let resolution = local
                    .handler()
                    .resolution()
                    .ok_or(NavigationRefusal::Projection)?;
                local
                    .declarations()
                    .map(|row| {
                        resolution
                            .authored_span(row.span)
                            .map_err(|_| NavigationRefusal::Projection)
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }
            TemplateSymbolRef::File(binding) => {
                let declaration = binding
                    .template_declaration()
                    .ok_or(NavigationRefusal::Projection)?;
                let original = declaration
                    .declaration()
                    .original()
                    .ok_or(NavigationRefusal::Projection)?;
                vec![original.authored_span()]
            }
        };
        Ok(spans)
    }

    fn definitions(&self, position: Position) -> Result<Vec<Location>, NavigationRefusal> {
        let Some(symbol) = self.symbol(position)? else {
            return Ok(Vec::new());
        };
        self.locations(self.declarations(symbol)?)
    }

    fn references(
        &self,
        position: Position,
        include: bool,
    ) -> Result<Vec<Location>, NavigationRefusal> {
        let Some(symbol) = self.symbol(position)? else {
            return Ok(Vec::new());
        };
        let mut spans = Vec::new();
        // A provider refusal invalidates every partial callback span before
        // any location/publication is constructed.
        self.file
            .for_each_template_reference_to(symbol, |site| spans.push(site.span()))
            .map_err(NavigationRefusal::TemplateQuery)?;
        if include {
            spans.extend(self.declarations(symbol)?);
        }
        self.locations(spans)
    }

    fn locations(&self, mut spans: Vec<Span>) -> Result<Vec<Location>, NavigationRefusal> {
        spans.sort_unstable_by_key(|span| (span.start, span.end));
        spans.into_iter().map(|span| self.location(span)).collect()
    }
}
