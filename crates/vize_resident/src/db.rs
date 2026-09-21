//! The salsa database: inputs, the block firewall, and the stage queries.
//!
//! ```text
//! SourceFile.text (input, LOW) ──► sfc_blocks ──► Block { source, start }
//!                                                   │ (tracked fields)
//!                         Block.source ──► s1_block │
//!  ProjectConfig (input, HIGH) ─┐                   │
//!                         Block.source ──► s2_page ◄┘
//! ```
//!
//! `sfc_blocks` re-runs on every keystroke, but it only *re-creates* the
//! file's [`Block`]s: a block's identity is `(file, kind, ordinal)`, and its
//! content ([`BlockSource`], carrying the P5-1a S0 key) and position
//! (`start`, the S0 side table) are separate tracked fields. Salsa compares
//! each re-created field with the previous value and bumps only the ones
//! that changed, so an edit in one block leaves every other block's
//! `source` untouched and their `s1_block` / `s2_page` memos are reused
//! without running — backdating at the firewall. An edit above a block
//! moves only its `start`, which no stage query reads.

use salsa::{Database as _, Durability, Setter as _};
use vize_s0::String;

use crate::accounting::{Accounting, Recorder, tally};
use crate::artifact::{
    BlockArtifacts, BlockKind, BlockSource, PageArtifact, StageConfig, SurfaceArtifact,
    page_artifact, split_blocks, surface_artifact,
};

/// One open file: its path and its current text (an open buffer, so
/// [`Durability::LOW`]).
#[salsa::input(debug)]
pub struct SourceFile {
    /// The file's path, as the client names it.
    #[returns(ref)]
    pub path: String,
    /// The current buffer text.
    #[returns(ref)]
    pub text: String,
}

/// Project configuration the stages read ([`Durability::HIGH`]: it changes
/// rarely, so a keystroke never re-validates what depends only on it).
#[salsa::input(singleton, debug)]
pub struct ProjectConfig {
    /// The stage-relevant configuration.
    #[returns(copy)]
    pub stage: StageConfig,
}

/// One SFC block of one file. Identity: `(file, kind, ordinal)`; content and
/// position are tracked separately so each invalidates only its readers.
#[salsa::tracked(debug)]
pub struct Block<'db> {
    /// The file the block belongs to.
    pub file: SourceFile,
    /// The block kind.
    #[returns(ref)]
    pub kind: BlockKind,
    /// Index among the file's blocks of the same kind.
    #[returns(copy)]
    pub ordinal: u32,
    /// The block's content and S0 key — the firewall every stage reads.
    #[tracked]
    #[returns(ref)]
    pub source: BlockSource,
    /// File-absolute start of the content — the S0 side table.
    #[tracked]
    #[returns(copy)]
    pub start: u32,
}

/// The file's blocks, in document order.
#[salsa::tracked(returns(ref))]
pub fn sfc_blocks(db: &dyn salsa::Database, file: SourceFile) -> Vec<Block<'_>> {
    split_blocks(file.text(db).as_str())
        .into_iter()
        .map(|slot| Block::new(db, file, slot.kind, slot.ordinal, slot.source, slot.start))
        .collect()
}

/// The block's S1 artifact (`Some` for an HTML template block).
#[salsa::tracked(returns(ref))]
pub fn s1_block<'db>(db: &'db dyn salsa::Database, block: Block<'db>) -> Option<SurfaceArtifact> {
    surface_artifact(block.source(db))
}

/// The block's S2 artifact (`Some` for template and style blocks), with
/// block-relative spans.
#[salsa::tracked(returns(ref))]
pub fn s2_page<'db>(db: &'db dyn salsa::Database, block: Block<'db>) -> Option<PageArtifact> {
    page_artifact(block.source(db), ProjectConfig::get(db).stage(db))
}

/// The resident tier's database.
#[salsa::db]
#[derive(Clone)]
pub struct ResidentDatabase {
    storage: salsa::Storage<Self>,
    recorder: Recorder,
}

#[salsa::db]
impl salsa::Database for ResidentDatabase {}

impl Default for ResidentDatabase {
    fn default() -> Self {
        Self::new(StageConfig::default())
    }
}

impl ResidentDatabase {
    /// A database whose project configuration starts as `config`.
    #[must_use]
    pub fn new(config: StageConfig) -> Self {
        let recorder = Recorder::default();
        let db = Self {
            storage: salsa::Storage::new(Some(recorder.callback())),
            recorder,
        };
        // The singleton is reached through `ProjectConfig::get` from here on.
        let _config: ProjectConfig = ProjectConfig::builder(config)
            .durability(Durability::HIGH)
            .new(&db);
        db
    }

    /// Open a file with `text` as its buffer.
    pub fn open(&self, path: &str, text: &str) -> SourceFile {
        SourceFile::builder(String::from(path), String::from(text))
            .durability(Durability::LOW)
            .new(self)
    }

    /// Replace `file`'s buffer with `text` (a new revision).
    pub fn edit(&mut self, file: SourceFile, text: &str) {
        file.set_text(self)
            .with_durability(Durability::LOW)
            .to(String::from(text));
    }

    /// Replace the project configuration (a new revision).
    pub fn configure(&mut self, config: StageConfig) {
        let project = ProjectConfig::get(self);
        project
            .set_stage(self)
            .with_durability(Durability::HIGH)
            .to(config);
    }

    /// Every artifact of every block of `file`, read through the queries —
    /// the incremental side of TS-42 (the clean side is
    /// [`compute_file_artifacts`](crate::compute_file_artifacts)).
    #[must_use]
    pub fn file_artifacts(&self, file: SourceFile) -> Vec<BlockArtifacts> {
        sfc_blocks(self, file)
            .iter()
            .map(|&block| BlockArtifacts {
                kind: block.kind(self).clone(),
                ordinal: block.ordinal(self),
                start: block.start(self),
                source_key: block.source(self).key,
                surface: s1_block(self, block).clone(),
                page: s2_page(self, block).clone(),
            })
            .collect()
    }

    /// Executions and reuses per query since the last call (TS-46).
    #[must_use]
    pub fn take_accounting(&self) -> Accounting {
        tally(self.recorder.drain(), |ingredient| {
            String::from(self.ingredient_debug_name(ingredient).as_ref())
        })
    }
}
