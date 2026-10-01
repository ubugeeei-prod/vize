//! Helper imports built only after every body fragment has been appended.

use super::AssemblyError;
use crate::runtime::{HelperSet, Vocabulary};
use crate::write::{LinkSink, Writer};

pub(super) fn preamble<L: LinkSink>(
    vocabulary: &Vocabulary,
    helpers: &HelperSet,
) -> Result<Writer<L>, AssemblyError> {
    for &helper in helpers.in_use_order() {
        vocabulary
            .name(helper)
            .ok_or(AssemblyError::UnknownHelper(helper))?;
    }
    let mut writer = Writer::default();
    let mut base = 0;
    for module in vocabulary.modules {
        let end = base + module.names.len();
        let mut selected = helpers.in_use_order().iter().filter_map(|helper| {
            let index = usize::from(helper.index());
            index
                .checked_sub(base)
                .and_then(|local| module.names.get(local))
                .copied()
        });
        if let Some(first) = selected.next() {
            writer.push("import { ");
            append(&mut writer, first);
            for name in selected {
                writer.push(", ");
                append(&mut writer, name);
            }
            writer.push(" } from ");
            let specifier = serde_json::to_string(module.module)
                .map_err(|_| AssemblyError::InvalidModuleSpecifier)?;
            writer.push(&specifier);
            writer.push("\n");
        }
        base = end;
    }
    Ok(writer)
}

fn append<L: LinkSink>(writer: &mut Writer<L>, name: &str) {
    writer.push(name);
    writer.push(" as _");
    writer.push(name);
}
