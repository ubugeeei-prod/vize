use super::{META, script_reads, script_tests::linter};
use crate::{
    context::LintContext,
    diagnostic::{HelpLevel, LintDiagnostic},
};
use std::borrow::Cow;
use vize_atelier_sfc::{
    SfcDescriptor,
    types::{BlockLocation, SfcScriptBlock},
};
use vize_l0::Allocator;

#[test]
fn typeof_probe_must_witness_the_same_runtime_global_across_called_scopes() {
    let source = "<script setup>function read() { window.innerWidth; } { const window = {}; if (typeof window !== 'undefined') read(); }</script><template />";
    let result = linter().lint_sfc(source, "ShadowGuard.vue");
    assert_eq!((result.error_count, result.warning_count), (0, 1));
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, META.name);
    assert_eq!(
        diagnostic.message,
        "'window' is a browser-only global and is not available in SSR"
    );
    assert_eq!(
        (diagnostic.start as usize, diagnostic.end as usize),
        (
            source.find("window.innerWidth").unwrap(),
            source.find("window.innerWidth").unwrap() + 6
        )
    );
    assert!(diagnostic.help.is_none() && diagnostic.labels.is_empty() && diagnostic.fix.is_none());
    for source in [
        "<script setup>function read() { window.innerWidth; } if (typeof window !== 'undefined') read();</script><template />",
        "<script setup>function read() { const window = {}; window.innerWidth; } if (typeof window !== 'undefined') read();</script><template />",
        "<script setup>declare const window: Window; function read() { window.innerWidth; } if (typeof window !== 'undefined') read();</script><template />",
    ] {
        // The ambient declaration control requires its authored TS parser mode.
        let source = source.replace("<script setup>", "<script setup lang=\"ts\">");
        assert!(
            linter()
                .lint_sfc(&source, "GlobalGuard.vue")
                .diagnostics
                .is_empty()
        );
    }
}

fn assert_browser_read(source: &str, name: &str, start: usize) {
    let result = linter().lint_sfc(source, "SplitScope.vue");
    assert_eq!((result.error_count, result.warning_count), (0, 1));
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, META.name);
    assert_eq!(
        diagnostic.message,
        format!("'{name}' is a browser-only global and is not available in SSR")
    );
    assert_eq!(
        (diagnostic.start as usize, diagnostic.end as usize),
        (start, start + name.len())
    );
    assert_eq!(diagnostic.severity, crate::diagnostic::Severity::Warning);
    assert!(diagnostic.help.is_none() && diagnostic.labels.is_empty() && diagnostic.fix.is_none());
}

fn physical_orders(module: &str, setup: &str) -> [std::string::String; 2] {
    [
        format!("{module}{setup}<template />"),
        format!("{setup}{module}<template />"),
    ]
}

#[test]
fn setup_locals_do_not_shadow_module_reads_or_called_module_functions() {
    for (module, setup) in [
        (
            "<script>const width = window.innerWidth;</script>",
            "<script setup>const window = { innerWidth: 0 }; window.innerWidth;</script>",
        ),
        (
            "<script>function read() { return window.innerWidth; }</script>",
            "<script setup>const window = { innerWidth: 0 }; read();</script>",
        ),
        (
            "<script>function read() { return window.innerWidth; }</script>",
            "<script setup>const window = {}; if (typeof window !== 'undefined') read();</script>",
        ),
    ] {
        for source in physical_orders(module, setup) {
            let start = source.find(module).unwrap() + module.find("window.innerWidth").unwrap();
            assert_browser_read(&source, "window", start);
        }
    }
}

#[test]
fn module_and_setup_ordinary_duplicate_locals_remain_distinct() {
    for (module, setup) in [
        (
            "<script>const window = { innerWidth: 1 }; window.innerWidth;</script>",
            "<script setup>const window = { innerWidth: 2 }; window.innerWidth;</script>",
        ),
        (
            "<script>var window = { innerWidth: 1 }; window.innerWidth;</script>",
            "<script setup>var window = { innerWidth: 2 }; window.innerWidth;</script>",
        ),
        (
            "<script>function read() { return window.innerWidth; }</script>",
            "<script setup>function read() { return 0; } read();</script>",
        ),
        (
            "<script>const window = {}; function read() { window.innerWidth; }</script>",
            "<script setup>function read() { const window = {}; window.innerWidth; } read();</script>",
        ),
    ] {
        for source in physical_orders(module, setup) {
            assert!(
                linter()
                    .lint_sfc(&source, "Distinct.vue")
                    .diagnostics
                    .is_empty()
            );
        }
    }
}

