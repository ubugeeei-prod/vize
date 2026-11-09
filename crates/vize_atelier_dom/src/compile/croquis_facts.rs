//! The `<script setup>` script tables projected onto the L2 binding table.
//!
//! The legacy transform lane reads attached [`ScriptTables`] in exactly two
//! places (`vize_atelier_core::lane::context`):
//!
//! - `reactive_source` decides the inline-mode `.value` / `_unref` reads in
//!   the identifier collector and the simple-identifier fallback. That is
//!   projected as [`ReactiveRead`] facts, name for name, onto the table the
//!   L2 transform rewrite reads.
//! - `is_component_registered` promotes a lowercase non-native tag to a
//!   component. The L2 lane resolves components from the binding metadata
//!   alone, so the projection is exact only when the tables register nothing
//!   the metadata lacks; otherwise the compile stays on the legacy lane under
//!   the `croquis` reason.

use vize_atelier_core::lane::ScriptTables;
use vize_atelier_core::options::BindingMetadata;
use vize_l4::targets::dom::artifact::{BindingTable, ReactiveRead};

use crate::options::DomCompilerOptions;

/// Whether `options` carries script tables the L2 lane cannot reproduce
/// from binding metadata (P3-17). Projectable tables are admitted:
/// non-inline DOM reads the metadata table plus the reactive reads, and the
/// production-path oracle holds those compiles to the legacy module byte for
/// byte.
pub(in crate::compile) fn unprojectable_croquis(options: &DomCompilerOptions) -> bool {
    options
        .script_tables
        .as_ref()
        .is_some_and(|tables| !projectable(tables, options.binding_metadata.as_ref()))
}

/// The binding table the L2 emitter reads: the script's binding metadata
/// plus, when script tables are attached, their reactive reads.
pub(in crate::compile) fn l2_binding_table_for(
    options: &DomCompilerOptions,
) -> Option<BindingTable> {
    let table = super::stage_options::l2_binding_table(options.binding_metadata.as_ref())?;
    Some(match options.script_tables.as_ref() {
        Some(tables) => table.with_reactive_reads(reactive_reads(tables)),
        None => table,
    })
}

/// How an inline render closure reads each reactive binding, by name.
fn reactive_reads(tables: &ScriptTables) -> impl Iterator<Item = (&str, ReactiveRead)> {
    tables.reactive().map(|(name, source)| {
        let read = if source.reads_value() {
            ReactiveRead::Value
        } else {
            ReactiveRead::Direct
        };
        (name, read)
    })
}

/// Whether every legacy read of `tables` is reproducible from `metadata`
/// plus the projected reactive reads.
pub(super) fn projectable(tables: &ScriptTables, metadata: Option<&BindingMetadata>) -> bool {
    metadata.is_some_and(|metadata| tables.components_bound_in(metadata))
}

#[cfg(test)]
mod tests {
    use super::{projectable, reactive_reads};
    use vize_atelier_core::lane::{ReactiveSource, ScriptTables};
    use vize_atelier_core::options::{BindingMetadata, BindingType};
    use vize_l4::targets::dom::artifact::{BindingTable, ReactiveRead};

    #[test]
    fn reactive_reads_follow_the_last_source() {
        let tables = ScriptTables::new(
            [
                ("count", ReactiveSource::Ref),
                ("state", ReactiveSource::Reactive),
                ("count", ReactiveSource::Readonly),
                ("total", ReactiveSource::Computed),
            ],
            [],
        );
        let table = BindingTable::new([], [], true).with_reactive_reads(reactive_reads(&tables));
        assert_eq!(table.reactive_read("count"), Some(ReactiveRead::Direct));
        assert_eq!(table.reactive_read("state"), Some(ReactiveRead::Direct));
        assert_eq!(table.reactive_read("total"), Some(ReactiveRead::Value));
        assert_eq!(table.reactive_read("other"), None);
    }

    #[test]
    fn registration_beyond_the_metadata_is_not_projectable() {
        let mut metadata = BindingMetadata::default();
        metadata
            .bindings
            .insert("Child".into(), BindingType::SetupConst);
        let child = ScriptTables::new([], ["Child"]);
        assert!(projectable(&child, Some(&metadata)));
        assert!(!projectable(&child, None));

        let hidden = ScriptTables::new([], ["Child", "Hidden"]);
        assert!(!projectable(&hidden, Some(&metadata)));

        let drawn = ScriptTables::new([], ["child"]);
        assert!(!projectable(&drawn, Some(&metadata)));
    }
}
