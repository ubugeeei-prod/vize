use tower_lsp::lsp_types::Url;

use crate::{ide::IdeContext, server::ServerState};

#[cfg(feature = "native")]
mod publication;

const DOCUMENTATION: &str = concat!(
    "**CSS color**\n\n_Concrete authored color_\n\n**Example**\n\n",
    "```css\n.example {\n  color: #336699;\n  color: rgb(51, 102, 153);\n",
    "  color: hsl(210 50% 40%);\n}\n```\n\n**Docs**\n\n",
    "[MDN CSS colors](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_colors)"
);

fn document(source: &str) -> (ServerState, Url) {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({"typecheck": false})));
    let uri = Url::parse("file:///css-literal-hover/App.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    (state, uri)
}

fn position(source: &str, offset: usize) -> serde_json::Value {
    let prefix = source.get(..offset).unwrap();
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let character = prefix.rsplit('\n').next().unwrap().encode_utf16().count();
    serde_json::json!({"line":line,"character":character})
}

#[test]
fn complete_hover_packets_keep_authored_utf16_ranges_in_every_supported_style_profile() {
    for ending in ["\n", "\r\n"] {
        for language in [
            None,
            Some("css"),
            Some("scss"),
            Some("less"),
            Some("postcss"),
            Some("sass"),
            Some("stylus"),
            Some("styl"),
        ] {
            let attribute = language.map_or(String::new(), |lang| format!(" lang=\"{lang}\""));
            let body = if matches!(language, Some("sass" | "stylus" | "styl")) {
                format!(".日本🧭{ending}  color: #336699{ending}")
            } else {
                "/*日本🧭*/ .x { color: #336699; }".into()
            };
            let source =
                format!("<template><div /></template>{ending}<style{attribute}>{body}</style>");
            let start = source.find("#336699").unwrap();
            let end = start + 7;
            let expected = serde_json::json!({
                "contents":{"kind":"markdown","value":DOCUMENTATION},
                "range":{"start":position(&source,start),"end":position(&source,end)}
            });
            let (state, uri) = document(&source);
            for offset in start..end {
                let ctx = IdeContext::new(&state, &uri, offset).unwrap();
                assert_eq!(
                    serde_json::to_value(super::hover(&ctx).unwrap()).unwrap(),
                    expected,
                    "{language:?}/{ending:?}/{offset}"
                );
            }
            for offset in [start - 1, end] {
                let ctx = IdeContext::new(&state, &uri, offset).unwrap();
                assert_eq!(
                    super::hover(&ctx).map(|hover| serde_json::to_value(hover).unwrap()),
                    None
                );
            }
            assert_eq!(
                state.resident.take_stats(),
                vize_incremental::DescriptorStats {
                    lookups: 9,
                    parses: 1
                },
                "every selected query reuses the resident descriptor for {language:?}/{ending:?}"
            );
            #[cfg(feature = "native")]
            assert!(!state.has_corsa_bridge());
        }
    }
}

#[test]
fn unsafe_or_unsupported_lookalikes_cannot_gain_color_hover() {
    let cases = [
        ("css", "#336699 { color: red; }", "#336699"),
        (
            "css",
            ".outer { color:#336699:hover { color: red; } }",
            "#336699",
        ),
        ("css", ".x { content: '#336699'; }", "#336699"),
        ("css", ".x { /* #336699 */ color:red; }", "#336699"),
        ("css", ".x { background:url(#336699); }", "#336699"),
        ("css", ".x { color:rgb(var(--value) 0 0); }", "rgb"),
        ("css", ".x { color:rgb(1 invalid 3); }", "rgb"),
        ("css", ".x { color:#336699z; }", "#336699"),
        ("scss", ".x { color:#{'#336699'}; }", "#336699"),
        ("less", ".x { color:@{'#336699'}; }", "#336699"),
        ("unknown", ".x { color:#336699; }", "#336699"),
    ];
    for (language, body, needle) in cases {
        let source = format!("<style lang=\"{language}\">{body}</style>");
        let offset = source.find(needle).unwrap() + 1;
        let (state, uri) = document(&source);
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        assert_eq!(
            super::hover(&ctx).map(|hover| serde_json::to_value(hover).unwrap()),
            None,
            "{source}"
        );
    }
}

#[test]
fn unsupported_functional_comments_are_refused_and_multiline_values_are_normalized() {
    let literal = "rgb(51 /* ```css\n**unsafe** */ 102 153)";
    let source = format!("<style>.日本🧭 {{ color:{literal}; }}</style>");
    let start = source.find(literal).unwrap();
    let (state, uri) = document(&source);
    let ctx = IdeContext::new(&state, &uri, start + 2).unwrap();
    assert_eq!(
        super::hover(&ctx).map(|hover| serde_json::to_value(hover).unwrap()),
        None
    );

    // Preserve the existing parser's exact admission boundary. Valid multiline
    // whitespace is accepted, and only its normalized presentation is rendered.
    let literal = "rgb(51\n  102 153)";
    let source = format!("<style>.日本🧭 {{ color:{literal}; }}</style>");
    let start = source.find(literal).unwrap();
    let end = start + literal.len();
    let (state, uri) = document(&source);
    let ctx = IdeContext::new(&state, &uri, start + 2).unwrap();
    assert_eq!(
        serde_json::to_value(super::hover(&ctx).unwrap()).unwrap(),
        serde_json::json!({
            "contents":{"kind":"markdown","value":DOCUMENTATION},
            "range":{"start":position(&source,start),"end":position(&source,end)}
        })
    );
}
