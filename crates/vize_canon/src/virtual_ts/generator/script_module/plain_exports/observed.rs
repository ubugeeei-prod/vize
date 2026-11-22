//! Authored plain-script exports consumed by virtual TypeScript and editor navigation.
use super::super::{NamespaceHoistPlan, strip_named_value_exports};
use super::{
    PlainScriptExport, PlainScriptExportKind, emit_setup_invocation_and_exports_with_mappings,
    push_setup_return_fields,
};
use crate::virtual_ts::{TextBuf, VizeMapping, VizeSemanticLink};
use oxc_span::SourceType;
use std::borrow::Cow;
use vize_atelier_sfc::SfcDescriptor;
use vize_carton::{CompactString, FxHashSet, SmallVec, SourceBlock, SourceRoot, Span};
use vize_l1::embed::authored_program::AuthoredProgram;
use vize_l2::query::script::{
    DeclarationExport, DeclarationSyntaxError, ScriptDeclarationKind, ScriptDeclarations,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlainScriptBridgeError {
    Selection,
    ExternalScript,
    Grammar,
    Source,
    RetryRequired,
    Syntax(DeclarationSyntaxError),
}

/// The ambient declarations and setup footer of a plain-script module. Editor
/// links address the caller's generated document, including its original prefix.
#[derive(Debug, PartialEq, Eq)]
pub struct PlainScriptExportText {
    pub ambient: CompactString,
    pub code: CompactString,
    pub links: Vec<VizeSemanticLink>,
}

/// One module's setup-return fields, relocated declarations, export text and
/// editor navigation links. All ranges originate in the selected ordinary block.
#[derive(Debug)]
pub struct PlainScriptBridge<'a> {
    block: SourceBlock<'a>,
    exports: Vec<PlainScriptExport>,
    keywords: Vec<u32>,
    hoist: NamespaceHoistPlan,
}

impl<'a> PlainScriptBridge<'a> {
    /// Supply the checker's original TS module parse. If it reports diagnostics,
    /// supply its TSX retry over the same block. Valid TS takes precedence even
    /// in a block whose declared language is JSX or TSX.
    pub fn new(
        descriptor: &'a SfcDescriptor<'_>,
        ts: &AuthoredProgram<'_>,
        tsx: Option<&AuthoredProgram<'_>>,
    ) -> Result<Self, PlainScriptBridgeError> {
        let script = descriptor
            .script
            .as_ref()
            .ok_or(PlainScriptBridgeError::Selection)?;
        if descriptor.script_setup.is_some() || script.setup {
            return Err(PlainScriptBridgeError::Selection);
        }
        if script.src.is_some() {
            return Err(PlainScriptBridgeError::ExternalScript);
        }
        if let Some(language) = script.lang.as_deref()
            && !["js", "jsx", "ts", "tsx"]
                .iter()
                .any(|profile| language.trim().eq_ignore_ascii_case(profile))
        {
            return Err(PlainScriptBridgeError::Grammar);
        }
        let block = ts.block();
        if !core::ptr::eq(descriptor.source.as_ref(), block.root_source())
            || script.loc.start != block.start() as usize
            || script.loc.end != block.end() as usize
            || !(core::ptr::eq(script.content.as_ref(), block.source())
                || script.content.as_ref() == block.source())
        {
            return Err(PlainScriptBridgeError::Source);
        }
        if ts.source_type() != SourceType::ts().with_module(true) {
            return Err(PlainScriptBridgeError::Grammar);
        }
        let retried = ts.parsed().panicked || !ts.parsed().diagnostics.is_empty();
        let original = match (retried, tsx) {
            (false, _) => ts,
            (true, None) => return Err(PlainScriptBridgeError::RetryRequired),
            (true, Some(retry)) => {
                if retry.source_type() != SourceType::tsx().with_module(true) {
                    return Err(PlainScriptBridgeError::Grammar);
                }
                if !core::ptr::eq(retry.block().root_source(), block.root_source())
                    || retry.block().span() != block.span()
                {
                    return Err(PlainScriptBridgeError::Source);
                }
                retry
            }
        };
        let declarations =
            ScriptDeclarations::new(original).map_err(PlainScriptBridgeError::Syntax)?;
        // Most plain modules export only a handful of names. Keep those
        // borrowed spellings inline, then admit further names through a hash set.
        let mut first_names = SmallVec::<[&str; 8]>::new();
        let mut other_names = FxHashSet::default();
        let mut exports = Vec::new();
        let mut keywords = Vec::new();
        // Namespace relocation retries TSX only for a fatal first parse.
        // After a retry retain its independent original recovery policy.
        let mut needs_hoist = retried;
        declarations.visit_declaration_statements(|statement| {
            let form = statement.kind();
            if statement.export != DeclarationExport::TypeOnly {
                needs_hoist |= matches!(
                    form,
                    Some(
                        ScriptDeclarationKind::Function
                            | ScriptDeclarationKind::Class
                            | ScriptDeclarationKind::Enum
                            | ScriptDeclarationKind::Namespace
                    )
                );
            }
            if statement.export != DeclarationExport::Named {
                return;
            }
            let kind = match form {
                Some(ScriptDeclarationKind::Variable(_) | ScriptDeclarationKind::Function) => {
                    PlainScriptExportKind::Value
                }
                Some(ScriptDeclarationKind::Class) => PlainScriptExportKind::Class,
                Some(ScriptDeclarationKind::Enum) => PlainScriptExportKind::Enum,
                _ => return,
            };
            keywords.push(statement.span.start - block.start());
            statement.visit_names(|row| {
                if first_names.contains(&row.name) {
                    return;
                }
                if first_names.len() < first_names.inline_size() {
                    first_names.push(row.name);
                } else if !other_names.insert(row.name) {
                    return;
                }
                exports.push(PlainScriptExport {
                    name: CompactString::new(row.name),
                    kind,
                    source_range: (row.span.start - block.start()) as usize
                        ..(row.span.end - block.start()) as usize,
                    bridged_value: true,
                });
            });
        });
        drop(other_names);
        let hoist = if needs_hoist {
            // Relocation keeps a recovered TS tree when its parse completed.
            let hoist_parse = if ts.parsed().panicked { original } else { ts };
            NamespaceHoistPlan::from_authored_program(&hoist_parse.parsed().program, block.source())
        } else {
            NamespaceHoistPlan::default()
        };
        hoist.reconcile_exports(&mut exports);
        // The owned plan needs the immutable descriptor, but it can outlive
        // both parser arenas. Owned descriptors may copy each block's text.
        let root = SourceRoot::new(descriptor.source.as_ref())
            .map_err(|_| PlainScriptBridgeError::Source)?;
        let source = root
            .source()
            .get(script.loc.start..script.loc.end)
            .ok_or(PlainScriptBridgeError::Source)?;
        let block = root
            .block(source, block.start())
            .map_err(|_| PlainScriptBridgeError::Source)?;
        Ok(Self {
            block,
            exports,
            keywords,
            hoist,
        })
    }

