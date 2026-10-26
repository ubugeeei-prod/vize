//! Retain one native syntax owner without fabricating generic bindings.

use std::ops::Range;

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{String, append};
use vize_croquis::Croquis;

use crate::virtual_ts::generic_parameters::GenericParameters;
use crate::virtual_ts::types::{
    DEFAULT_LIB_REFERENCES, VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput,
    emit_lib_reference_directives,
};
use crate::virtual_ts::{ProjectionFeatures, ProjectionMapping, ProjectionMeta, VizeMapping};

use super::setup_helpers::SetupHelperPlan;

/// Unsupported script shapes are refused by the descriptor-owned caller. The
/// actual setup body remains verbatim, including independent type errors and
/// import bindings. Only synthetic component scaffolding becomes opaque.
pub(crate) fn generate(
    summary: &Croquis,
    script: &str,
    source: (u32, SourceType),
    attribute: Range<usize>,
    plan: &GenericParameters,
    options: &VirtualTsOptions,
    generation: VirtualTsGenerationOptions<'_>,
) -> Option<VirtualTsOutput> {
    let (script_offset, source_type) = source;
    let _profile = vize_carton::profiler::global_profiler()
        .global_span("canon.virtual_ts.recover_generic_syntax");
    if !plan.recoverable_at_closing_delimiter || generation.split_script_setup_offsets.is_some() {
        return None;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() || parsed.program.hashbang.is_some() {
        return None;
    }
    let mut imports = Vec::new();
    let mut body = Vec::new();
    let mut previous = 0;
    for statement in &parsed.program.body {
        let span = statement.span();
        let chunk = previous..span.end as usize;
        match statement {
            Statement::ImportDeclaration(_) | Statement::TSImportEqualsDeclaration(_) => {
                push_chunk(&mut imports, chunk)
            }
            Statement::ExportAllDeclaration(_)
            | Statement::ExportDefaultDeclaration(_)
            | Statement::ExportNamedDeclaration(_)
            | Statement::TSExportAssignment(_)
            | Statement::TSModuleDeclaration(_) => return None,
            _ => push_chunk(&mut body, chunk),
        }
        previous = span.end as usize;
    }
    push_chunk(&mut body, previous..script.len());
    let mut code = String::default();
    let mut mapping = ProjectionMapping::new();
    emit_lib_reference_directives(
        &mut code,
        generation.lib_references.unwrap_or(DEFAULT_LIB_REFERENCES),
    );
    super::imports::emit_reference_path_directives(&mut code, &options.reference_paths);
    super::imports::emit_reference_type_directives(&mut code, Some(script));
    super::preamble::emit_module_preamble(&mut code, summary, generation, false);
    for chunk in imports {
        copy_script(&mut code, &mut mapping, script, chunk, script_offset)?;
        code.push('\n');
    }
    let is_async = super::generics::setup_signature(summary).2;
    if is_async {
        code.push_str("async ");
    }
    code.push_str("function __setup");
    let opening = code.len();
    code.push('<');
    let parameters = code.len();
    code.push_str(plan.authored());
    let closing = code.len();
    code.push_str(">() {\n");
    let helpers = SetupHelperPlan::collect(summary, Some(script), false);
    helpers.emit_error_recovery(&mut code, generation.hoist_shared_preamble);
    code.push('\n');
    for chunk in body {
        copy_script(&mut code, &mut mapping, script, chunk, script_offset)?;
    }
    code.push_str("\n}\nvoid __setup;\n");
    code.push_str(crate::virtual_ts::helpers::ERROR_COMPONENT_STUB);
    let diagnostic = ProjectionMeta {
        features: ProjectionFeatures::DIAGNOSTICS,
        ..ProjectionMeta::default()
    };
    mapping.push_with(
        VizeMapping::new(
            opening..opening + 1,
            attribute.start.checked_sub(1)?..attribute.start,
        ),
        diagnostic,
    );
    mapping.push_with(
        VizeMapping::new(parameters..parameters + plan.raw().len(), attribute.clone()),
        diagnostic,
    );
    mapping.push_with(
        VizeMapping::new(
            closing..closing + 1,
            attribute.end..attribute.end.checked_add(1)?,
        ),
        diagnostic,
    );
    mapping.sort_by_authored();
    Some(VirtualTsOutput { code, mapping })
}

fn push_chunk(chunks: &mut Vec<Range<usize>>, range: Range<usize>) {
    if let Some(last) = chunks.last_mut().filter(|last| last.end == range.start) {
        last.end = range.end;
    } else {
        chunks.push(range);
    }
}

fn copy_script(
    code: &mut String,
    mapping: &mut ProjectionMapping,
    script: &str,
    range: Range<usize>,
    source_offset: u32,
) -> Option<()> {
    let text = script.get(range.clone())?;
    if text.is_empty() {
        return Some(());
    }
    let start = code.len();
    append!(*code, "{text}");
    mapping.push(VizeMapping::new(
        start..code.len(),
        source_offset as usize + range.start..source_offset as usize + range.end,
    ));
    Some(())
}
