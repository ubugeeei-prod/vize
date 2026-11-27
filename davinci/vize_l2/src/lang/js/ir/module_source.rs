//! Original static module source operands over the actual completed unit.

use super::ProgramInput;
use crate::ir::{Export, Import, L2Ir, Namespace, ScriptUnit, ScriptUnitId};
use oxc_ast::ast::StringLiteral;
use vize_l0::Span;

mod read;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleSourceErrorKind {
    Source,
    Profile,
    MissingUnit,
    ProgramOrigin,
    EmptyProgram,
    IncompleteFile,
    DynamicImport,
    EmptySourceExport,
    OriginalSite,
    LoneSurrogateRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleSourceError {
    pub unit: ScriptUnitId,
    pub span: Span,
    pub kind: ModuleSourceErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleSourceKind {
    Import,
    NamedReexport,
    AllReexport,
}

/// Borrowed authority over one actual unit, its live Program and completed File.
/// URI, snapshot and whole-SFC policy are not established by this neutral view.
///
/// ```compile_fail
/// use vize_l2::{ir::L2Ir, lang::js::{OriginalModuleSources, ProgramInput}};
/// fn forge<'f, 'p, 'a>(l2ir: &'f L2Ir<'a>, input: ProgramInput<'p, 'a>) {
///     let unit = &l2ir.units()[0];
///     let _ = OriginalModuleSources { l2ir, input, unit };
/// }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::OriginalModuleSources;
/// fn copy(view: OriginalModuleSources<'_, '_, '_>) { let _ = view.clone(); }
/// ```
/// ```compile_fail
/// use vize_l2::{ir::L2Ir, lang::js::ProgramInput};
/// fn discard<'p, 'a>(l2ir: L2Ir<'a>, input: ProgramInput<'p, 'a>) {
///     let view = l2ir.original_module_sources(input).unwrap();
///     let mut original = None;
///     view.for_each(|row| original = Some(row)).unwrap();
///     drop(l2ir);
///     let _ = original.unwrap().l2ir();
/// }
/// ```
/// ```compile_fail
/// use oxc_ast::ast::Program;
/// use vize_l2::ir::L2Ir;
/// fn substitute(l2ir: &L2Ir<'_>, program: &Program<'_>) {
///     let _ = l2ir.original_module_sources(program);
/// }
/// ```
/// ```compile_fail
/// use oxc_parser::Parser;
/// use oxc_span::SourceType;
/// use vize_l0::{Allocator, SourceRoot};
/// use vize_l2::lang::js::{L2IrProducer, ProgramInput, ProgramScope};
/// let arena = Allocator::default();
/// let source = "import 'x';";
/// let block = SourceRoot::new(source).unwrap().whole_block();
/// let original = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
/// let mut producer = L2IrProducer::new(&arena, source).unwrap();
/// producer.program(ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(), ProgramScope::Module).unwrap();
/// let file = producer.finish().unwrap();
/// let view = file.original_module_sources(ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap()).unwrap();
/// drop(original);
/// view.for_each(|literal| { let _ = literal.literal(); }).unwrap();
/// ```
pub struct OriginalModuleSources<'f, 'p, 'a> {
    l2ir: &'f L2Ir<'a>,
    input: ProgramInput<'p, 'a>,
    unit: &'f ScriptUnit,
}

impl<'a> L2Ir<'a> {
    /// Join checked original parser admission to this exact completed unit.
    /// Equal text, numeric IDs and a second parse cannot replace its body storage.
    pub fn original_module_sources<'f, 'p>(
        &'f self,
        input: ProgramInput<'p, 'a>,
    ) -> Result<OriginalModuleSources<'f, 'p, 'a>, ModuleSourceError> {
        let reject = |kind| ModuleSourceError {
            unit: ScriptUnitId(input.index),
            span: input.block.span(),
            kind,
        };
        if !core::ptr::eq(self.artifact().source(), input.block.root_source()) {
            return Err(reject(ModuleSourceErrorKind::Source));
        }
        let profile = input.source_type();
        // JSX completeness comes from the same sole resolver walk. The exact
        // original JS/TS/JSX/TSX unit profile remains checked below.
        if !profile.is_module() || profile.is_typescript_definition() {
            return Err(reject(ModuleSourceErrorKind::Profile));
        }
        let Some(unit) = self
            .units()
            .iter()
            .find(|unit| unit.id.index() == input.index)
        else {
            return Err(reject(ModuleSourceErrorKind::MissingUnit));
        };
        if unit.span != input.block.span() {
            return Err(reject(ModuleSourceErrorKind::Source));
        }
        if unit.profile != input.profile {
            return Err(reject(ModuleSourceErrorKind::Profile));
        }
        let program = input.references.program();
        // The original empty arena Vec has a shared dangling data pointer.
        // Its address/length cannot certify this particular normally owned root.
        if program.body.is_empty() {
            return Err(reject(ModuleSourceErrorKind::EmptyProgram));
        }
        if unit.origin.body != program.body.as_ptr() as usize
            || unit.origin.length != program.body.len()
        {
            return Err(reject(ModuleSourceErrorKind::ProgramOrigin));
        }
        if !self.is_complete() || !unit.walk_completed() {
            return Err(reject(ModuleSourceErrorKind::IncompleteFile));
        }
        if unit.origin.module_source_gaps & crate::ir::DYNAMIC_IMPORT != 0 {
            return Err(reject(ModuleSourceErrorKind::DynamicImport));
        }
        if unit.origin.module_source_gaps & crate::ir::EMPTY_SOURCE_EXPORT != 0 {
            return Err(reject(ModuleSourceErrorKind::EmptySourceExport));
        }
        Ok(OriginalModuleSources {
            l2ir: self,
            input,
            unit,
        })
    }
}

