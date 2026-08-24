//! The committed TS-19 fixture battery, shared by the plain suite
//! (`surface_fidelity.rs`) and the corpus-runnable lane
//! (`davinci_surface_corpus.rs`).
//!
//! Every fixture pins its typed-hole census exactly — the structural
//! half of TS-19, so a recovery-path change moves a pinned number
//! loudly instead of drifting silently.

pub struct Fixture {
    pub name: &'static str,
    pub source: &'static str,
    /// Hole policy clause 1, token level (`TokenStatus::Missing`).
    pub missing_tokens: usize,
    /// Hole policy clause 1, node level (`ElementClose::Missing`).
    pub missing_close_tags: usize,
    /// Hole policy clause 2 (`SurfaceChild::Unexpected`).
    pub unexpected_nodes: usize,
}

const fn well_formed(name: &'static str, source: &'static str) -> Fixture {
    Fixture {
        name,
        source,
        missing_tokens: 0,
        missing_close_tags: 0,
        unexpected_nodes: 0,
    }
}

const fn malformed(
    name: &'static str,
    source: &'static str,
    missing_tokens: usize,
    missing_close_tags: usize,
    unexpected_nodes: usize,
) -> Fixture {
    Fixture {
        name,
        source,
        missing_tokens,
        missing_close_tags,
        unexpected_nodes,
    }
}

/// Hole-free templates: every construct S1 models, zero holes expected.
pub const WELL_FORMED: &[Fixture] = &[
    well_formed(
        "kitchen_sink",
        "<div :id=\"a && b\" v-if=\"ok\" @click.stop=\"go($event)\">\n  {{ msg }}\n  <span v-for=\"item of items\" :key=\"item.id\">{{ item.name }}</span>\n</div>",
    ),
    well_formed(
        "entities_raw",
        "<p title=\"a &amp; b\">x &amp; y &#x27; z &unknown;</p>",
    ),
    well_formed(
        "void_and_self_closing",
        "<br /><img src=\"x\"><input disabled>",
    ),
    well_formed("quote_forms", "<a b='1' c=\"2\" d=3 e>t</a>"),
    well_formed(
        "dynamic_directives",
        "<a v-on:click.stop.prevent=\"f\" :[key]=\"v\" #named>x</a>",
    ),
    well_formed("comments", "<!-- c --><div><!-- inner --></div><!---->"),
    well_formed("interpolation_spacing", "{{msg}}  {{ a + b }}\t{{c}}"),
    well_formed("multi_root", "<div>a</div>\n<p>b</p>"),
    well_formed("crlf", "<div\r\n  a=\"1\"\r\n>x\r\n</div>\r\n"),
    well_formed(
        "unicode",
        "<div title=\"日本語\">こんにちは {{ 名前 }}</div>",
    ),
    well_formed(
        "svg_self_closing",
        "<svg viewBox=\"0 0 1 1\"><path d=\"M0 0\"/></svg>",
    ),
    well_formed("pre_whitespace", "<pre>  a\n\n  b  </pre>"),
    well_formed(
        "rcdata_textarea_title",
        "<textarea>{{ x }} & <plain</textarea><title>a < b</title>",
    ),
    well_formed(
        "rcdata_script_style",
        "<script>if (a < b) { x(); }</script><style>a { color: red; }</style>",
    ),
    well_formed(
        "nested_template",
        "<template v-if=\"x\"><slot name=\"s\"><span>a</span></slot></template>",
    ),
    well_formed("close_tag_whitespace", "<div>x</div\n>"),
];

/// Malformed and recovered forms. The pinned triples are
/// (missing tokens, missing close tags, unexpected nodes); a fixture
/// with all zeros recovers without holes (the bytes still round-trip and
/// the tokenizer still reports its diagnostic).
pub const MALFORMED: &[Fixture] = &[
    // EOF-in-tag: the `>` is a Missing token, the end tag a Missing node.
    malformed("eof_in_tag", "<div", 1, 1, 0),
    malformed("eof_in_tag_ws", "<div ", 1, 1, 0),
    malformed("eof_in_attr_name", "<div cla", 1, 1, 0),
    // EOF in a quoted value: closing quote and `>` both Missing.
    malformed("eof_in_quoted_value", "<div a=\"x", 2, 1, 0),
    malformed("eof_in_empty_quoted_value", "<div a=\"", 2, 1, 0),
    // Unclosed child: the node-level hole, parent still closes.
    malformed("unclosed_child", "<div><span></div>", 0, 1, 0),
    // Stray end tags: typed Unexpected nodes.
    malformed("stray_close", "</div>", 0, 0, 1),
    malformed("extra_close", "<div></div></div>", 0, 0, 1),
    malformed("empty_close", "</>x", 0, 0, 1),
    malformed("eof_in_close", "</di", 0, 0, 1),
    malformed("eof_after_close_slash", "</", 0, 0, 1),
    // Announced-but-absent value: the content is a Missing token.
    malformed("missing_attr_value", "<div a= ></div>", 1, 0, 0),
    // Recovered without holes: junk rides in `leading` under a
    // diagnostic (hole policy clause 3).
    malformed("solidus_junk", "<div / a>x</div>", 0, 0, 0),
    malformed("attr_quote_no_eq", "<div a\"v\">x</div>", 0, 0, 0),
    malformed(
        "missing_attr_whitespace",
        "<div a=\"b\"c=\"d\">x</div>",
        0,
        0,
        0,
    ),
    malformed("close_tag_junk", "<div>x</div foo>", 0, 0, 0),
    malformed("unclosed_interpolation", "{{ foo", 0, 0, 0),
    malformed("unclosed_comment", "<!-- x", 0, 0, 0),
    malformed("abrupt_empty_comment", "<!-->x", 0, 0, 0),
    // Declarations and one-dash bogus comments are consumed *silently*
    // by the tokenizer (`state_in_declaration` reports no event), so
    // their bytes surface as typed Unexpected holes — measured, and a
    // better fit for the hole policy than inventing a comment node the
    // tokenizer never reported.
    malformed("bogus_doctype", "<!DOCTYPE html><div>x</div>", 0, 0, 1),
    malformed("bogus_one_dash", "<!-x><div>y</div>", 0, 0, 1),
    malformed("processing_instruction", "<?xml version=\"1.0\"?>", 0, 0, 0),
    malformed("cdata_in_svg", "<svg><![CDATA[a < b]]></svg>", 0, 0, 0),
    malformed("unquoted_slash_value", "<a href=/x/y>z</a>", 0, 0, 0),
    malformed("case_insensitive_close", "<DIV>x</div>", 0, 0, 0),
    // v1 shape pin: no implied end tags — a sibling `<li>` nests until
    // an explicit close, so both stay open at EOF (recorded in the P2-7
    // record; reconciling with the shipping parser's tree is P2-8's).
    malformed("sibling_li_nests_v1", "<li>a<li>b", 0, 2, 0),
];
