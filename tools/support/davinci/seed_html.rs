//! TS-37 seeded HTML nesting classes (Davinci P4-11a / P4-11b) for
//! `seed-defects.rs --html-nesting`.
//!
//! Every eligible `.vue` file gets one snippet per nesting class appended to
//! its template root, each written so the P4-11a checker proves exactly the
//! marked findings; files with a `<script setup>` also get the P4-11b
//! cross-component class: an imported block component whose `<div>` root is
//! rendered inside `<p>`. The expected diagnostics are identities (file,
//! rule, span), so recall is count-exact by identity.
//!
//! A file whose name (stem) appears in another source file is not seeded: its
//! template may be composed into a parent, and a seeded root there would add
//! composed findings at the parent's usage site that no manifest row names.

pub const NESTING_RULE: &str = "vue/permitted-contents";
pub const COMPOSED_RULE: &str = "html/cross-component-nesting";
pub const COMPOSED_CLASS: &str = "cross-component-nesting";
pub const BLOCK_FILE: &str = "__davinci_seed_block.vue";
pub const BLOCK_SOURCE: &str = "<template>\n  <div>davinci seeded block</div>\n</template>\n";
const BLOCK_IMPORT: &str = "import DavinciSeedBlock from \"./__davinci_seed_block.vue\";\n";
const COMPOSED_SNIPPET: &str = "<p>@<DavinciSeedBlock /></p>";

/// One snippet per class, in `ViolationClass` order; each `@` marks the start
/// of one expected finding (an element's `<tag`, or a text node), in order.
pub const NESTING: &[(&str, &[&str])] = &[
    ("<p>a @<div>b</div></p>", &["paragraph-auto-closed"]),
    ("<h2>a @<h3>b</h3></h2>", &["heading-auto-closed"]),
    (
        "<ul><li>a <span>@<li>b</li></span></li></ul>",
        &["list-item-auto-closed"],
    ),
    (
        "<form><div>@<form></form></div></form>",
        &["nested-form-dropped"],
    ),
    (
        "<a href=\"#\">a <div>@<a href=\"#\">b</a></div></a>",
        &["formatting-adopted"],
    ),
    (
        "<button>a <span>@<button>b</button></span></button>",
        &["button-auto-closed"],
    ),
    (
        "<select>@<select></select></select>",
        &["select-auto-closed"],
    ),
    (
        "<select><option>a @<option>b</option></option></select>",
        &["implied-end-tag-closed"],
    ),
    ("<div>@<tr><td>a</td></tr></div>", &["table-part-misplaced"]),
    (
        "<table>@<tr><td>a</td></tr></table>",
        &["table-wrapper-inserted"],
    ),
    (
        "<table><tbody>@<thead></thead></tbody></table>",
        &["table-auto-closed"],
    ),
    ("<table>@<div>a</div></table>", &["foster-parented"]),
    (
        "<table>@<form>@<tbody></tbody></form></table>",
        &["child-not-permitted", "form-in-table-emptied"],
    ),
    ("<svg>@<div>a</div></svg>", &["foreign-content-breakout"]),
    ("<div>@<body></body></div>", &["document-element-dropped"]),
    ("<div>@<image></div>", &["image-renamed"]),
    ("<iframe>@<p>a</p></iframe>", &["raw-text-content"]),
    (
        "<div><keygen>@seeded</keygen></div>",
        &["void-element-content"],
    ),
    ("<span>@<div>a</div></span>", &["phrasing-content-expected"]),
    (
        "<a href=\"#\">a @<button>b</button></a>",
        &["interactive-content-nested"],
    ),
    ("<ul>@<div>a</div></ul>", &["child-not-permitted"]),
];

/// One expected finding, as a byte span of the seeded text.
#[derive(Clone, Debug)]
pub struct Expected {
    pub class: &'static str,
    pub rule: &'static str,
    pub start: usize,
    pub end: usize,
}

/// A seeded file: the text, the insertions made (original offset, inserted
/// length; ascending), and the findings the insertions must produce.
#[derive(Clone, Debug)]
pub struct HtmlSeed {
    pub seeded: String,
    pub inserts: Vec<(usize, usize)>,
    pub expected: Vec<Expected>,
    pub composed: bool,
}

/// Lines at column 0 with their byte offsets.
fn lines(source: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0;
    source.split_inclusive('\n').map(move |line| {
        let start = offset;
        offset += line.len();
        (start, line)
    })
}

/// Where the top-level `<template>` (no attributes) closes, or why the file
/// is not eligible.
fn template_close(source: &str) -> Result<usize, &'static str> {
    let openers: Vec<(usize, &str)> = lines(source)
        .filter(|(_, line)| line.starts_with("<template"))
        .collect();
    let [(open, line)] = openers.as_slice() else {
        return Err("no single top-level template");
    };
    if !line.starts_with("<template>") {
        return Err("template has attributes");
    }
    lines(source)
        .find(|(start, line)| start > open && line.starts_with("</template>"))
        .map(|(start, _)| start)
        .ok_or("template does not close at column 0")
}

