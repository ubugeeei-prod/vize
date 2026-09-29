use super::super::virtual_project::VirtualProject;
use super::{CorsaResult, TypeCheckResult};

pub(super) fn finish_registered_result(
    mut result: TypeCheckResult,
    project: &VirtualProject,
) -> CorsaResult<TypeCheckResult> {
    result
        .diagnostics
        .extend(project.diagnostics().iter().cloned());
    let had_errors = result.has_errors();
    super::generic_private_names::apply(&mut result.diagnostics, project);
    if result.has_errors() {
        result.success = false;
        result.exit_code = result.exit_code.max(1);
    } else if had_errors {
        result.success = true;
        result.exit_code = 0;
    }
    Ok(result)
}
