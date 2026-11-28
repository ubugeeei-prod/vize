#![expect(
    clippy::unwrap_used,
    reason = "test inputs and source ranges are asserted"
)]
use super::*;
use crate::script_parse::collect_script_parse_diagnostics;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::{Allocator, SourceRoot};

#[test]
fn incompatible_editor_grammar_refuses_even_when_the_source_has_no_errors() {
    let arena = Allocator::default();
    let source = "export const value = 42";
    let syntax = AuthoredProgram::parse(
        &arena,
        SourceRoot::new(source).unwrap().whole_block(),
        SourceType::mjs(),
    );
    for lang in [None, Some("js"), Some("ts"), Some("unknown")] {
        assert_eq!(
            ScriptDiagnosticProfile::for_language(lang).read(&syntax),
            Err(ScriptDiagnosticRefusal::DifferentGrammar)
        );
    }
    assert_eq!(
        ScriptDiagnosticProfile::for_language(Some(" tsx ")).source_type(),
        SourceType::tsx()
    );
}

#[test]
#[ignore = "requires the repository or external original Vue source manifest"]
fn every_original_script_parser_diagnostic_matches_at_its_authored_offset() {
    let manifest = std::fs::read(std::env::var_os("VIZE_SCRIPT_QUERY_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut files, mut descriptors, mut blocks, mut findings, mut invalid) = (0, 0, 0, 0, 0);
    for entry in manifest
        .split(|&byte| byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        let name = std::str::from_utf8(entry).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        files += 1;
        let Ok(descriptor) = parse_sfc(source, SfcParseOptions::default()) else {
            continue;
        };
        descriptors += 1;
        let host = SourceRoot::new(source).unwrap();
        for block in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
            .into_iter()
            .flatten()
        {
            let lang = block.lang.as_deref();
            let start = u32::try_from(block.loc.start).unwrap();
            let text = source.get(block.loc.start..block.loc.end).unwrap();
            assert_eq!(text, block.content.as_ref(), "{name}");
            let expected = collect_script_parse_diagnostics(None, &block.content, start, lang);
            let profile = ScriptDiagnosticProfile::for_language(lang);
            let arena = Allocator::default();
            let syntax = AuthoredProgram::parse(
                &arena,
                host.block(text, start).unwrap(),
                profile.source_type(),
            );
            assert_eq!(profile.read(&syntax).unwrap(), expected, "{name}");
            findings += expected.len();
            blocks += 1;
        }
    }
    assert!(blocks > 0);
    eprintln!(
        "checker_original_script files={files} descriptors={descriptors} blocks={blocks} findings={findings} invalid={invalid}"
    );
}
