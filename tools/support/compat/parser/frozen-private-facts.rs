//! Audit-only child module, copied identically into both immutable sources.
//! It exposes no production API or constructor and changes no parser body.
use std::{env, fs, io::Write, path::PathBuf};
use oxc_allocator::Allocator;
use oxc_span::SourceType;
use crate::{ParseOptions, Parser, config::RuntimeParserConfig};

#[test]
fn observe_private_source_facts() {
    let directory = PathBuf::from(env::var_os("VIZE_EXPRESSION_OVERLAP_CASES").unwrap());
    let mut cases = fs::read_dir(directory).unwrap().map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "input"))
        .collect::<Vec<_>>();
    cases.sort();
    let mut output = fs::File::create(env::var_os("VIZE_EXPRESSION_OVERLAP_FACTS").unwrap()).unwrap();
    for path in cases {
        let source = fs::read_to_string(&path).unwrap();
        for source_type in [SourceType::ts(), SourceType::tsx(), SourceType::jsx(), SourceType::ts().with_script(true), SourceType::ts().with_unambiguous(true)] {
            for tokens in [false, true] {
                for parse_regular_expression in [false, true] {
                    let allocator = Allocator::default();
                    let options = ParseOptions { parse_regular_expression, ..ParseOptions::default() };
                    let result = Parser::new(&allocator, &source, source_type)
                        .with_config(RuntimeParserConfig::new(tokens)).with_options(options).parse();
                    writeln!(output, "{} {source_type:?} tokens={tokens} regexp={parse_regular_expression}: {:?}",
                        path.file_name().unwrap().to_str().unwrap(),
                        (result.panicked, result.is_flow_language, result.has_legacy_literals, result.has_jsdoc_comments),
                    ).unwrap();
                }
            }
        }
    }
}
