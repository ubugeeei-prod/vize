//! A TypeScript view of one SFC script block for whole-document import
//! actions.
//!
//! The canonical virtual document rewrites import specifiers and references
//! every script binding from generated code, so tsgo can neither keep authored
//! specifiers nor see an import as unused there. This projection is the
//! authored script verbatim, followed by one statement that reads exactly the
//! imports the template uses. Edits inside the script map back by a fixed
//! offset; an edit that touches the appended statement is refused.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::{Position, Range};

use crate::ide::{IdeContext, offset_to_position, position_to_offset};

pub(super) struct ImportProjection {
    /// Overlay path beside the SFC, so relative specifiers read the same.
    pub(super) path: vize_l0::String,
    pub(super) text: String,
    /// Byte offset of the script content in the SFC.
    script_start: usize,
    script_len: usize,
}

impl ImportProjection {
    pub(super) fn build(ctx: &IdeContext<'_>) -> Option<Self> {
        let descriptor = ctx.descriptor()?;
        let block = descriptor
            .script_setup
            .as_ref()
            .or(descriptor.script.as_ref())?;
        if block.src.is_some() {
            return None;
        }
        let extension = match block.lang.as_deref() {
            Some("ts") => "ts",
            Some("tsx") => "tsx",
            Some("jsx") => "jsx",
            None | Some("js") => "js",
            Some(_) => return None,
        };
        let content = block.content.as_ref();
        if ctx
            .content
            .get(block.loc.start..block.loc.start + content.len())?
            != content
        {
            return None;
        }
        let source_path = ctx.uri.to_file_path().ok()?;
        let path = vize_l0::cstr!("{}.__vize_imports__.{extension}", source_path.to_str()?);

        let used = template_used_imports(ctx, descriptor, content, extension)?;
        let mut text = content.to_string();
        if !used.is_empty() {
            text.push_str("\n;void [");
            text.push_str(&used.join(", "));
            text.push_str("];\n");
        }
        Some(Self {
            path,
            text,
            script_start: block.loc.start,
            script_len: content.len(),
        })
    }

    /// Map a projection range onto the SFC, refusing the appended statement.
    pub(super) fn authored_range(&self, ctx: &IdeContext<'_>, range: Range) -> Option<Range> {
        let map = |position: Position| {
            let offset = position_to_offset(&self.text, position.line, position.character)?;
            if offset > self.script_len {
                return None;
            }
            let (line, character) = offset_to_position(&ctx.content, self.script_start + offset);
            Some(Position::new(line, character))
        };
        Some(Range::new(map(range.start)?, map(range.end)?))
    }
}

/// Value imports of the script that something reads. Croquis lists the setup
/// bindings that neither the script, the template nor a style reads; every
/// other value import is named in the appended statement. When the template
/// cannot be analysed, nothing is proven unused and every import is kept.
fn template_used_imports(
    ctx: &IdeContext<'_>,
    descriptor: &vize_incremental::SharedDescriptor,
    content: &str,
    extension: &str,
) -> Option<Vec<String>> {
    let patterned = ctx.state.patterned_template_enabled();
    let descriptor = vize_atelier_sfc::prepare_root_patterned_template(
        descriptor,
        patterned,
        false,
        Default::default(),
    )
    .ok()?;
    let template_allocator = vize_l0::Allocator::new();
    let parsed = descriptor
        .template
        .as_ref()
        .map(|template| vize_armature::parse(&template_allocator, &template.content));
    // Extension notices (such as a rewritten self-closing tag) lose no reads;
    // any other parse error can hide one.
    let template_ok = parsed.as_ref().is_none_or(|(_, errors)| {
        errors
            .iter()
            .all(|error| error.code == vize_relief::ErrorCode::ExtendPoint)
    });
    let mut options = vize_atelier_sfc::croquis::SfcCroquisOptions::full();
    options.analyzer_options.experimental_patterned_template = patterned;
    options.unused_bindings = true;
    let ast = parsed.as_ref().map(|(root, _)| root);
    let croquis = if ctx.state.legacy_vue2_enabled() {
        vize_atelier_sfc::croquis::analyze_sfc_descriptor_with_context_legacy_vue2(
            &descriptor,
            ast,
            options,
        )
        .croquis
    } else if ctx.state.options_api_enabled() {
        vize_atelier_sfc::croquis::analyze_sfc_descriptor_with_context_options_api(
            &descriptor,
            ast,
            options,
        )
        .croquis
    } else {
        vize_atelier_sfc::croquis::analyze_sfc_descriptor(&descriptor, ast, options)
    };
    let unused: &[vize_l0::CompactString] = if template_ok {
        &croquis.unused_bindings
    } else {
        &[]
    };

    let allocator = Allocator::default();
    let source_type = match extension {
        "ts" => SourceType::ts(),
        "tsx" => SourceType::tsx(),
        "jsx" => SourceType::jsx(),
        _ => SourceType::mjs(),
    };
    let parsed = Parser::new(&allocator, content, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let mut used = Vec::new();
    for statement in &parsed.program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind == ImportOrExportKind::Type {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            let (local, type_only) = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                    specifier.local.name.as_str(),
                    specifier.import_kind == ImportOrExportKind::Type,
                ),
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    (specifier.local.name.as_str(), false)
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    (specifier.local.name.as_str(), false)
                }
            };
            if !type_only && !unused.iter().any(|name| name.as_str() == local) {
                used.push(local.to_string());
            }
        }
    }
    Some(used)
}