    /// File-absolute spans Canon emits verbatim at module scope, including
    /// the original comments attached to relocated overloads and declarations.
    pub fn relocated_declarations(&self) -> impl Iterator<Item = Span> + '_ {
        self.hoist
            .spans()
            .iter()
            .map(|&(start, end)| Span::new(self.block.start() + start, self.block.start() + end))
    }

    /// File-absolute ranges of named value export keywords, before relocation.
    pub fn export_keywords(&self) -> impl Iterator<Item = Span> + '_ {
        self.keywords.iter().filter_map(|&relative| {
            // TypeScript decorators can extend a statement's source range.
            // The line emitter erases only a keyword at an actual token start.
            let start = relative as usize;
            (self.block.source().get(start..start + 6) == Some("export")).then(|| {
                let absolute = self.block.start() + relative;
                Span::new(absolute, absolute + 6)
            })
        })
    }

    /// Mask the same export keywords as Canon's line projector while preserving
    /// every byte offset. Module-scope spans are available independently.
    pub fn masked_script(&self) -> Cow<'a, str> {
        let mut source = Cow::Borrowed(self.block.source());
        strip_named_value_exports(&mut source, 0, &self.keywords);
        source
    }

    pub fn append_setup_fields(&self, fields: &mut Vec<CompactString>) {
        push_setup_return_fields(&self.exports, fields);
        self.hoist
            .push_captured_return_fields(&self.exports, fields);
    }

    /// Render the ambient declarations and append the footer to a virtual
    /// document. Its previous rows determine the editor links. The checker's
    /// projection scope preserves authored utility-type names in both pieces.
    pub fn project(
        &self,
        summary: &vize_croquis::Croquis,
        text: CompactString,
        mappings: &mut Vec<VizeMapping>,
    ) -> PlainScriptExportText {
        let _names = crate::virtual_ts::generated_names::bind(None, summary);
        let mut ambient = TextBuf::default();
        self.hoist
            .emit_ambient_captures(&mut ambient, &self.exports);
        let mut text = TextBuf::from_compact(text);
        let links = emit_setup_invocation_and_exports_with_mappings(
            &mut text,
            mappings,
            &self.exports,
            self.block.start(),
        );
        PlainScriptExportText {
            ambient: ambient.into_compact(),
            code: text.into_compact(),
            links,
        }
    }
}

#[cfg(test)]
mod tests;
