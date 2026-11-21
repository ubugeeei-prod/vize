//! Compare every borrowed artifact with a clean lowering and with the existing
//! owned resident read. Style input expectations use the original descriptor.
use super::*;
use crate::{StageConfig, compute_file_artifacts, descriptor::parse_outcome};
use std::{fs, path::Path};

#[derive(Debug, Default)]
struct Counts {
    accepted: usize,
    rejected: usize,
    non_utf8: usize,
    blocks: usize,
    surfaces: usize,
    pages: usize,
    positions: usize,
    styles: usize,
    modules: usize,
}

#[expect(
    clippy::expect_used,
    reason = "the original accepted these corpus inputs and spans"
)]
fn compare(
    db: &mut ResidentDatabase,
    file: SourceFile,
    path: &str,
    source: &str,
    counts: &mut Counts,
) {
    db.rename(file, path);
    db.edit(file, source);
    let original = match parse_outcome(path, source) {
        ParsedSfc::Descriptor(descriptor) => descriptor,
        ParsedSfc::Failed(error) => {
            assert_eq!(db.document_blocks(file).err(), Some(&error), "{path}");
            counts.rejected += 1;
            return;
        }
    };
    let clean = compute_file_artifacts(source, StageConfig::default());
    assert_eq!(db.file_artifacts(file), clean, "{path}");
    let view = db
        .document_blocks(file)
        .expect("original descriptor admitted this document");
    assert_eq!(view.iter().len(), clean.len(), "{path}");
    counts.accepted += 1;
    counts.blocks += clean.len();
    for (block, fresh) in view.iter().zip(&clean) {
        assert_eq!(
            (block.kind(), block.ordinal(), block.start()),
            (&fresh.kind, fresh.ordinal, fresh.start),
            "{path}"
        );
        assert_eq!(block.source().key, fresh.source_key, "{path}");
        assert_eq!(block.surface(), fresh.surface.as_ref(), "{path}");
        assert_eq!(block.page(), fresh.page.as_ref(), "{path}");
        counts.surfaces += usize::from(block.surface().is_some());
        counts.pages += usize::from(block.page().is_some());

        let len = u32::try_from(block.source().text.len()).expect("block fits source coordinates");
        let span = block
            .authored_span(Span::new(0, len))
            .expect("whole authored block");
        assert_eq!(span.slice(source), block.source().text.as_str(), "{path}");
        for byte in [span.start, span.end.saturating_sub(1), span.end] {
            if !source.is_char_boundary(byte as usize) {
                assert_eq!(
                    view.at(byte).err(),
                    Some(BlockPositionError::NotCharBoundary),
                    "{path}"
                );
            } else {
                let expected = view.iter().find(|row| {
                    byte.checked_sub(row.start())
                        .is_some_and(|rel| (rel as usize) < row.source().text.len())
                });
                assert_eq!(
                    view.at(byte).expect("valid boundary").map(BlockView::start),
                    expected.map(BlockView::start),
                    "{path} at {byte}"
                );
            }
            counts.positions += 1;
        }
        if block.kind() != &BlockKind::Style {
            assert!(block.style_module().is_none(), "{path}");
            continue;
        }

        let style = original
            .styles
            .get(block.ordinal() as usize)
            .expect("original style with this ordinal");
        assert_eq!(span.start as usize, style.loc.start, "{path}");
        assert_eq!(span.end as usize, style.loc.end, "{path}");
        assert_eq!(block.style_module(), style.module.as_deref(), "{path}");
        counts.styles += 1;
        counts.modules += usize::from(style.module.is_some());
        assert_eq!(
            block.source().text.as_str(),
            style.content.as_ref(),
            "{path}"
        );
        assert_eq!(block.source().attr("lang"), style.lang.as_deref(), "{path}");
        assert_eq!(block.source().attr("src"), style.src.as_deref(), "{path}");
    }
}

#[test]
#[ignore = "set VIZE_BLOCK_QUERY_MANIFEST to a NUL-delimited list of original Vue files"]
fn original_documents() {
    let list = fs::read(std::env::var_os("VIZE_BLOCK_QUERY_MANIFEST").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut counts = Counts::default();
    let mut db = ResidentDatabase::default();
    let file = db.open("Initial.vue", "");
    for raw in list
        .split(|&byte| byte == 0)
        .filter(|path| !path.is_empty())
    {
        let path = core::str::from_utf8(raw).unwrap();
        let bytes = fs::read(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"));
        if let Ok(text) = core::str::from_utf8(&bytes) {
            compare(&mut db, file, path, text, &mut counts);
        } else {
            eprintln!("non UTF-8 source: {path}");
            counts.non_utf8 += 1;
        }
    }
    assert!(counts.accepted > 0 && counts.styles > 0 && counts.pages > 0);
    eprintln!("{counts:?}");
}
