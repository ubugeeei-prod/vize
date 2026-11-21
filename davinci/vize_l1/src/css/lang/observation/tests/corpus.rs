extern crate std;

use super::{SourceBlock, SourceRoot, StyleLanguage, assert_same_style};
use crate::container::{ContainerFormat, Vue, lang::style_lang};
use davinci_test_support::corpus::{collect_vue_files, read_corpus_source};
use std::{
    fs,
    path::{Path, PathBuf},
    vec::Vec,
};
use vize_l0::Allocator;

#[derive(Debug, Default)]
struct Counts {
    files: usize,
    dialects: [usize; 5],
    external_blocks: usize,
    unknown_languages: usize,
}

fn compare(path: &Path, counts: &mut Counts) {
    let text = read_corpus_source(path);
    let root = SourceRoot::new(&text).unwrap();
    counts.files += 1;
    let mut check = |language: StyleLanguage, block: SourceBlock<'_>| {
        assert_same_style(language, block);
        let slot = match language {
            StyleLanguage::Css => 0,
            StyleLanguage::Scss => 1,
            StyleLanguage::Less => 2,
            StyleLanguage::Sass => 3,
            StyleLanguage::Stylus => 4,
        };
        counts.dialects[slot] += 1;
    };
    let extension = path.extension().and_then(|extension| extension.to_str());
    if extension != Some("vue") {
        check(style_lang(extension).unwrap(), root.whole_block());
        return;
    }
    let arena = Allocator::default();
    let container = Vue.split(&arena, &text);
    for block in container.blocks.iter().filter(|b| b.name == "style") {
        if block.attr("src").is_some() {
            counts.external_blocks += 1;
            continue;
        }
        let Ok(language) = style_lang(block.attr("lang").and_then(|attr| attr.value)) else {
            counts.unknown_languages += 1;
            continue;
        };
        let span = block.content;
        let slice = root
            .source()
            .get(span.start as usize..span.end as usize)
            .unwrap();
        check(language, root.block(slice, span.start).unwrap());
    }
}

#[test]
fn repository_style_blocks_match_structure_and_class_readers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for dir in [
        "tests/_fixtures",
        "tests/fixtures",
        "crates",
        "playground",
        "examples",
        "npm",
    ] {
        collect_vue_files(&root.join(dir), &mut files);
    }
    let mut counts = Counts::default();
    for file in files {
        compare(&file, &mut counts);
    }
    assert!(counts.files > 0 && counts.dialects.iter().sum::<usize>() > 0);
    std::eprintln!("{counts:?}");
}

#[test]
#[ignore = "NUL-delimited repository or pinned Vue and standalone style paths"]
fn external_style_corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = PathBuf::from(std::env::var_os("VIZE_STYLE_OBSERVATION_MANIFEST").unwrap());
    let paths = fs::read_to_string(manifest).unwrap();
    let mut counts = Counts::default();
    for path in paths.split_terminator('\0') {
        compare(&root.join(path), &mut counts);
    }
    assert!(counts.files > 0 && counts.dialects.iter().sum::<usize>() > 0);
    std::eprintln!("{counts:?}");
}
