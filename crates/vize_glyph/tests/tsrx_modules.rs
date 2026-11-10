//! #12524: TSRX modules format host TypeScript and every L1 markup region;
//! modules with syntax errors or a structural rewrite are refused.
#![expect(
    clippy::disallowed_macros,
    clippy::expect_used,
    clippy::panic,
    reason = "tests assert by panicking; insta formats snapshot metadata"
)]

use std::path::{Path, PathBuf};

use vize_glyph::{EndOfLine, FormatError, FormatOptions, format_tsrx};
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

fn assert_clean(source: &str) {
    let allocator = Allocator::default();
    let module = tsrx::parse(&allocator, source);
    assert!(module.is_clean(), "formatted output must parse:\n{source}");
}

fn assert_formats(name: &str, source: &str) -> vize_l0::String {
    let formatted = format(source).unwrap_or_else(|error| panic!("{name}: {error}"));
    assert_clean(&formatted);
    let again = format(&formatted).unwrap_or_else(|error| panic!("{name} repeat: {error}"));
    assert_eq!(again.as_str(), formatted.as_str(), "{name}: not idempotent");
    formatted
}

#[test]
fn every_valid_corpus_module_has_a_reviewed_idempotent_layout() {
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
        "import { ref } from \"vue\";\n// leading\nexport const Note = () => /* before */ <p>keep   {spacing}</p>; // trailing\nconst items = [1, 2].map((i) => <li>{i}</li>);\nexport function List() @{\n  <ul>\n    // first\n    <li>a</li>\n    {/* empty container */}\n  </ul>\n}\n"
    );
}

#[test]
fn a_canonical_module_is_a_byte_identical_fixed_point() {
    let source = "export function Counter({ start }: { start: number }) @{\n  let count = start;\n  <button onClick={() => count++}>{count}</button>\n}\n";
    let formatted = assert_formats("fixed point", source);
    assert_eq!(
        formatted.as_str(),
        "export function Counter({ start }: { start: number }) @{\n  let count = start;\n\n  <button onClick={() => count++}>{count}</button>\n}\n"
    );
}

#[test]
fn multiline_markup_in_host_expressions_is_reindented() {
    let source = "function view(items: string[]) {\n  return <ul>\n    {items.map((item) => <li>{item}</li>)}\n  </ul>\n}\nconst label = true ? <b>\n  yes\n</b> : null\n";
    insta::assert_snapshot!("multiline", assert_formats("multiline", source).as_str());
}

#[test]
fn auto_line_endings_apply_to_host_and_markup() {
    let source = "export function A() @{\r\n\t<div>\r\n\t\t<span>one</span>\r\n\t\t<span>two</span>\r\n\t</div>\r\n}\r\n";
    let options = FormatOptions {
        end_of_line: EndOfLine::Auto,
        ..FormatOptions::default()
    };
    let formatted = format_tsrx(source, &options, &Allocator::default(), None).unwrap();
    assert!(!formatted.replace("\r\n", "").contains(['\r', '\n']));
    let again = format_tsrx(formatted.as_str(), &options, &Allocator::default(), None).unwrap();
    assert_eq!(again, formatted);
}

#[test]
fn directives_attributes_and_setup_statements_use_canonical_layout() {
    let source = "export function View({items,show}:{items:string[],show:boolean}) @{ const total=items.length; <section data-total={ total } aria-label=\"items\" class=\"list\">@if(show){<ul>@for(const item of items;key item){<li title={ item }>{ item }</li>}</ul>}@else{<p> empty </p>}@try{<p>ready</p>}@pending{<p>wait</p>}@catch(error,reset){<button onClick={ reset }>{ error.message }</button>}</section> }";
    let formatted = assert_formats("directives", source);
    insta::assert_snapshot!("directives", formatted.as_str());
}
