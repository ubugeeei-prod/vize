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
