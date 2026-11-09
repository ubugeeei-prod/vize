//! #6858: TSRX modules format their host TypeScript and keep every markup
//! region's bytes; modules with syntax errors are refused.
#![expect(
    clippy::disallowed_macros,
    clippy::expect_used,
    clippy::panic,
    reason = "tests assert by panicking; insta formats snapshot metadata"
)]

use std::path::{Path, PathBuf};

use vize_glyph::{FormatError, FormatOptions, format_tsrx};
use vize_l0::Allocator;
use vize_l1::markup::tsrx;

fn corpus(directory: &str) -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/tsrx");
    let mut files: Vec<PathBuf> = std::fs::read_dir(root.join(directory))
        .expect("TSRX corpus")
        .map(|entry| entry.expect("corpus entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "tsrx")
        })
        .collect();
    files.sort();
    files
}

fn format(source: &str) -> Result<vize_l0::String, FormatError> {
    format_tsrx(
        source,
        &FormatOptions::default(),
        &Allocator::default(),
        None,
    )
}

/// The text of every outermost region, in source order.
fn outer_regions(source: &str) -> Vec<vize_l0::String> {
    let allocator = Allocator::default();
    let module = tsrx::parse(&allocator, source);
    assert!(module.is_clean(), "formatted output must parse:\n{source}");
    let mut end = 0;
    let mut texts = Vec::new();
    for region in module.regions() {
        if region.span.start < end {
            continue;
        }
        end = region.span.end;
        let text = source.get(region.span.start as usize..end as usize);
        texts.push(text.expect("region on char boundaries").into());
    }
    texts
}

fn assert_formats(name: &str, source: &str) -> vize_l0::String {
    let formatted = format(source).unwrap_or_else(|error| panic!("{name}: {error}"));
    assert_eq!(
        outer_regions(&formatted),
        outer_regions(source),
        "{name}: markup regions must keep their bytes"
    );
    let again = format(&formatted).unwrap_or_else(|error| panic!("{name} repeat: {error}"));
    assert_eq!(again.as_str(), formatted.as_str(), "{name}: not idempotent");
    formatted
}

#[test]
fn every_valid_corpus_module_formats_idempotently_with_verbatim_markup() {
    let files = corpus(".");
    assert_eq!(files.len(), 18, "every valid corpus row must execute");
    for file in files {
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        let source = std::fs::read_to_string(&file).unwrap();
        let formatted = assert_formats(&name, &source);
        insta::with_settings!({ snapshot_suffix => name.clone() }, {
            insta::assert_snapshot!("corpus", formatted.as_str());
        });
    }
}

#[test]
fn every_error_corpus_module_is_refused() {
    let files = corpus("errors");
    assert_eq!(files.len(), 15, "every error corpus row must execute");
    for file in files {
        let source = std::fs::read_to_string(&file).unwrap();
        assert!(
            matches!(format(&source), Err(FormatError::ScriptParseError(_))),
            "{} must be refused",
            file.display()
        );
    }
}

#[test]
fn host_code_is_formatted_around_markup_and_its_comments() {
    let source = "import {ref} from 'vue'\n// leading\nexport const Note = ( ) => /* before */ <p>  keep   {  spacing  }  </p> // trailing\nconst items = [1,2].map((i)=><li>{ i }</li>)\nexport function List() @{\n  <ul>\n    // first\n    <li>a</li>{/* empty container */}\n  </ul>\n}\n";
    let formatted = assert_formats("inline", source);
    assert_eq!(
        formatted.as_str(),
        "import { ref } from \"vue\";\n// leading\nexport const Note = () => /* before */ <p>  keep   {  spacing  }  </p>; // trailing\nconst items = [1, 2].map((i) => <li>{ i }</li>);\nexport function List() @{\n  <ul>\n    // first\n    <li>a</li>{/* empty container */}\n  </ul>\n}\n"
    );
}

#[test]
fn a_formatted_module_is_a_byte_identical_fixed_point() {
    let source = "export function Counter({ start }: { start: number }) @{\n  let count = start;\n  <button onClick={() => count++}>{count}</button>\n}\n";
    assert_eq!(assert_formats("fixed point", source).as_str(), source);
}

#[test]
fn multiline_markup_in_host_expressions_keeps_its_bytes() {
    let source = "function view(items: string[]) {\n  return <ul>\n    {items.map((item) => <li>{item}</li>)}\n  </ul>\n}\nconst label = true ? <b>\n  yes\n</b> : null\n";
    insta::assert_snapshot!("multiline", assert_formats("multiline", source).as_str());
}
