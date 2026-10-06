use super::{META, NoBrowserGlobalsInSsr, script_reads};
use crate::{Linter, diagnostic::HelpLevel, rule::RuleRegistry};
use oxc_span::SourceType;
use vize_l0::String;

const ORIGINAL: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/ssr-script-setup-7982/WidthLabel.vue.txt"
);
const BAD: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/ssr-script-setup-7982/DocsBad.vue.txt"
);
const GOOD: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/ssr-script-setup-7982/DocsGood.vue.txt"
);

fn linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.add(Box::new(NoBrowserGlobalsInSsr));
    Linter::with_registry(registry).with_help_level(HelpLevel::None)
}

fn script_names(source: &str) -> Vec<String> {
    script_reads::find_reads(source, SourceType::ts())
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

#[test]
fn original_issue_and_documented_bad_good_preserve_complete_diagnostics() {
    let result = linter().lint_sfc(ORIGINAL, "WidthLabel.vue");
    assert_eq!((result.error_count, result.warning_count), (0, 3));
    let expected = [
        ("window", ORIGINAL.find("window").unwrap(), 6),
        ("navigator", ORIGINAL.find("navigator").unwrap(), 9),
        (
            "document",
            ORIGINAL.find("{{ document").unwrap(),
            "{{ document.title }}".len(),
        ),
    ];
    for (diagnostic, (name, start, len)) in result.diagnostics.iter().zip(expected) {
        assert_eq!(diagnostic.rule_name, META.name);
        assert_eq!(
            diagnostic.message,
            format!("'{name}' is a browser-only global and is not available in SSR")
        );
        assert_eq!(
            (diagnostic.start, diagnostic.end),
            (start as u32, (start + len) as u32)
        );
        assert_eq!(diagnostic.severity, crate::diagnostic::Severity::Warning);
        assert!(
            diagnostic.help.is_none() && diagnostic.labels.is_empty() && diagnostic.fix.is_none()
        );
    }
    let bad = linter().lint_sfc(BAD, "Bad.vue");
    assert_eq!(bad.warning_count, 1);
    assert_eq!(
        &BAD[bad.diagnostics[0].start as usize..bad.diagnostics[0].end as usize],
        "window"
    );
    assert!(linter().lint_sfc(GOOD, "Good.vue").diagnostics.is_empty());
}

#[test]
fn deferred_mount_event_and_arbitrary_callback_bodies_stay_quiet() {
    assert!(
        script_names(
            r#"
        import { onMounted } from 'vue';
        const click = () => window.alert(document.title);
        onMounted(() => navigator.language);
        registerHandler(() => localStorage.getItem('theme'));
        function event() { return window.innerWidth; }
    "#
        )
        .is_empty()
    );
    assert!(linter().lint_sfc(r#"<script setup>const click = () => window.alert('x');</script><template><button @click="click">Go</button></template>"#, "Event.vue").diagnostics.is_empty());
}

#[test]
fn called_local_functions_iifes_and_aliases_of_vue_server_hooks_execute() {
    let names = script_names(
        r#"
        import { onServerPrefetch as prefetch, watchSyncEffect } from 'vue';
        function read() { return window.innerWidth; }
        read();
        (() => document.title)();
        const callback = () => navigator.language;
        prefetch(callback);
        watchSyncEffect(() => localStorage.getItem('theme'));
    "#,
    );
    assert_eq!(names, ["window", "document", "navigator", "localStorage"]);
}

#[test]
fn effect_options_and_unrelated_hook_names_do_not_forge_execution() {
    assert_eq!(
        script_names(
            r#"
        import { watchEffect } from 'vue';
        import { onServerPrefetch } from 'other';
        watchEffect(() => window.innerWidth);
        watchEffect(() => document.title, { flush: 'post' });
        onServerPrefetch(() => navigator.language);
    "#
        ),
        ["window"]
    );
}

#[test]
fn lexical_parameters_hoisting_and_local_hook_shadow_are_authoritative() {
    assert!(
        script_names(
            r#"
        const window = { innerWidth: 0 };
        window.innerWidth;
        function read(document: { title: string }) { return document.title; }
        read({ title: 'safe' });
        navigator();
        function navigator() { return 0; }
    "#
        )
        .is_empty()
    );
    assert_eq!(
        script_names(
            r#"
        function onMounted() { return window.innerWidth; }
        onMounted();
    "#
        ),
        ["window"]
    );
}

#[test]
fn module_declarations_are_visible_to_setup_in_either_physical_order() {
    for source in [
        "<script>const window = { innerWidth: 0 };</script><script setup>window.innerWidth;</script><template />",
        "<script setup>window.innerWidth;</script><script>const window = { innerWidth: 0 };</script><template />",
    ] {
        assert!(
            linter()
                .lint_sfc(source, "Split.vue")
                .diagnostics
                .is_empty()
        );
    }
}

#[test]
fn strings_keys_comments_regex_and_type_queries_are_not_runtime_reads() {
    assert!(
        script_names(
            r#"
        const text = 'window'; const regex = /document/;
        const object = { navigator: true }; object.navigator;
        /* localStorage */ type Browser = typeof window;
        type EventType = MouseEvent;
        declare const doc: typeof document;
    "#
        )
        .is_empty()
    );
    assert_eq!(
        script_names("const value = window.innerWidth as MouseEvent;"),
        ["window"]
    );
}

#[test]
fn direct_typeof_and_witnessed_defined_branches_are_safe_but_member_probe_is_not() {
    assert!(
        script_names(
            r#"
        typeof (window);
        if (typeof window !== 'undefined') { window.innerWidth; }
        typeof document === 'undefined' ? '' : document.title;
        typeof navigator !== 'undefined' && navigator.language;
        typeof localStorage === 'undefined' || localStorage.length;
    "#
        )
        .is_empty()
    );
    assert_eq!(
        script_names(
            r#"
        typeof window.innerWidth;
        if (typeof document === 'undefined') { document.title; }
        if (typeof window !== 'undefined') {} window.innerWidth;
    "#
        ),
        ["window", "document", "window"]
    );
}

#[test]
fn escaped_identifiers_and_unicode_crlf_keep_physical_source_byte_ranges() {
    let source = "<script setup lang=\"ts\">\r\nconst 雪 = '🌸'; \\u0077indow.innerWidth;\r\n</script>\r\n<template />";
    let result = linter().lint_sfc(source, "Unicode.vue");
    assert_eq!(result.warning_count, 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        r"\u0077indow"
    );
    assert_eq!(
        diagnostic.start as usize,
        source.find(r"\u0077indow").unwrap()
    );
}

#[test]
fn language_modes_are_authored_and_invalid_or_external_source_is_refused() {
    for (lang, source) in [
        ("ts", "const value = <number>window.innerWidth;"),
        ("tsx", "const value = <p>{window.innerWidth}</p>;"),
        ("jsx", "const value = <p>{window.innerWidth}</p>;"),
        ("js", "const value = window.innerWidth;"),
    ] {
        let source = format!("<script setup lang=\"{lang}\">{source}</script><template />");
        assert_eq!(linter().lint_sfc(&source, "Language.vue").warning_count, 1);
    }
    for source in [
        "<script setup lang=\"coffee\">window.innerWidth</script><template />",
        "<script src=\"./other.js\"/><script setup>window.innerWidth;</script><template />",
    ] {
        assert!(
            linter()
                .lint_sfc(source, "Opaque.vue")
                .diagnostics
                .is_empty()
        );
    }
    assert!(script_names("const broken = ; window.innerWidth;").is_empty());
}

#[test]
fn script_only_sfc_suppression_and_disabled_rule_are_preserved() {
    assert_eq!(
        linter()
            .lint_sfc("<script setup>window.innerWidth;</script>", "Only.vue")
            .warning_count,
        1
    );
    assert!(linter().lint_sfc("<script setup>\n// eslint-disable-next-line ssr/no-browser-globals-in-ssr\nwindow.innerWidth;\n</script>", "Suppressed.vue").diagnostics.is_empty());
    assert!(
        linter()
            .with_disabled_rules(vec![String::from(META.name)])
            .lint_sfc(ORIGINAL, "Disabled.vue")
            .diagnostics
            .is_empty()
    );
}

#[test]
fn recursion_and_repeated_calls_emit_each_physical_read_once() {
    assert_eq!(
        script_names("function run() { window.innerWidth; run(); } run(); run();"),
        ["window"]
    );
}

#[test]
fn instance_fields_are_deferred_but_class_static_initializers_execute() {
    assert!(
        script_names(
            "class Panel { title = document.title; read() { return window.innerWidth; } }"
        )
        .is_empty()
    );
    assert_eq!(
        script_names(
            "class Panel { static title = document.title; static { window.innerWidth; } }"
        ),
        ["document", "window"]
    );
}

#[test]
fn ambient_and_type_imports_do_not_forge_runtime_shadow_or_execution() {
    assert!(script_names("function* read() { window.innerWidth; } read();").is_empty());
    assert_eq!(
        script_names("declare const window: { innerWidth: number }; window.innerWidth;"),
        ["window"]
    );
    assert_eq!(
        script_names("import type { window } from 'types'; window.innerWidth;"),
        ["window"]
    );
    assert!(script_names("let read = () => window.innerWidth; read = () => 0; read();").is_empty());
}
