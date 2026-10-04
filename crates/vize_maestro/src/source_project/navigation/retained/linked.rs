//! Original Descriptor and selected Component remain ordinary worker locals.
#![expect(
    clippy::disallowed_types,
    reason = "the worker owns one genuine snapshot Arc"
)]
use super::super::{
    NavigationRefusal, SourceSnapshot,
    profile::VueConfiguration,
    worker::{Command, Control, refused},
};
use std::sync::{Arc, mpsc::Receiver};
use vize_l0::Allocator;
use vize_l1::{container::Vue, markup::NativeTemplateComponent};
mod query;
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
    let descriptor = Vue.observe_descriptor(&arena, snapshot.source(), configuration.options());
    let Ok(admitted) = descriptor.admitted() else {
        return refused(receiver, &control, NavigationRefusal::TemplateNamesProducer);
    };
    let selected = match NativeTemplateComponent::parse_in(&arena, admitted) {
        Ok(Some(selected)) => selected,
        // This lexical profile requires an original selected template.
        _ => return refused(receiver, &control, NavigationRefusal::TemplateNamesProducer),
    };
    if control.retired() {
        return;
    }
    let carrier = selected.component().carrier();
    if !carrier.errors.is_empty() || !carrier.unsupported.is_empty() || carrier.authored.is_some() {
        return refused(receiver, &control, NavigationRefusal::TemplateNamesProducer);
    }
    let lines = vize_l0::line_index::LineBreaks::Lsp
        .line_starts(snapshot.source())
        .collect::<Vec<_>>();
    let query = query::TemplateNames {
        snapshot: &snapshot,
        selected: &selected,
        lines: &lines,
        #[cfg(test)]
        original: super::super::worker::linked::Inspection {
            descriptor: core::ptr::from_ref(&descriptor) as usize,
            selected: core::ptr::from_ref(&selected) as usize,
            component: core::ptr::from_ref(selected.component()) as usize,
            productions: control
                .sfc_productions
                .load(std::sync::atomic::Ordering::Acquire),
        },
    };
    serve::run(&query, receiver, &control);
    // All borrowed queries end first, then selected Component, Descriptor, arena,
    // original snapshot and finally the live-slot guard in the outer worker.
    drop(selected);
    drop(descriptor);
}
