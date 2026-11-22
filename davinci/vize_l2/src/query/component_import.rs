//! Resolve authored static component owners against their original setup import.
use super::{BindingRef, Model};
use crate::file::{DeclarationKind, FileIssueKind, Namespace, ScopeId, TemplateIssue};
use crate::{op::Op, walk::NodeRef};
use oxc_ast::ast::{ImportDeclarationSpecifier, ModuleExportName, Statement};
use oxc_parser::{AdmittedProgram, ParseOptions};
use vize_l0::{Span, id::NodeId};
use vize_l1::container::vue::{ScriptRole, ScriptView, TemplateView};
use vize_l1::embed::Lang;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentImportIssue {
    OriginalProgram,
    IncompleteFile,
    OriginalNode,
    UnrepresentableImport,
    UnresolvedTag(Span),
    AmbiguousTag(Span),
}

pub struct ImportedComponentSite<'f, 'a> {
    node: NodeId,
    tag: &'a str,
    name: Span,
    element: Span,
    binding: BindingRef<'f, 'a>,
    module: &'f str,
    export: &'f str,
}

impl<'f, 'a> ImportedComponentSite<'f, 'a> {
    #[must_use]
    pub fn node(&self) -> NodeId {
        self.node
    }
    #[must_use]
    pub fn tag(&self) -> &'a str {
        self.tag
    }
    #[must_use]
    pub fn name_span(&self) -> Span {
        self.name
    }
    #[must_use]
    pub fn element_span(&self) -> Span {
        self.element
    }
    #[must_use]
    pub fn binding(&self) -> BindingRef<'f, 'a> {
        self.binding
    }
    #[must_use]
    pub fn module(&self) -> &'f str {
        self.module
    }
    #[must_use]
    pub fn export(&self) -> &'f str {
        self.export
    }
}

impl<'f, 'a> Model<'f, 'a> {
    /// Visit the model's component operations and their direct setup value
    /// imports in opening-tag order. Each construction diagnostic must have
    /// its original node. The SFC owner checks whole-template coverage.
    /// A late refusal invalidates the entire visit, including its earlier rows.
    pub fn visit_setup_component_imports(
        self,
        script: ScriptView<'_, 'a>,
        program: &AdmittedProgram<'_, 'a>,
        template: TemplateView<'_, 'a>,
        mut visit: impl FnMut(ImportedComponentSite<'f, 'a>),
    ) -> Result<(), ComponentImportIssue> {
        let file = self.file;
        let [unit] = file.units() else {
            return Err(ComponentImportIssue::OriginalProgram);
        };
        let ty = program.source_type();
        if script.role() != ScriptRole::Setup
            || !matches!(script.lang(), Lang::Js | Lang::Ts)
            || !core::ptr::eq(file.artifact().source(), script.source())
            || !core::ptr::eq(script.source(), template.source())
            || !core::ptr::eq(program.source(), script.block().source())
            || unit.id.index() as usize != script.container_index()
            || unit.span != script.block().span()
            || !unit.profile.module
            || unit.profile.jsx
            || !ty.is_module()
            || ty.is_unambiguous()
            || ty.is_jsx()
            || ty.is_typescript_definition()
            || ty.is_typescript() != (script.lang() == Lang::Ts)
            || ty.is_typescript() != unit.profile.typescript
            || program.program().source_type != ty
            || program.options() != ParseOptions::default()
            || program.program().body.is_empty()
            || unit.origin.body != program.program().body.as_ptr() as usize
            || unit.origin.length != program.program().body.len()
        {
            return Err(ComponentImportIssue::OriginalProgram);
        }
        if !unit.walk_completed()
            || !file.issues().is_empty()
            || !file.facts.template_walk.is_complete()
            || file
                .template_issues()
                .iter()
                .any(|i| i.kind != FileIssueKind::UnsupportedComponent)
        {
            return Err(ComponentImportIssue::IncompleteFile);
        }
        // OXC's Rust string projection cannot carry an unpaired UTF-16 unit.
        // Check the original literal before publishing module or export names.
        for statement in &program.program().body {
            if let Statement::ImportDeclaration(import) = statement
                && (import.source.lone_surrogates || import.specifiers.iter().flatten().any(|s| {
                    matches!(s, ImportDeclarationSpecifier::ImportSpecifier(named)
                        if matches!(&named.imported, ModuleExportName::StringLiteral(literal) if literal.lone_surrogates))
                }))
            {
                return Err(ComponentImportIssue::UnrepresentableImport);
            }
        }
        let template = template.block();
        let mut previous_name_end = template.span().start;
        let mut issues = file.template_issues().iter();
        let mut result = Ok(());
        file.artifact()
            .visit_nodes(&mut |id, node| {
                let NodeRef::Op(Op::Component(component)) = node else {
                    return;
                };
                if result.is_err() {
                    return;
                }
                let span = component.span;
                let tag = component.name;
                result = (|| {
                    if issues.next()
                        != Some(&TemplateIssue {
                            node: Some(id),
                            span,
                            kind: FileIssueKind::UnsupportedComponent,
                        })
                        || !template.contains_block_span(span)
                    {
                        return Err(ComponentImportIssue::OriginalNode);
                    }
                    let name = template
                        .span_of(tag)
                        .filter(|s| {
                            span.start.checked_add(1) == Some(s.start)
                                && previous_name_end <= s.start
                                && s.start < s.end
                                && s.end <= span.end
                        })
                        .ok_or(ComponentImportIssue::OriginalNode)?;
                    let binding = self.component_binding(unit.scope, tag, name)?;
                    let declaration = binding
                        .declaration()
                        .ok_or(ComponentImportIssue::UnresolvedTag(name))?;
                    let (Some(module), Some(export)) = (
                        declaration.import_source.as_deref(),
                        declaration.imported_name.as_deref(),
                    ) else {
                        return Err(ComponentImportIssue::UnresolvedTag(name));
                    };
                    if declaration.unit != unit.id
                        || declaration.scope != unit.scope
                        || !declaration.is_direct_program()
                        || declaration.kind != DeclarationKind::Import
                        || declaration.namespace != Namespace::Value
                        || export == "*"
                    {
                        return Err(ComponentImportIssue::UnresolvedTag(name));
                    }
                    previous_name_end = name.end;
                    visit(ImportedComponentSite {
                        node: id,
                        tag,
                        name,
                        element: span,
                        binding,
                        module,
                        export,
                    });
                    Ok(())
                })();
            })
            .map_err(|_| ComponentImportIssue::OriginalNode)?;
        result?;
        if issues.next().is_some() {
            return Err(ComponentImportIssue::OriginalNode);
        }
        Ok(())
    }

