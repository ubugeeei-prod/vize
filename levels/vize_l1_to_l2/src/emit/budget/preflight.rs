//! Borrowed surface validation at the actual parse boundary.

use super::{DomEmitBudget, EmitInvocation, emit_dom_source_with_options_and_observer_captured};
use crate::{DomEmit, DomEmitOptions, EmitError, LegacyCaps};
use vize_l0::{
    Allocator,
    dump::capture::CaptureSink,
    pass::{BudgetObserver, NoObserver},
};
use vize_l1::markup::{SurfaceError, SurfaceTree};

/// Invocation-local validation owned by the consumer, not the compiler stage.
/// A false result stops before lowering; diagnostics stay with that consumer.
/// None is reserved for a consumer's actual compatibility parse entry.
/// This does not select the emitter used after that parse.
#[doc(hidden)]
pub trait SurfacePreflight {
    fn check(&mut self, surface: Option<(&SurfaceTree<'_>, &[SurfaceError])>) -> bool;
}

/// Reuse the exact parsed surface; ordinary entry points install no hook.
/// NoObserver remains the pass observer with traversal accounting disabled.
#[doc(hidden)]
#[expect(
    clippy::too_many_arguments,
    reason = "existing source inputs and invocation-local validation"
)]
pub fn emit_dom_source_with_preflight<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    caps: LegacyCaps,
    options: &DomEmitOptions<'_>,
    strict_slot_params: bool,
    no_slotted: bool,
    preflight: &mut dyn SurfacePreflight,
    observe: bool,
    capture: &mut C,
) -> Result<(DomEmit, Option<DomEmitBudget>), EmitError> {
    emit_dom_source_with_preflight_and_environment(
        allocator,
        source,
        caps,
        options,
        strict_slot_params,
        no_slotted,
        preflight,
        observe,
        true,
        capture,
    )
}

/// Reuse the exact parsed surface with the SFC build environment.
#[doc(hidden)]
#[expect(clippy::too_many_arguments, reason = "SFC emission inputs")]
pub fn emit_dom_source_with_preflight_and_environment<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    caps: LegacyCaps,
    options: &DomEmitOptions<'_>,
    strict_slot_params: bool,
    no_slotted: bool,
    preflight: &mut dyn SurfacePreflight,
    observe: bool,
    development: bool,
    capture: &mut C,
) -> Result<(DomEmit, Option<DomEmitBudget>), EmitError> {
    let invocation = EmitInvocation {
        options,
        strict_slot_params,
        no_slotted,
        development,
        preflight: Some(preflight),
    };
    if observe {
        let mut transform = BudgetObserver::new();
        let observed = emit_dom_source_with_options_and_observer_captured(
            allocator,
            source,
            caps,
            invocation,
            &mut transform,
            capture,
        )?;
        Ok((
            observed.emit,
            Some(DomEmitBudget {
                transform,
                emit_walks: 1,
                emit_visits: observed.emit_visits,
            }),
        ))
    } else {
        let observed = emit_dom_source_with_options_and_observer_captured(
            allocator,
            source,
            caps,
            invocation,
            &mut NoObserver,
            capture,
        )?;
        Ok((observed.emit, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_l0::dump::capture::{NoCapture, StageCapture};

    struct Guard {
        calls: usize,
        allow: bool,
    }
    impl SurfacePreflight for Guard {
        fn check(&mut self, surface: Option<(&SurfaceTree<'_>, &[SurfaceError])>) -> bool {
            let (tree, _) = surface.expect("only the actual native parse boundary");
            assert_eq!(tree.source, "<div id='x'>text</div>");
            self.calls += 1;
            self.allow
        }
    }

    #[test]
    fn actual_parse_hook_runs_once_and_preserves_complete_emission_and_capture() {
        let arena = Allocator::default();
        let source = "<div id='x'>text</div>";
        let options = DomEmitOptions::DEFAULT;
        let expected =
            crate::emit_dom_source_sfc_with_options(&arena, source, LegacyCaps::VUE3, &options)
                .unwrap();
        for observe in [false, true] {
            let mut guard = Guard {
                calls: 0,
                allow: true,
            };
            let result = emit_dom_source_with_preflight(
                &arena,
                source,
                LegacyCaps::VUE3,
                &options,
                true,
                false,
                &mut guard,
                observe,
                &mut NoCapture,
            )
            .unwrap();
            assert_eq!(result.0, expected);
            assert_eq!(result.1.is_some(), observe);
            assert_eq!(guard.calls, 1);
            let mut guard = Guard {
                calls: 0,
                allow: true,
            };
            let mut capture = StageCapture::new("dom");
            let result = emit_dom_source_with_preflight(
                &arena,
                source,
                LegacyCaps::VUE3,
                &options,
                true,
                false,
                &mut guard,
                observe,
                &mut capture,
            )
            .unwrap();
            assert_eq!(result.0, expected);
            assert_eq!(result.1.is_some(), observe);
            assert_eq!(guard.calls, 1);
            assert!(!capture.pages.is_empty());
        }
    }

    #[test]
    fn refused_preflight_publishes_no_lowering_or_capture_pages() {
        let arena = Allocator::default();
        let mut guard = Guard {
            calls: 0,
            allow: false,
        };
        let mut capture = StageCapture::new("dom");
        let result = emit_dom_source_with_preflight(
            &arena,
            "<div id='x'>text</div>",
            LegacyCaps::VUE3,
            &DomEmitOptions::DEFAULT,
            true,
            false,
            &mut guard,
            false,
            &mut capture,
        );
        assert_eq!(result, Err(EmitError::Diagnostics));
        assert_eq!(guard.calls, 1);
        assert!(capture.pages.is_empty());
    }
}
