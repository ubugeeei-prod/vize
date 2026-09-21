//! An admitted source never reaches the legacy parser, so admission must imply
//! that the legacy parser reports nothing for it. The corpus is every fixture
//! input, each of its prefixes and punctuation deletions (unterminated tags,
//! attributes, interpolations and comments, empty modifiers), and hand-written
//! malformed markup.

use super::{VaporS3BridgeStatus, lower_source_for_vapor, options};
use crate::compile::{VaporCompilerOptions, parser_options};
use vize_atelier_core::{
    TemplateSyntaxMode, options::CustomElementMatcher,
    parser::parse_with_options_custom_elements_and_template_syntax,
};
use vize_carton::{Allocator, String};

const MALFORMED: &[&str] = &[
    "<div a=\"1\" a=\"2\"></div>",
    "<div :a=\"x\" :a=\"y\"></div>",
    "<div></span>",
    "</div>",
    "<div><p></div>",
    "<br></br>",
    "{{ a",
    "<div>{{ a </div>",
    "<!-- open",
    "<!-- <!-- nested --> -->",
    "<!-- x --!>",
    "<![CDATA[x]]>",
    "<!DOCTYPE html>",
    "<?xml version=\"1.0\"?>",
    "</>",
    "< div></div>",
    "<1div></1div>",
    "<div a=\"b\"c=\"d\"></div>",
    "<div =a></div>",
    "<div a<b></div>",
    "<div \"a\"></div>",
    "<div a='b></div>",
    "<div a=b\"c></div>",
    "<div/ ></div>",
    "<div></div/>",
    "<div",
    "<div a",
    "<div a=",
    "<textarea>{{ a }}",
    "<template><div></div>",
    "<span>&#0;</span>",
    "<span>&#x110000;</span>",
    "<p>\u{0}</p>",
    "<div A=\"1\" a=\"2\"></div>",
    "<Comp A=\"1\" a=\"2\"></Comp>",
    "<slot name=\"x\" N=\"1\" n=\"2\"></slot>",
    "<div v-=\"x\"></div>",
    "<div :=\"x\"></div>",
    "<div @=\"x\"></div>",
    "<button @click.=\"x\"></button>",
    "<button @click..stop=\"x\"></button>",
    "<div :title.=\"x\"></div>",
    "<button><Comp><button>x</button></Comp></button>",
    "<button><span v-if=\"a\"><button>x</button></span></button>",
    "<button><b v-for=\"i in n\"><button>x</button></b></button>",
    "<button><slot><button>x</button></slot></button>",
    "<ul><li><Comp><li>x</li></Comp></li></ul>",
    "<ul><li><i v-if=\"a\"><li>x</li></i></li></ul>",
    "<ul><li><ul><li>x</li></ul></li></ul>",
    "<div><span/></div>",
    "<Comp><div/></Comp>",
    "<div v-if=\"a\"><span/></div>",
    "<Comp/>",
    "<slot/>",
    "<br/><hr/><img/><input/>",
    "<b><i></b></i>",
    "<b><div></b></div>",
];

#[test]
fn admitted_sources_are_clean_for_the_legacy_parser() {
    let mut disagreements = std::vec::Vec::new();
    let mut check = |source: &str| {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        if matches!(status, VaporS3BridgeStatus::Accepted(_)) && !legacy_errors(source).is_empty() {
            disagreements.push(String::from(source));
        }
    };
    for source in corpus() {
        check(&source);
        for (at, c) in source.char_indices() {
            check(&source[..at]);
            // Deleting markup punctuation breaks tags, attributes, directives
            // and delimiters; deleting a letter only renames something.
            if !c.is_alphanumeric() {
                let mut deleted = String::from(&source[..at]);
                deleted.push_str(&source[at + c.len_utf8()..]);
                check(&deleted);
            }
        }
    }
    assert_eq!(disagreements, std::vec::Vec::<String>::new());
}

/// The legacy parser flattens and reports elements nested 4096 deep; the
/// native lane stops well before that limit.
#[test]
fn deep_nesting_is_left_to_the_legacy_parser() {
    let admitted = |depth: usize| {
        let mut source = String::default();
        (0..depth).for_each(|_| source.push_str("<div>"));
        (0..depth).for_each(|_| source.push_str("</div>"));
        let allocator = Allocator::new();
        let admitted = vize_carton::ensure_sufficient_stack(|| {
            matches!(
                lower_source_for_vapor(&allocator, &source, options()),
                VaporS3BridgeStatus::Accepted(_)
            )
        });
        (admitted, legacy_errors(&source).is_empty())
    };
    assert_eq!(
        [admitted(1024), admitted(1025)],
        [(true, true), (false, true)]
    );
}

fn legacy_errors(source: &str) -> std::vec::Vec<vize_atelier_core::CompilerError> {
    let allocator = Allocator::new();
    parse_with_options_custom_elements_and_template_syntax(
        &allocator,
        source,
        parser_options(&VaporCompilerOptions::default()),
        CustomElementMatcher::default(),
        TemplateSyntaxMode::Standard,
    )
    .1
}

/// Every single-line fixture input in the shared compiler corpora.
fn corpus() -> std::vec::Vec<String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
    let mut sources: std::vec::Vec<String> = MALFORMED.iter().map(|s| String::from(*s)).collect();
    for dir in ["vapor", "vdom", "parser", "errors"] {
        let mut paths: std::vec::Vec<_> = std::fs::read_dir(root.join(dir))
            .unwrap_or_else(|error| panic!("{dir}: {error}"))
            .map(|entry| entry.expect("fixture entry").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "pkl"))
            .collect();
        paths.sort();
        for path in paths {
            let text = std::fs::read_to_string(&path).expect("fixture text");
            sources.extend(text.lines().filter_map(input));
        }
    }
    sources
}

/// The `input = …` value of one Pkl line: a `#"…"#` raw string or a plain
/// string with Pkl's single-character escapes. Multi-line strings are skipped.
fn input(line: &str) -> Option<String> {
    let value = line.trim_start().strip_prefix("input = ")?;
    if let Some(raw) = value.strip_prefix("#\"") {
        return raw.strip_suffix("\"#").map(String::from);
    }
    let body = value
        .strip_prefix('"')
        .filter(|body| !body.starts_with("\"\""))?;
    let mut out = String::default();
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => out.push(match chars.next()? {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                escaped => escaped,
            }),
            c => out.push(c),
        }
    }
    None
}
