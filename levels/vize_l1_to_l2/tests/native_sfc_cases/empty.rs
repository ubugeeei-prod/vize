use super::support::options;
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::{NativeSfcIssueKind, lower_sfc_native};

#[test]
fn empty_ordinary_and_setup_are_retained_typed_refusals_without_units() {
    let arena = Allocator::default();
    for source in [
        "<script></script>",
        "<script setup></script>",
        "<script> \r\n\t</script><template></template>",
        "<script setup>\u{a0}\u{feff}</script><template></template>",
    ] {
        let observed = lower_sfc_native(&arena, source, options());
        assert!(observed.descriptor().admitted().is_ok());
        assert!(observed.admitted().is_none());
        let script = observed.scripts().first().unwrap();
        let syntax = script.syntax().unwrap();
        assert!(syntax.admitted_program().is_some());
        assert_eq!(syntax.source().span(), script.block().span());
        assert!(script.unit().is_none());
        assert!(
            observed
                .issues()
                .iter()
                .any(|issue| issue.kind == NativeSfcIssueKind::EmptyScriptSelection)
        );
        assert!(observed.file().unwrap().file().units().is_empty());
    }
}

#[test]
fn empty_setup_cannot_turn_ordinary_locals_into_combined_vue_exposure() {
    let arena = Allocator::default();
    let source =
        "<script setup></script><script>const value = 1;</script><template>kept</template>";
    let observed = lower_sfc_native(&arena, source, options());
    assert!(observed.admitted().is_none());
    assert_eq!(observed.scripts().len(), 2);
    let file = observed.file().unwrap();
    assert!(file.setup().is_none());
    assert!(file.ordinary().is_some());
    assert_eq!(file.file().units().len(), 1);
    for binding in file.file().bindings() {
        assert!(file.exposure(binding).is_none());
    }
    assert!(observed.template().unwrap().produced().is_some());
}

#[test]
fn comment_directive_semicolon_and_hashbang_content_are_not_empty_roles() {
    let arena = Allocator::default();
    for source in [
        "<script>// comment\n</script>",
        "<script setup>/* comment */</script>",
        "<script>\"use strict\";</script>",
        "<script setup>;</script>",
        "<script>#!/usr/bin/env node\n</script>",
    ] {
        let observed = lower_sfc_native(&arena, source, options());
        let script = observed.scripts().first().unwrap();
        let program = script.syntax().unwrap().program().unwrap();
        assert!(
            !program.body.is_empty()
                || !program.comments.is_empty()
                || !program.directives.is_empty()
                || program.hashbang.is_some()
        );
        assert!(script.unit().is_some());
        assert!(
            !observed
                .issues()
                .iter()
                .any(|issue| issue.kind == NativeSfcIssueKind::EmptyScriptSelection)
        );
    }
}
