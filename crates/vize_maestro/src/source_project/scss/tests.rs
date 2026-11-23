use super::*;
use crate::source_project::SourceSnapshotCache;
use std::path::Path;
use tower_lsp::lsp_types::Url;
use vize_l0::{
    Allocator, SourceRoot,
    config::{VueDialect, VueVersion},
    line_index::LineBreaks,
};
use vize_l1::container::vue::{DescriptorOptions, Vue};

fn vue() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: Default::default(),
    }
}

fn position(text: &str, offset: usize) -> Position {
    let (line, character) = LineBreaks::Lsp.offset_to_position(text, offset);
    Position::new(line, character)
}

#[test]
fn scss_definition_publishes_only_the_current_source_with_checked_utf16_ranges() {
    let source = "<template>😀</template>\r\n<style lang='scss'>$old_name:red; .c{color:$old-name} $old-name:blue;.d{x:$old_name}</style>";
    let uri = Url::parse("file:///Theme.vue").unwrap();
    let documents = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    documents.open(uri.clone(), source.into(), 3, "vue".into());
    let snapshot = cache.capture(&documents, &uri).unwrap();
    let arena = Allocator::default();
    let surface = Vue.observe_descriptor(&arena, snapshot.source(), vue());
    let style = surface.admitted().unwrap().styles().next().unwrap();
    let products = ScssProducts::from_style(style).unwrap();
    for (reference, declaration) in [("$old-name}", "$old_name:"), ("$old_name}", "$old-name:")] {
        let cursor = position(source, source.find(reference).unwrap() + 1);
        let at = source.find(declaration).unwrap();
        let expected = Location::new(
            uri.clone(),
            Range::new(
                position(source, at),
                position(source, at + declaration.len() - 1),
            ),
        );
        assert_eq!(
            snapshot
                .scss_definition(&documents, &products, cursor)
                .unwrap(),
            Some(GotoDefinitionResponse::Scalar(expected))
        );
    }
    for cursor in [
        Position::new(0, 11),
        Position::new(30, 0),
        Position::new(1, 0),
    ] {
        assert_eq!(
            snapshot
                .scss_definition(&documents, &products, cursor)
                .unwrap(),
            None
        );
    }
    let same_bytes = snapshot.source().to_owned();
    let foreign = ScssProducts::read(SourceRoot::new(&same_bytes).unwrap().whole_block());
    assert_eq!(
        snapshot.scss_definition(&documents, &foreign, Position::new(0, 0)),
        Err(ScssDefinitionError::ForeignSource)
    );
    documents.close(&uri);
    assert_eq!(
        snapshot.scss_definition(&documents, &products, Position::new(0, 0)),
        Err(ScssDefinitionError::Host(SnapshotRefusal::MissingDocument))
    );
    documents.open(uri.clone(), source.into(), 3, "vue".into());
    assert_eq!(
        snapshot.scss_definition(&documents, &products, Position::new(0, 0)),
        Err(ScssDefinitionError::Host(SnapshotRefusal::Superseded))
    );
    let fresh = cache.capture(&documents, &uri).unwrap();
    assert_eq!(
        fresh.scss_definition(&documents, &products, Position::new(0, 0)),
        Err(ScssDefinitionError::ForeignSource)
    );
}

/// Independent source oracle for the admitted root variable declarations.
/// An assignment becomes visible after its complete right-hand side.
fn expected_definition(text: &str, use_start: usize, name: &str) -> Option<(usize, usize)> {
    let prefix = text.get(..use_start)?;
    prefix
        .match_indices('$')
        .filter_map(|(start, _)| {
            let tail = prefix.get(start + 1..)?;
            let len = tail
                .bytes()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
                .count();
            let spelling = tail.get(..len)?;
            let remaining = tail.get(len..)?.trim_ascii_start().strip_prefix(':')?;
            if remaining.contains(';') && spelling.replace('_', "-") == name.replace('_', "-") {
                Some((start, start + len + 1))
            } else {
                None
            }
        })
        .last()
}

#[test]
#[ignore = "all original SCSS editor variable references in tracked Vue files"]
fn every_original_scss_variable_use_has_the_authored_lsp_location() {
    let inputs = std::fs::read_to_string(std::env::var_os("VIZE_SCSS_MANIFEST").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let mut styles = 0;
    let mut references = 0;
    let documents = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    for file in inputs.split_terminator('\0') {
        if file.ends_with(".scss") {
            continue;
        }
        let path = root.join(file);
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        if !source.to_ascii_lowercase().contains("scss") {
            continue;
        }
        let uri = Url::from_file_path(&path).unwrap();
        documents.open(uri.clone(), source, 1, "vue".into());
        let snapshot = cache.capture(&documents, &uri).unwrap();
        let arena = Allocator::default();
        let desc = Vue.observe_descriptor(&arena, snapshot.source(), vue());
        if let Ok(view) = desc.admitted() {
            for style in view.styles() {
                let Ok(products) = ScssProducts::from_style(style) else {
                    continue;
                };
                let Ok(program) = products.program() else {
                    continue;
                };
                styles += 1;
                let block = products.block();
                let base = block.start() as usize;
                for use_ in program.variable_uses() {
                    let source = snapshot.source();
                    let name = use_.span.slice(source).strip_prefix('$').unwrap();
                    let (start, end) =
                        expected_definition(block.source(), use_.span.start as usize - base, name)
                            .unwrap();
                    let expected = Location::new(
                        uri.clone(),
                        Range::new(position(source, base + start), position(source, base + end)),
                    );
                    for cursor in [use_.span.start, use_.span.end - 1] {
                        assert_eq!(
                            snapshot
                                .scss_definition(
                                    &documents,
                                    &products,
                                    position(source, cursor as usize)
                                )
                                .unwrap(),
                            Some(GotoDefinitionResponse::Scalar(expected.clone())),
                            "{file}"
                        );
                    }
                    references += 1;
                }
            }
        }
        documents.close(&uri);
        cache.forget(&uri);
    }
    if std::env::var_os("VIZE_REQUIRE_REPOSITORY_SCSS").is_some() {
        assert!(
            styles >= 11 && references > 0,
            "styles={styles} references={references}"
        );
    }
    eprintln!("scss_editor original_styles={styles} references={references}");
}
