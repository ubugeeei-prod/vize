#![expect(
    clippy::expect_used,
    reason = "the benchmark owns its fixture, sizes and offsets"
)]

use std::hint::black_box;
use vize_incremental::{
    PageArtifact, ResidentDatabase, SourceFile, SurfaceArtifact, db::sfc_blocks,
};
use vize_l0::String;

pub fn source(rows: usize) -> String {
    use std::fmt::Write;
    let mut result = String::from("<script setup>const value = 1</script><template><main>");
    for i in 0..rows {
        write!(
            &mut result,
            "<section class='c{i}'>{{{{ value }}}}</section>"
        )
        .expect("write benchmark fixture");
    }
    result.push_str("</main></template>");
    for i in 0..rows {
        write!(
            &mut result,
            "<style module='theme' lang='scss'>.c{i} {{ &-label {{color:red}} }}</style>"
        )
        .expect("write benchmark fixture");
    }
    result
}

#[derive(Clone, Copy, Debug)]
pub enum Work {
    Pages,
    Positions,
}

fn artifacts(surface: Option<&SurfaceArtifact>, page: Option<&PageArtifact>) -> usize {
    black_box(surface).map_or(0, |s| s.findings.len() + 1)
        + black_box(page).map_or(0, |p| p.diagnostics.len() + 1)
}

impl Work {
    pub fn original(self, db: &ResidentDatabase, file: SourceFile, bytes: &[u32]) -> usize {
        match self {
            Self::Pages => db
                .file_artifacts(file)
                .iter()
                .map(|b| {
                    b.start as usize
                        + b.ordinal as usize
                        + b.kind.tag().len()
                        + artifacts(b.surface.as_ref(), b.page.as_ref())
                })
                .sum(),
            Self::Positions => {
                let source = file.text(db);
                let rows = sfc_blocks(db, file);
                bytes
                    .iter()
                    .filter_map(|&byte| {
                        assert!(source.is_char_boundary(byte as usize));
                        rows.iter()
                            .copied()
                            .find(|block| {
                                byte.checked_sub(block.start(db))
                                    .is_some_and(|rel| (rel as usize) < block.source(db).text.len())
                            })
                            .map(|block| block.start(db) as usize)
                    })
                    .sum()
            }
        }
    }

    pub fn candidate(self, db: &ResidentDatabase, file: SourceFile, bytes: &[u32]) -> usize {
        let view = db.document_blocks(file).expect("admitted fixture");
        match self {
            Self::Pages => view
                .iter()
                .map(|b| {
                    b.start() as usize
                        + b.ordinal() as usize
                        + b.kind().tag().len()
                        + artifacts(b.surface(), b.page())
                })
                .sum(),
            Self::Positions => bytes
                .iter()
                .filter_map(|&byte| view.at(byte).expect("benchmark offset"))
                .map(|block| block.start() as usize)
                .sum(),
        }
    }
}

pub fn offsets(db: &ResidentDatabase, file: SourceFile) -> [u32; 3] {
    let text = file.text(db);
    let first = text.find("const value").expect("script offset");
    let middle = text.find("</template>").expect("template closing tag");
    let last = text.rfind("color:red").expect("style offset");
    [first, middle, last].map(|i| u32::try_from(i).expect("fixture length"))
}
