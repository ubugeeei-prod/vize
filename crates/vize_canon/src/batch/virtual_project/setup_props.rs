use std::path::Path;

use vize_atelier_sfc::{SfcDescriptor, croquis::merge_resolved_props_into_croquis};

pub(super) fn augment_type_based_props_from_script_context(
    croquis: &mut vize_croquis::Croquis,
    descriptor: &SfcDescriptor<'_>,
    path: &Path,
) {
    let path_string = path.to_string_lossy();
    merge_resolved_props_into_croquis(croquis, descriptor, path_string.as_ref());
}