/// The `>` ending the start tag at `start`, outside attribute quotes (a
/// `generic="T extends Record<K, V>"` attribute contains `>`).
fn tag_end(source: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (index, byte) in source.bytes().enumerate().skip(start) {
        match (quote, byte) {
            (None, b'"' | b'\'') => quote = Some(byte),
            (Some(open), _) if open == byte => quote = None,
            (None, b'>') => return Some(index),
            _ => {}
        }
    }
    None
}

/// Where an import goes in the single top-level `<script setup>`, if any.
fn script_setup_insert(source: &str) -> Option<usize> {
    let setups: Vec<usize> = lines(source)
        .filter(|(_, line)| line.starts_with("<script"))
        .filter_map(|(start, _)| {
            let end = tag_end(source, start)?;
            let tag = &source[start..end];
            let setup = tag
                .split_whitespace()
                .any(|word| word == "setup" || word.starts_with("setup="));
            (setup && !tag.contains("src=")).then_some(end)
        })
        .collect();
    let [close] = setups.as_slice() else {
        return None;
    };
    let after = close + 1;
    Some(if source[after..].starts_with('\n') {
        after + 1
    } else {
        after
    })
}

/// Strip the `@` markers of `snippet`, returning the clean text and each
/// marked span relative to it.
fn unmark(snippet: &str) -> (String, Vec<(usize, usize)>) {
    let mut clean = String::with_capacity(snippet.len());
    let mut marks = Vec::new();
    for ch in snippet.chars() {
        if ch == '@' {
            marks.push(clean.len());
        } else {
            clean.push(ch);
        }
    }
    let spans = marks
        .into_iter()
        .map(|start| {
            let rest = &clean[start..];
            let end = if let Some(tag) = rest.strip_prefix('<') {
                start + 1 + tag.find([' ', '/', '>']).unwrap_or(tag.len())
            } else {
                start + rest.find('<').unwrap_or(rest.len())
            };
            (start, end)
        })
        .collect();
    (clean, spans)
}

/// Plan the seed of one file; `imported` is true when another source file
/// names it (see the module docs).
pub fn plan(source: &str, imported: bool) -> Result<HtmlSeed, &'static str> {
    if imported {
        return Err("imported by another file");
    }
    let close = template_close(source)?;
    let script = script_setup_insert(source).filter(|at| *at < source.len());
    let mut block = String::from("<div data-davinci-seed>\n");
    let mut marks: Vec<(&'static str, &'static str, usize, usize)> = Vec::new();
    let composed = script.is_some();
    let snippets = NESTING
        .iter()
        .map(|(snippet, classes)| (*snippet, *classes, NESTING_RULE));
    let composed_snippet = [(COMPOSED_SNIPPET, &[COMPOSED_CLASS][..], COMPOSED_RULE)];
    let chosen = snippets.chain(composed_snippet.into_iter().filter(|_| composed));
    for (snippet, classes, rule) in chosen {
        let (clean, spans) = unmark(snippet);
        assert_eq!(
            spans.len(),
            classes.len(),
            "seed snippet markers: {snippet}"
        );
        for (class, (start, end)) in classes.iter().zip(spans) {
            marks.push((class, rule, block.len() + start, block.len() + end));
        }
        block.push_str(&clean);
        block.push('\n');
    }
    block.push_str("</div>\n");

    let mut inserts: Vec<(usize, String)> = vec![(close, block)];
    if let Some(at) = script {
        inserts.push((at, BLOCK_IMPORT.to_string()));
    }
    inserts.sort_by_key(|(at, _)| *at);
    let mut seeded = String::with_capacity(source.len() + 2048);
    let mut cursor = 0;
    let mut block_start = 0;
    for (at, text) in &inserts {
        seeded.push_str(&source[cursor..*at]);
        if *at == close {
            block_start = seeded.len();
        }
        seeded.push_str(text);
        cursor = *at;
    }
    seeded.push_str(&source[cursor..]);
    let expected = marks
        .into_iter()
        .map(|(class, rule, start, end)| Expected {
            class,
            rule,
            start: block_start + start,
            end: block_start + end,
        })
        .collect();
    Ok(HtmlSeed {
        seeded,
        inserts: inserts.iter().map(|(at, text)| (*at, text.len())).collect(),
        expected,
        composed,
    })
}

/// Map an original byte offset into the seeded text.
pub fn shift(offset: usize, inserts: &[(usize, usize)], is_end: bool) -> usize {
    inserts
        .iter()
        .filter(|(at, _)| if is_end { offset > *at } else { offset >= *at })
        .map(|(_, len)| len)
        .sum::<usize>()
        + offset
}
