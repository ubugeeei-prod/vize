use super::options;
use crate::s3::{LegacyReason, VaporS3BridgeStatus, lower_source_for_vapor};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

/// Paragraphs, headings, anchors and forms, including the nestings HTML tree
/// construction leaves alone. Pinned against the retained lane's template
/// payload (its node numbering is child-first).
#[test]
fn paragraph_heading_anchor_and_form_elements_are_admitted() {
    for source in [
        r#"<p>{{ a }} <a :href="url">link</a></p>"#,
        r#"<section><h1>{{ title }}</h1><h2>sub</h2><p>body</p></section>"#,
        r#"<form @submit="save"><label>n</label><input :value="v"><button>go</button></form>"#,
        // A button bounds a paragraph's scope; a component or outlet bounds it too.
        r#"<p><button><div>{{ x }}</div></button></p>"#,
        r#"<p><MyComp><div>{{ x }}</div></MyComp></p>"#,
        r#"<a href="/x"><span>{{ label }}</span></a>"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
        let native = compile_vapor(&allocator, source, VaporCompilerOptions::default());
        let retained = compile_vapor(
            &allocator,
            source,
            VaporCompilerOptions {
                davinci_retained_lane: true,
                ..Default::default()
            },
        );
        assert_eq!(native.templates, retained.templates, "{source}");
    }
}

/// Phrasing elements have ordinary HTML tree construction. The native and
/// retained lanes number addressed nodes differently, but their templates
/// must match exactly; the mounted test pins dynamic child behavior.
#[test]
fn semantic_inline_elements_match_the_retained_lane() {
    for source in [
        r#"<p><abbr title="Application Programming Interface">API</abbr> <code>{{ value }}</code> <mark v-if="hot">hot</mark></p>"#,
        r#"<p><time :datetime="date">{{ date }}</time><data :value="id"><kbd>Enter</kbd></data></p>"#,
        r#"<p><samp>{{ sample }}</samp><var>x</var><sub>2</sub><sup>3</sup></p>"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
        for prefix_identifiers in [false, true] {
            let compile = |davinci_retained_lane| {
                compile_vapor(
                    &allocator,
                    source,
                    VaporCompilerOptions {
                        prefix_identifiers,
                        davinci_retained_lane,
                        ..Default::default()
                    },
                )
                .templates
            };
            assert_eq!(compile(false), compile(true), "{source}");
        }
    }
}

#[test]
fn repaired_nestings_select_the_legacy_lane() {
    for source in [
        r#"<p><div>x</div></p>"#,
        r#"<p><span><ul><li>x</li></ul></span></p>"#,
        r#"<p><h1>x</h1></p>"#,
        r#"<p><p>x</p></p>"#,
        r#"<p><b v-if="a"><section>x</section></b></p>"#,
        r#"<p><code><div>x</div></code></p>"#,
        r#"<a><span><a>x</a></span></a>"#,
        r#"<a><MyComp><a>x</a></MyComp></a>"#,
        r#"<form><div><form></form></div></form>"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Legacy(reason)
                if matches!(reason, LegacyReason::Structure | LegacyReason::SurfaceSemantics)),
            "{source}: {status:?}"
        );
    }
}
