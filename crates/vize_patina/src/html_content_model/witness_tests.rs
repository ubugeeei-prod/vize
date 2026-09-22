//! TS-36 for composed findings: every witness verifies; forged ones fail
//! with the exact error.

#[cfg(test)]
mod witnesses {
    use crate::html_content_model::witness::{
        CHECKS, CrossComponentNesting, HtmlComposedNesting, HtmlElements, NestingEvidence, audit,
        facts, witnessed,
    };
    use crate::html_content_model::{Skeleton, authored_skeleton, compose};
    use vize_davinci::diagnostic::{Diagnostic, Stage, WitnessChain, WitnessLink};
    use vize_davinci::fact::ids;
    use vize_davinci::witness::{WitnessError, verify};
    use vize_s0::{Allocator, Span, String};

    fn evidence(files: &[(&str, &str)]) -> NestingEvidence {
        let skeletons: Vec<Skeleton> = files
            .iter()
            .map(|(_, source)| authored_skeleton(&Allocator::with_capacity(4096), source))
            .collect();
        let resolve = |_: u32, tag: &str| {
            let index = files.iter().position(|(name, _)| *name == tag)?;
            u32::try_from(index).ok()
        };
        NestingEvidence::new(&skeletons, &compose(&skeletons, &resolve))
    }

    const SHOWCASE: [(&str, &str); 4] = [
        (
            "App",
            "<article><p>a <Card /></p><table><Row /></table><a href=\"#\"><Btn /></a></article>",
        ),
        ("Card", "<div>body</div>"),
        ("Row", "<tr><td>x</td></tr>"),
        ("Btn", "<button>go</button>"),
    ];

    #[test]
    fn every_composed_witness_verifies() {
        let evidence = evidence(&SHOWCASE);
        let diagnostics = witnessed(&evidence, |_| String::default());
        assert_eq!(diagnostics.len(), 3);
        let report = audit(&evidence, &diagnostics).expect("facts compute");
        if cfg!(debug_assertions) {
            assert_eq!((report.observed, report.verified), (3, 3));
            assert_eq!(report.failures, Vec::new());
        }
        let manager = facts(&evidence).expect("facts compute");
        let view = manager.view::<CrossComponentNesting>();
        for diagnostic in &diagnostics {
            assert_eq!(verify(diagnostic, &view, &CHECKS), Ok(()));
        }
    }

    #[test]
    fn forged_witnesses_fail_with_the_exact_error() {
        let evidence = evidence(&SHOWCASE);
        let manager = facts(&evidence).expect("facts compute");
        let view = manager.view::<CrossComponentNesting>();
        let forge = |chain| Diagnostic::proven(Stage::Semantic, Span::new(0, 1), "forged", chain);
        // The right fact, cited from the wrong place.
        let moved = forge(WitnessChain::new(WitnessLink::of::<HtmlComposedNesting>(
            &0,
            Span::new(0, 1),
        )));
        assert!(matches!(
            verify(&moved, &view, &CHECKS),
            Err(WitnessError::SpanMismatch { link: 0, group, .. }) if group == ids::HTML_COMPOSED_NESTING
        ));
        // A key no fact is stored under.
        let missing = forge(WitnessChain::new(WitnessLink::of::<HtmlElements>(
            &99,
            Span::new(0, 1),
        )));
        assert!(matches!(
            verify(&missing, &view, &CHECKS),
            Err(WitnessError::MissingKey { link: 0, group, .. }) if group == ids::HTML_ELEMENTS
        ));
    }
}
