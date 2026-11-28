//! Cross-block SFC context available to a script rule.
//!
//! Script rules see a single `<script>` / `<script setup>` block and normally
//! cannot observe the rest of the file. Rules that correlate a script
//! declaration with `<template>` usage read the template from here.
//!
//! # Two channels, two directions of error
//!
//! [`SfcScriptContext::template_source`] is the **raw** template text. It is
//! what [`crate::rules::script::props_emits::template_emits`] scans, and that
//! scan is documented as one-directional: a name recovered from it only ever
//! *suppresses* an "unused" report, so an over-match (a call inside an HTML
//! comment, a coincidental attribute value) can at worst hide a finding.
//!
//! [`SfcScriptContext::template_surface`] is the **parsed L1 template**, for the
//! opposite direction: a rule that *creates* a finding from template evidence.
//! There an over-match is a false positive — a diagnostic at a location where
//! the user did nothing wrong — so raw text is not good enough. The AST
//! structurally excludes the cases a text scan cannot:
//!
//! * an HTML comment is a `Comment` node, never an expression;
//! * a text node is a `Text` node;
//! * a plain attribute is an `Attribute`, not a `Directive` carrying an
//!   expression;
//! * a `v-pre` region has its directives rewritten to attributes and its
//!   interpolations to text, so nothing inside it is compiled.
//!
//! Matching a *name* inside a recovered expression still needs an oxc parse of
//! that expression, so an occurrence inside a string literal cannot count; see
//! [`crate::rules::vue::no_mutating_props`] for the worked example.
//!
//! [`SfcScriptContext::template_offset`] is the byte offset of the template
//! block's content within the whole SFC source, so a rule reporting at a
//! *template* location can map an AST offset back to the file.
//!
//! The `Default` value (no template) is what standalone entry points (inline
//! HTML scripts, [`crate::rules::script::ScriptLinter::lint`]) pass, so rules
//! must treat a missing template as "no template usage observable", never as an
//! error. `template_surface` is additionally `None` unless some enabled rule asked
//! for it through [`crate::rules::script::ScriptRule::uses_template_ast`], and
//! when the template failed to parse.

use vize_croquis::types::ResolvedTypeWorld;
use vize_l1::markup::SurfaceTree;

/// Cross-block context handed to a script rule when the linted block is part of
/// an SFC. See the module documentation for how the three channels differ.
#[derive(Clone, Copy, Default)]
pub struct SfcScriptContext<'a> {
    /// Whether the current script block came from a parsed SFC descriptor.
    ///
    /// Standalone JavaScript/TypeScript files and inline HTML scripts use the
    /// default value (`false`). Rules that need SFC-only conclusions should
    /// check this rather than inferring from offsets or template presence: an
    /// SFC may omit `<template>`, and an inline HTML script has a non-zero
    /// source offset.
    pub is_sfc: bool,
    /// Whether the authored standalone file uses a JSX component extension.
    ///
    /// Bare default-exported objects are component definitions in Vue SFCs
    /// and `.jsx` / `.tsx` modules, but not in ordinary JavaScript or
    /// TypeScript modules. This bit carries that filename-owned distinction to
    /// AST rules without asking them to infer it from source syntax.
    pub is_jsx_file: bool,
    /// Whether this is a `<script setup>` block. Some Vue macros are only
    /// available there, even though ordinary scripts may call the same APIs.
    pub is_script_setup: bool,
    /// Whether the block is TypeScript: an SFC block with `lang="ts"` or
    /// `lang="tsx"`, or a `.ts`/`.tsx`/`.mts`/`.cts` file. Every block is
    /// parsed as TypeScript, so rules that only make sense with type syntax
    /// read this instead of the parse result.
    pub typescript: bool,
    /// Whether the TypeScript block/file uses the TSX dialect. A few upstream
    /// rules intentionally require an exact `lang="ts"` SFC block and do not
    /// run for `lang="tsx"`.
    pub typescript_jsx: bool,
    /// Raw `<template>` block content, when the SFC declares a template.
    pub template_source: Option<&'a str>,
    /// Parsed `<template>` AST, when the SFC declares a template, some enabled
    /// rule requested it, and it parsed without fatal errors.
    pub template_surface: Option<&'a SurfaceTree<'a>>,
    /// Byte offset of the `<template>` content inside the SFC source, for
    /// reporting at a template location.
    pub template_offset: Option<u32>,
    /// Whether the block being linted is the SFC's **only** script block.
    ///
    /// A rule whose conclusion is a property of the whole file rather than of
    /// one block has to be careful with an SFC carrying both `<script>` and
    /// `<script setup>`: it is invoked once per block, each invocation sees
    /// only half the script surface, so a conclusion drawn there would be both
    /// wrong (an option declared in the sibling block is invisible — a
    /// `<script> export default { inheritAttrs: false }` next to a
    /// `<script setup>`) and doubled (both invocations would report it). Such
    /// a rule reports only when this is `true`, or combines both blocks through
    /// [`Self::sibling_program`] from one invocation.
    ///
    /// `false` in the `Default` value: a standalone script has no template to
    /// correlate with, so no whole-file conclusion is available there either.
    pub sole_script_block: bool,
    /// The other script block's program when the SFC has both `<script>` and
    /// `<script setup>` and that block parsed cleanly. A whole-file rule can
    /// combine both halves from one invocation instead of skipping the file.
    pub sibling_program: Option<&'a oxc_ast::ast::Program<'a>>,
    /// Byte offset of [`Self::sibling_program`] within the SFC source.
    ///
    /// Rules which resolve a declaration from the other script block need its
    /// own origin when they report at that declaration. Reusing the current
    /// block's offset silently points diagnostics at the wrong line.
    pub sibling_offset: Option<usize>,
    /// Cross-file type world for compiler macro type arguments.
    ///
    /// The world is attached only for an SFC whose filename is known. Rules
    /// retain their conservative single-file behavior when resolution is
    /// unavailable or incomplete.
    pub resolved_type_world: Option<&'a ResolvedTypeWorld>,
    /// The SFC's script parses, sharing one semantic pass per block when the
    /// engine holds one (see [`Self::semantic_of`]).
    pub(crate) scripts: Option<&'a dyn crate::linter::sfc_scripts::ScriptParses>,
    /// What the template and `<style>` `v-bind()` read from the script scope,
    /// from the SFC's one binding analysis. `Unknown` unless an enabled rule
    /// demanded it (`has_template_read_demand`) and every block was walked.
    pub(crate) template_side: super::TemplateSideReads<'a>,
    /// The `<script setup generic="...">` attribute: type parameters whose
    /// constraints and defaults can name an import.
    pub(crate) setup_generic: Option<&'a str>,
    /// The SFC file the script blocks import from. `None` outside the SFC
    /// pass, so a rule that follows an import into another module keeps its
    /// single-file answer there.
    pub(crate) importing_file: Option<ImportingFile<'a>>,
}