#[test]
fn actual_setup_imports_are_hoisted_and_same_imports_are_deduplicated() {
    for source in physical_orders(
        "<script>const width = window.innerWidth;</script>",
        "<script setup>import { window } from 'browser-adapter'; window.innerWidth;</script>",
    ) {
        assert!(
            linter()
                .lint_sfc(&source, "HoistedImport.vue")
                .diagnostics
                .is_empty()
        );
    }
    for source in physical_orders(
        "<script>import { onServerPrefetch as server } from 'vue';</script>",
        "<script setup>import { onServerPrefetch as server } from 'vue'; server(() => navigator.language);</script>",
    ) {
        assert_browser_read(&source, "navigator", source.find("navigator").unwrap());
    }
}

#[test]
fn split_statements_unsupported_setup_exports_and_import_conflicts_are_refused() {
    for source in [
        "<script>const width =</script><script setup>window.innerWidth;</script><template />",
        "<script>const width = window.innerWidth;</script><script setup>export const local = 0;</script><template />",
        "<script>import { read } from 'first';</script><script setup>import { read } from 'second'; window.innerWidth;</script><template />",
    ] {
        assert!(
            linter()
                .lint_sfc(source, "Unsupported.vue")
                .diagnostics
                .is_empty()
        );
    }
}

#[test]
fn supplied_script_frames_keep_reads_after_trailing_comments_in_both_physical_orders() {
    for (module, setup) in [
        (
            "<script>const local = 0; // module comment</script>",
            "<script setup>window.innerWidth;</script>",
        ),
        (
            "<script>window.innerWidth;</script>",
            "<script setup>const local = 0; // setup comment</script>",
        ),
    ] {
        for source in physical_orders(module, setup) {
            let start = source.find("window.innerWidth").unwrap();
            // Supplied frames exercise the rule's private boundary mechanism.
            // They do not claim legacy SFC parser acceptance for a closing tag
            // inside an unterminated physical line comment.
            let rows = supplied_frames(&source, module, setup);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].rule_name, META.name);
            assert_eq!(
                rows[0].message,
                "'window' is a browser-only global and is not available in SSR"
            );
            assert_eq!(
                (rows[0].start as usize, rows[0].end as usize),
                (start, start + 6)
            );
            assert_eq!(rows[0].severity, crate::diagnostic::Severity::Warning);
            assert!(rows[0].help.is_none() && rows[0].labels.is_empty() && rows[0].fix.is_none());
        }
    }
}

fn supplied_frames(source: &str, module: &str, setup: &str) -> Vec<LintDiagnostic> {
    let block = |authored: &str, is_setup| {
        let tag_start = source.find(authored).unwrap();
        let start = tag_start + authored.find('>').unwrap() + 1;
        let end = tag_start + authored.find("</script>").unwrap();
        SfcScriptBlock {
            content: Cow::Borrowed(&source[start..end]),
            loc: BlockLocation {
                start,
                end,
                tag_start,
                tag_end: tag_start + authored.len(),
                start_line: 1,
                start_column: start + 1,
                end_line: 1,
                end_column: end + 1,
            },
            lang: None,
            src: None,
            setup: is_setup,
            attrs: Default::default(),
            bindings: None,
        }
    };
    let descriptor = SfcDescriptor {
        source: Cow::Borrowed(source),
        script: Some(block(module, false)),
        script_setup: Some(block(setup, true)),
        ..Default::default()
    };
    let allocator = Allocator::default();
    let mut ctx = LintContext::new(&allocator, source, "SuppliedFrames.vue");
    ctx.set_help_level(HelpLevel::None);
    ctx.set_sfc_descriptor(&descriptor);
    script_reads::check(&mut ctx);
    ctx.into_diagnostics()
}
