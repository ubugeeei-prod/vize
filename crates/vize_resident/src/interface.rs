//! Production interface publication alongside the resident SFC descriptor.
//!
//! The host exports Croquis facts only after source, filename or project
//! configuration changes. Publication occurs outside Salsa queries; readers
//! observe a source and its fresh alpha pages together. Configuration changes
//! invalidate every export, with lazy refresh before its next consumer read.

use vize_davinci::summary::{AlphaPages, Facet, SfcSummary};
use vize_l0::{FxHashMap, String, cstr};

use crate::{ResidentDocuments, SharedDescriptor, SummaryInput, sfc_summary};

mod surface;
pub use surface::{ComponentSurface, component_surface};

#[derive(Default)]
pub(crate) struct Interfaces {
    entries: FxHashMap<String, Entry>,
    configuration: String,
    generation: u64,
    stats: InterfaceStats,
}

struct Entry {
    input: SummaryInput,
    filename: String,
    generation: u64,
}

impl Interfaces {
    pub(crate) fn remove(&mut self, key: &str) -> Option<SummaryInput> {
        self.entries.remove(key).map(|entry| entry.input)
    }
}

/// Publication and consumer executions since the last interval.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InterfaceStats {
    /// Host Croquis exports computed.
    pub exports: u32,
    /// Summary query bodies executed.
    pub summaries: u32,
    /// Summary query values reused.
    pub reused: u32,
    /// Production metadata consumer query bodies executed.
    pub consumers: u32,
    /// Production metadata consumer values reused.
    pub consumer_reuses: u32,
}

impl ResidentDocuments {
    /// Invalidate editor-source exports at LOW durability. The next provider
    /// read must publish against this source snapshot before returning facts.
    pub fn set_source_world_revision(&mut self, revision: u64) {
        use salsa::{Durability, Setter as _};
        crate::summary::SourceWorld::get(&self.db)
            .set_revision(&mut self.db)
            .with_durability(Durability::LOW)
            .to(revision);
    }

    /// Read the current production interface. `export` is called only when
    /// the source, filename or project configuration changed. A parse or
    /// export rejection cannot return the previous revision's interface.
    pub fn interface(
        &mut self,
        key: &str,
        filename: &str,
        text: &str,
        configuration: &str,
        export: impl FnOnce(&SharedDescriptor) -> Option<AlphaPages>,
    ) -> Option<SfcSummary> {
        if self.interfaces.configuration.as_str() != configuration {
            self.interfaces.configuration = String::from(configuration);
            self.interfaces.generation = self.interfaces.generation.wrapping_add(1);
            self.db
                .configure_tsconfig(&cstr!("{}\0{}", configuration, self.interfaces.generation));
        }
        let file = self.file(key, filename, text);
        let previous = self.interfaces.entries.get(key);
        let needs_export = previous.is_none_or(|entry| {
            entry.filename.as_str() != filename
                || entry.generation != self.interfaces.generation
                || !crate::summary::alpha_is_current(&self.db, entry.input)
        });
        // Drain validation accounting before measuring the refreshed read.
        let _validation = self.db.take_accounting();
        if needs_export {
            let descriptor = self.descriptor(key, filename, text)?;
            let pages = export(&descriptor)?;
            self.interfaces.stats.exports = self.interfaces.stats.exports.saturating_add(1);
            let input = if let Some(entry) = self.interfaces.entries.get(key) {
                let input = entry.input;
                self.db.revise_alpha(input, pages);
                input
            } else {
                self.db.publish_alpha(file, pages)
            };
            self.interfaces.entries.insert(
                String::from(key),
                Entry {
                    input,
                    filename: String::from(filename),
                    generation: self.interfaces.generation,
                },
            );
        }
        let input = self.interfaces.entries.get(key)?.input;
        let result = sfc_summary(&self.db, input).as_ref().ok().cloned();
        if let Some(counts) = self.db.take_accounting().get("sfc_summary") {
            self.interfaces.stats.summaries = self
                .interfaces
                .stats
                .summaries
                .saturating_add(counts.executed);
            self.interfaces.stats.reused =
                self.interfaces.stats.reused.saturating_add(counts.reused);
        }
        result
    }

    /// Fingerprint only the declarations this consumer uses. An absent
    /// declaration is absent in the current revision, never an old result.
    pub fn interface_fingerprint(
        &self,
        key: &str,
        facet: Facet,
        name: &str,
    ) -> Option<vize_davinci::summary::Fingerprint> {
        let input = self.interfaces.entries.get(key)?.input;
        crate::declaration_fingerprint(&self.db, input, facet, self.db.declaration_name(name))
    }

    /// Read and reset the production interface interval.
    pub fn take_interface_stats(&mut self) -> InterfaceStats {
        core::mem::take(&mut self.interfaces.stats)
    }

    /// Project/dependency notifications invalidate exports even when a
    /// document's bytes did not change. Refresh remains lazy per document.
    pub fn invalidate_interfaces(&mut self) {
        self.interfaces.generation = self.interfaces.generation.wrapping_add(1);
        self.db.configure_tsconfig(&cstr!(
            "{}\0{}",
            self.interfaces.configuration,
            self.interfaces.generation
        ));
    }
}

pub(crate) fn release_alpha(db: &mut crate::ResidentDatabase, input: SummaryInput) {
    db.revise_alpha(
        input,
        AlphaPages {
            signature: vize_davinci::summary::Signature {
                name: String::from("closed"),
                params: String::default(),
            },
            props: vec![],
            emits: vec![],
            slots: vec![],
            reactivity: vec![],
            components: vec![],
        },
    );
    let _released = sfc_summary(db, input);
}