/// Where an SFC's imports resolve from: its filename and the run's disk view.
#[derive(Clone, Copy)]
pub(crate) struct ImportingFile<'a> {
    pub(crate) filename: &'a str,
    /// The run's shared view, when the linter holds one. Without it a caller
    /// resolves through a fresh view of its own.
    pub(crate) sources: Option<&'a crate::linter::run_type_sources::RunTypeSources>,
}

impl<'a> SfcScriptContext<'a> {
    /// The parsed template together with the offset needed to report inside it.
    ///
    /// Both are present or neither is useful, so rules that report at a
    /// template location take them as a pair rather than unwrapping twice.
    #[inline]
    pub fn template_ast(&self) -> Option<(&'a SurfaceTree<'a>, u32)> {
        Some((self.template_surface?, self.template_offset?))
    }

    /// The semantic pass the engine shares for `program`, when `program` is
    /// one of this SFC's block parses. `None` means the rule runs its own.
    #[inline]
    pub(crate) fn semantic_of(
        &self,
        program: &oxc_ast::ast::Program<'_>,
    ) -> Option<&'a oxc_semantic::SemanticBuilderReturn<'a>> {
        self.scripts?.semantic_of(program)
    }

    /// The SFC's other script block, parsed on first read like every block.
    /// `None` when there is none or its parse failed.
    pub(crate) fn sibling_block(&self) -> Option<&'a oxc_ast::ast::Program<'a>> {
        let scripts = self.scripts?;
        let parsed = if self.is_script_setup {
            scripts.script()
        } else {
            scripts.script_setup()
        }?;
        (!parsed.panicked && parsed.diagnostics.is_empty()).then_some(&parsed.program)
    }
}

/// What the template side of an SFC reads from its script scope.
#[derive(Clone, Copy, Default)]
pub(crate) enum TemplateSideReads<'a> {
    /// The template side was not analyzed, or a block that can read a script
    /// name was not walked. No name can be ruled out.
    #[default]
    Unknown,
    /// The SFC has neither a template nor a style block.
    Nothing,
    /// The binding analysis recorded the template side's reads. The relation
    /// itself says whether every block was walked.
    Analyzed {
        reads: &'a vize_croquis::croquis::TemplateBindingReads,
        /// The component's runtime bindings, which a template name resolves
        /// to before any other source.
        bindings: &'a vize_croquis::BindingMetadata,
        /// Whether `bindings` holds every prop. Props typed by an import that
        /// did not resolve can declare any name.
        bindings_complete: bool,
        /// Whether `vue/no-undefined-refs` is active. It reports a template
        /// expression that reads a name nothing provides.
        undefined_refs_reported: bool,
    },
}

/// The block that evaluates a name no runtime binding provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnboundRead {
    /// A template expression.
    Template,
    /// A `<style>` `v-bind()` value.
    StyleBind,
}

impl TemplateSideReads<'_> {
    pub(crate) fn is_complete(self) -> bool {
        match self {
            Self::Unknown => false,
            Self::Nothing => true,
            Self::Analyzed { reads, .. } => reads.is_complete(),
        }
    }

    /// Whether the template side may read the script binding `name`.
    pub(crate) fn may_read(self, name: &str) -> bool {
        match self {
            Self::Unknown => true,
            Self::Nothing => false,
            Self::Analyzed { reads, .. } => reads.may_read(name),
        }
    }

    /// The read of `name` that nothing provides at runtime and no other rule
    /// reports: no binding, no prop, no JavaScript or Vue global. A
    /// `$`-prefixed name can be a property a plugin installs on the component
    /// proxy, as `vue/no-undefined-refs` also assumes.
    pub(crate) fn unbound_read(self, name: &str) -> Option<UnboundRead> {
        let Self::Analyzed {
            reads,
            bindings,
            bindings_complete,
            undefined_refs_reported,
        } = self
        else {
            return None;
        };
        if !reads.is_complete()
            || !bindings_complete
            || name.starts_with('$')
            || bindings.contains(name)
            || vize_croquis::builtins::is_js_global(name)
            || vize_croquis::builtins::is_vue_builtin(name)
        {
            return None;
        }
        if !undefined_refs_reported && reads.expression_reads(name) {
            return Some(UnboundRead::Template);
        }
        reads
            .style_bind_reads(name)
            .then_some(UnboundRead::StyleBind)
    }
}