#[derive(Clone, Copy)]
enum Row<'f> {
    Import(&'f Import),
    Export(&'f Export),
}

/// The original AST operand and actual semantic row stay borrowed together.
/// Metadata extraction does not grant completion to another File/source.
///
/// ```compile_fail
/// use vize_l2::{ir::L2Ir, lang::js::OriginalModuleSource};
/// fn substitute<'f, 'p, 'a>(l2ir: &'f L2Ir<'a>, original: OriginalModuleSource<'f, 'p, 'a>) {
///     let _ = OriginalModuleSource { l2ir, ..original };
/// }
/// ```
/// ```compile_fail
/// use vize_l0::{Span, String};
/// use vize_l2::ir::{Import, Namespace, ScriptUnitId};
/// fn forge(unit: ScriptUnitId, span: Span) {
///     let _ = Import { unit, source: String::from("x"), namespace: Namespace::Value, span, source_site: None };
/// }
/// ```
pub struct OriginalModuleSource<'f, 'p, 'a> {
    l2ir: &'f L2Ir<'a>,
    row: Row<'f>,
    literal: &'p StringLiteral<'a>,
    raw: &'a str,
    authored: &'a str,
    copied: &'f str,
    quoted: Span,
    content: Span,
    declaration: Span,
    kind: ModuleSourceKind,
    namespace: Namespace,
}

impl<'f, 'p, 'a> OriginalModuleSource<'f, 'p, 'a> {
    pub fn l2ir(&self) -> &'f L2Ir<'a> {
        self.l2ir
    }
    pub fn literal(&self) -> &'p StringLiteral<'a> {
        self.literal
    }
    pub fn quoted_span(&self) -> Span {
        self.quoted
    }
    pub fn content_span(&self) -> Span {
        self.content
    }
    pub fn declaration_span(&self) -> Span {
        self.declaration
    }
    pub fn kind(&self) -> ModuleSourceKind {
        self.kind
    }
    /// Namespace of the original source-bearing declaration, not one specifier.
    pub fn namespace(&self) -> Namespace {
        self.namespace
    }
    pub fn raw(&self) -> &'a str {
        self.raw
    }
    pub fn authored_content(&self) -> &'a str {
        self.authored
    }
    /// Original decoded value; lone-surrogate requests cannot mint this receipt.
    pub fn decoded_request(&self) -> &'a str {
        self.literal.value.as_str()
    }
    /// Existing copied semantic request remains separate from authored geometry.
    pub fn copied_request(&self) -> &'f str {
        self.copied
    }
    pub fn import(&self) -> Option<&'f Import> {
        if let Row::Import(row) = self.row {
            Some(row)
        } else {
            None
        }
    }
    pub fn export(&self) -> Option<&'f Export> {
        if let Row::Export(row) = self.row {
            Some(row)
        } else {
            None
        }
    }
}
