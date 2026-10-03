//! Original descriptor, scripts, Component and embeds share one local lifetime.
#![expect(
    clippy::disallowed_types,
    reason = "the worker owns the original immutable host snapshot Arc"
)]

use std::sync::{Arc, mpsc::Receiver};
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::lower_sfc_native;

use super::{RetainedNavigation, serve};
use crate::source_project::navigation::{
    NavigationRefusal, SourceSnapshot,
    profile::VueConfiguration,
    worker::{Command, Control, refused},
};

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
    // One real producer invocation, which observes the original descriptor and
    // parses each selected script/embed once. This is not a one-parser counter.
    #[cfg(test)]
    control
        .sfc_productions
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let observation = lower_sfc_native(&arena, snapshot.source(), configuration.options());
    if control.retired() {
        return;
    }
    let Some(native) = observation.admitted() else {
        // The entire original refusal owner remains alive through this loop.
        return refused(
            receiver,
            &control,
            NavigationRefusal::SfcProducer(observation.issues().to_vec()),
        );
    };
    let file = native.file().file();
    let lines = vize_l0::line_index::LineBreaks::Lsp
        .line_starts(snapshot.source())
        .collect::<Vec<_>>();
    let query = RetainedNavigation {
        snapshot: &snapshot,
        file,
        lines: &lines,
        template: observation.template(),
        #[cfg(test)]
        original: {
            let programs = observation
                .scripts()
                .iter()
                .map(|script| {
                    let original = script.syntax().unwrap().admitted_program().unwrap();
                    let program = original.program();
                    (
                        program.body.as_ptr() as usize,
                        program.body.len(),
                        script.block().span(),
                    )
                })
                .collect::<Vec<_>>();
            let expressions = observation
                .template()
                .into_iter()
                .flat_map(|template| template.embeds())
                .map(|embed| {
                    (
                        core::ptr::from_ref(embed.syntax.expression().unwrap()) as usize,
                        embed.syntax.source().span(),
                    )
                })
                .collect();
            super::Original {
                program: 0,
                statements: 0,
                sfc: Some(super::super::worker::SfcInspection {
                    observation: core::ptr::from_ref(&observation) as usize,
                    descriptor: core::ptr::from_ref(observation.descriptor()) as usize,
                    programs,
                    expressions,
                    productions: control
                        .sfc_productions
                        .load(std::sync::atomic::Ordering::Relaxed),
                }),
            }
        },
    };
    serve(&query, receiver, &control);
    // Borrowed native view/query end before observation; arena and snapshot
    // remain stack owners until all original SFC owners have dropped.
    drop(observation);
}
