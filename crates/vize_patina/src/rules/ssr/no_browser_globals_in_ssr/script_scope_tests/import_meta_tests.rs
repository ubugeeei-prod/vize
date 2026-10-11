use super::super::{META, script_reads, script_tests::linter};
use oxc_span::SourceType;

const CORPUS: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/linter/ssr-import-meta-guards/ClientGuards.vue.txt"
);

fn names(source: &str) -> Vec<vize_l0::String> {
    script_reads::find_reads(source, SourceType::ts())
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

#[test]
fn direct_import_meta_branches_preserve_complete_unsafe_diagnostics() {
    for source in [CORPUS.to_owned(), CORPUS.replace('\n', "\r\n")] {
        let result = linter().lint_sfc(&source, "ClientGuards.vue");
        assert_eq!((result.error_count, result.warning_count), (0, 6));
        assert_eq!(result.diagnostics.len(), 6);
        let expected = [
            ("window", 243, 9),
            ("document", 272, 11),
            ("navigator", 318, 14),
            ("document", 348, 16),
            ("document", 543, 21),
            ("document", 607, 23),
        ];
        for (diagnostic, (name, start, line)) in result.diagnostics.iter().zip(expected) {
            let start = start + if source.contains('\r') { line - 1 } else { 0 };
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
            assert!(
                diagnostic.help.is_none()
                    && diagnostic.labels.is_empty()
                    && diagnostic.fix.is_none()
            );
        }
    }
}

#[test]
fn direct_flag_polarity_is_shared_by_if_conditional_and_short_circuit_arms() {
    for (client, non_client) in [
        ("import.meta.client", "!import.meta.client"),
        ("!!(import.meta.client)", "!(import.meta.client)"),
    ] {
        let safe = format!(
            "if ({client}) {{ window.innerWidth; }} else {{ typeof document; }} {client} ? document.title : 0; {client} && navigator.language; {non_client} || localStorage.length;"
        );
        assert!(names(&safe).is_empty(), "{safe}");
        let unsafe_source = format!(
            "if ({client}) {{}} else {{ window.innerWidth; }} {client} ? 0 : document.title; {non_client} && navigator.language; {client} || localStorage.length;"
        );
        assert_eq!(
            names(&unsafe_source),
            ["window", "document", "navigator", "localStorage"],
            "{unsafe_source}"
        );
    }
}

#[test]
fn absent_server_flags_do_not_prove_either_branch_client_only() {
    // Without Nuxt's injected flag, !import.meta.server is true during SSR.
    // A generic SSR context therefore retains both arms and each polarity.
    for condition in [
        "import.meta.server",
        "!import.meta.server",
        "!!(import.meta.server)",
    ] {
        let source = format!(
            "if ({condition}) {{ window.innerWidth; }} else {{ document.title; }} {condition} ? navigator.language : localStorage.length; {condition} && location.href; {condition} || history.length;"
        );
        assert_eq!(
            names(&source),
            [
                "window",
                "document",
                "navigator",
                "localStorage",
                "location",
                "history"
            ],
            "{source}"
        );
    }
}

#[test]
fn lexical_lookalikes_aliases_and_unproved_conditions_remain_unsafe() {
    for condition in [
        "meta.client",
        "env.client",
        "process.client",
        "client",
        "import.meta['client']",
        "import.meta.client?.value",
        "import.meta.client ?? false",
    ] {
        let source = format!(
            "const meta = {{ client: false }}; const env = meta; const client = import.meta.client; if ({condition}) {{ document.title; }}"
        );
        assert_eq!(names(&source), ["document"], "{source}");
    }
    assert_eq!(
        names("if (document.title && import.meta.client) window.innerWidth;"),
        ["document", "window"]
    );
    assert_eq!(names("import.meta.client ?? document.title;"), ["document"]);
    assert_eq!(
        names("if (import.meta.client) document.title; document.title;"),
        ["document"]
    );
    assert!(
        names("const document = { title: 'local' }; if (import.meta.server) document.title;")
            .is_empty()
    );
}

#[test]
fn client_only_call_sites_do_not_hide_server_calls_or_typeof_laws() {
    assert_eq!(
        names("function read() { window.innerWidth; } if (import.meta.client) read(); read();"),
        ["window"]
    );
    assert_eq!(
        names(
            "if (import.meta.server) { if (typeof window !== 'undefined') window.innerWidth; document.title; }"
        ),
        ["document"]
    );
    assert_eq!(
        names(
            "if (import.meta.client) {} else { if (typeof document === 'undefined') document.title; }"
        ),
        ["document"]
    );
    assert_eq!(
        names(
            "const window = {}; if (typeof window !== 'undefined') { function read() { document.title; } read(); }"
        ),
        ["document"]
    );
}