    fn component_binding(
        self,
        scope: ScopeId,
        tag: &str,
        span: Span,
    ) -> Result<BindingRef<'f, 'a>, ComponentImportIssue> {
        if vize_l0::is_builtin_tag(tag)
            || matches!(
                tag,
                "Teleport"
                    | "teleport"
                    | "Suspense"
                    | "suspense"
                    | "KeepAlive"
                    | "keep-alive"
                    | "BaseTransition"
                    | "base-transition"
                    | "Transition"
                    | "transition"
                    | "TransitionGroup"
                    | "transition-group"
            )
        {
            return Err(ComponentImportIssue::UnresolvedTag(span));
        }
        if let Some(exact) = self.file.lookup(scope, tag, Namespace::Value) {
            return Ok(exact);
        }
        let (first, tail) = tag
            .split_at_checked(1)
            .ok_or(ComponentImportIssue::UnresolvedTag(span))?;
        if !first.bytes().all(|b| b.is_ascii_lowercase())
            || tail.ends_with('-')
            || !tail
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            || tag.contains("--")
        {
            return Err(ComponentImportIssue::UnresolvedTag(span));
        }
        // Do not invent a spelling for a namespace member, configured custom
        // element or unusual identifier. For ordinary kebab tags the possible
        // setup bindings are the camel and Pascal spellings of the same words.
        // Most tag spellings fit on the stack and use the scope's ordered name
        // map. Longer identifiers keep their full spelling in the streamed check.
        let mut buffer = [0u8; 128];
        if tag.len() <= buffer.len() {
            let mut len = 0;
            for (slot, byte) in buffer.iter_mut().zip(component_spelling(tag, false)) {
                *slot = byte;
                len += 1;
            }
            let key = buffer
                .get_mut(..len)
                .ok_or(ComponentImportIssue::UnresolvedTag(span))?;
            let lookup = |bytes: &[u8]| {
                core::str::from_utf8(bytes)
                    .ok()
                    .and_then(|name| self.file.lookup(scope, name, Namespace::Value))
            };
            let camel = lookup(key);
            key.first_mut()
                .ok_or(ComponentImportIssue::UnresolvedTag(span))?
                .make_ascii_uppercase();
            return match (camel, lookup(key)) {
                (Some(_), Some(_)) => Err(ComponentImportIssue::AmbiguousTag(span)),
                (Some(binding), None) | (None, Some(binding)) => Ok(binding),
                _ => Err(ComponentImportIssue::UnresolvedTag(span)),
            };
        }
        let mut matches = self.file.facts.declarations.iter().filter(|decl| {
            decl.scope == scope
                && decl.namespace == Namespace::Value
                && component_spelling(
                    tag,
                    decl.name
                        .as_bytes()
                        .first()
                        .is_some_and(u8::is_ascii_uppercase),
                )
                .eq(decl.name.bytes())
        });
        let declaration = matches
            .next()
            .ok_or(ComponentImportIssue::UnresolvedTag(span))?;
        if matches.next().is_some() {
            return Err(ComponentImportIssue::AmbiguousTag(span));
        }
        self.file
            .binding(declaration.id)
            .ok_or(ComponentImportIssue::UnresolvedTag(span))
    }
}

fn component_spelling(tag: &str, mut uppercase: bool) -> impl Iterator<Item = u8> + '_ {
    tag.bytes().filter_map(move |byte| {
        if byte == b'-' {
            uppercase = true;
            None
        } else {
            let letter = if uppercase {
                byte.to_ascii_uppercase()
            } else {
                byte
            };
            uppercase = false;
            Some(letter)
        }
    })
}
