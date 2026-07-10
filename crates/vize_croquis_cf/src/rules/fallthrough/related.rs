use crate::diagnostics::CrossFileDiagnostic;
use crate::registry::FileId;
use vize_carton::{CompactString, cstr};

pub(super) struct FallthroughUsageRelated {
    pub(super) parent_file_id: FileId,
    pub(super) attr_name: CompactString,
    pub(super) name_is_dynamic: bool,
    pub(super) standard_html_attr: bool,
    pub(super) report_as_unused: bool,
    pub(super) source_start: u32,
    pub(super) component_name: CompactString,
}

impl FallthroughUsageRelated {
    pub(super) fn display_name(&self) -> CompactString {
        if self.name_is_dynamic {
            cstr!("[{}]", self.attr_name)
        } else {
            self.attr_name.clone()
        }
    }
}

pub(super) fn with_fallthrough_relateds(
    mut diagnostic: CrossFileDiagnostic,
    relateds: Option<&[FallthroughUsageRelated]>,
    attrs_filter: Option<&[CompactString]>,
) -> CrossFileDiagnostic {
    let Some(relateds) = relateds else {
        return diagnostic;
    };

    for related in relateds {
        let display_name = related.display_name();
        if attrs_filter.is_some_and(|attrs| !attrs.contains(&display_name)) {
            continue;
        }

        diagnostic = diagnostic.with_related(
            related.parent_file_id,
            related.source_start,
            cstr!("{} passed to <{}>", display_name, related.component_name),
        );
    }

    diagnostic
}
