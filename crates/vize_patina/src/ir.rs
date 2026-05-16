//! Linter-facing intermediate representation.
//!
//! The IR abstracts over container formats such as Vue SFCs and standalone
//! JSX/TSX modules while keeping source slices zero-copy.

use oxc_span::SourceType;
use vize_atelier_sfc::SfcDescriptor;
use vize_carton::CompactString;

/// Container kind that produced the lint document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintDocumentKind {
    /// Vue Single File Component.
    VueSfc,
    /// Standalone HTML / template fragment.
    MarkupFragment,
    /// Standalone JS/TS/JSX/TSX module.
    ScriptModule,
    /// Reserved for future Astro adapters.
    AstroComponent,
    /// Reserved for future Svelte adapters.
    SvelteComponent,
}

/// Template syntax carried by a template fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSyntax {
    /// Vue template syntax.
    Vue,
    /// Reserved for future Astro templates.
    Astro,
    /// Reserved for future Svelte templates.
    Svelte,
    /// Generic HTML fragment.
    Html,
}

/// Script fragment kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptKind {
    /// `<script>`
    Script,
    /// `<script setup>`
    ScriptSetup,
    /// Standalone module / extracted script.
    Module,
}

/// Script language used to configure JS/TS parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLanguage {
    /// Plain JavaScript.
    JavaScript,
    /// JavaScript with JSX.
    Jsx,
    /// TypeScript.
    TypeScript,
    /// TypeScript with JSX.
    Tsx,
}

impl ScriptLanguage {
    /// Infer script language from an SFC `lang` attribute.
    pub fn from_lang_attr(lang: Option<&str>) -> Self {
        match lang.map(str::trim) {
            Some("tsx") => Self::Tsx,
            Some("ts") | Some("typescript") => Self::TypeScript,
            Some("jsx") => Self::Jsx,
            _ => Self::JavaScript,
        }
    }

    pub(crate) fn source_type(self) -> SourceType {
        match self {
            Self::JavaScript => SourceType::default().with_module(true),
            Self::Jsx => SourceType::jsx().with_module(true),
            Self::TypeScript => SourceType::ts().with_module(true),
            Self::Tsx => SourceType::tsx().with_module(true),
        }
    }
}

/// Byte range in the original source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    /// Inclusive start byte offset.
    pub start: u32,
    /// Exclusive end byte offset.
    pub end: u32,
}

impl ByteRange {
    /// Create a new byte range.
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

/// Template fragment extracted from a document.
#[derive(Debug, Clone, Copy)]
pub struct TemplateBlock<'a> {
    /// Template source.
    pub source: &'a str,
    /// Original source range.
    pub range: ByteRange,
    /// Template syntax.
    pub syntax: TemplateSyntax,
}

impl<'a> TemplateBlock<'a> {
    /// Create a template block.
    pub const fn new(source: &'a str, range: ByteRange, syntax: TemplateSyntax) -> Self {
        Self {
            source,
            range,
            syntax,
        }
    }
}

/// Script fragment extracted from a document.
#[derive(Debug, Clone, Copy)]
pub struct ScriptBlock<'a> {
    /// Script source.
    pub source: &'a str,
    /// Original source range.
    pub range: ByteRange,
    /// Script language.
    pub language: ScriptLanguage,
    /// Fragment kind.
    pub kind: ScriptKind,
}

impl<'a> ScriptBlock<'a> {
    /// Create a script block.
    pub const fn new(
        source: &'a str,
        range: ByteRange,
        language: ScriptLanguage,
        kind: ScriptKind,
    ) -> Self {
        Self {
            source,
            range,
            language,
            kind,
        }
    }

    /// Create a standalone module block.
    pub fn module(source: &'a str, language: ScriptLanguage) -> Self {
        Self::module_with_offset(source, 0, language)
    }

    /// Create a standalone module block starting at the given byte offset.
    pub fn module_with_offset(source: &'a str, offset: u32, language: ScriptLanguage) -> Self {
        Self::new(
            source,
            ByteRange::new(offset, offset + source.len() as u32),
            language,
            ScriptKind::Module,
        )
    }

    /// Start offset in the original source.
    pub const fn offset(&self) -> usize {
        self.range.start as usize
    }

    pub(crate) fn source_type(&self) -> SourceType {
        self.language.source_type()
    }
}

/// Generic lint input document.
#[derive(Debug, Clone)]
pub struct LintDocument<'a> {
    filename: CompactString,
    source: &'a str,
    kind: LintDocumentKind,
    template: Option<TemplateBlock<'a>>,
    scripts: Vec<ScriptBlock<'a>>,
}

impl<'a> LintDocument<'a> {
    /// Create an empty lint document.
    pub fn new(filename: &str, source: &'a str, kind: LintDocumentKind) -> Self {
        Self {
            filename: CompactString::from(filename),
            source,
            kind,
            template: None,
            scripts: Vec::with_capacity(2),
        }
    }

    /// Create a standalone script module document.
    pub fn script_module(filename: &str, source: &'a str, language: ScriptLanguage) -> Self {
        Self::new(filename, source, LintDocumentKind::ScriptModule)
            .with_script(ScriptBlock::module(source, language))
    }

    /// Create a standalone markup fragment document.
    pub fn markup_fragment(filename: &str, source: &'a str, syntax: TemplateSyntax) -> Self {
        Self::new(filename, source, LintDocumentKind::MarkupFragment).with_template(
            TemplateBlock::new(source, ByteRange::new(0, source.len() as u32), syntax),
        )
    }

    /// Create a standalone JSX module document.
    pub fn jsx(filename: &str, source: &'a str) -> Self {
        Self::script_module(filename, source, ScriptLanguage::Jsx)
    }

    /// Create a standalone TSX module document.
    pub fn tsx(filename: &str, source: &'a str) -> Self {
        Self::script_module(filename, source, ScriptLanguage::Tsx)
    }

    /// Build the IR from an SFC descriptor.
    pub fn from_sfc_descriptor(filename: &str, descriptor: &'a SfcDescriptor<'a>) -> Self {
        let mut document = Self::new(
            filename,
            descriptor.source.as_ref(),
            LintDocumentKind::VueSfc,
        );

        if let Some(template) = descriptor.template.as_ref() {
            document = document.with_template(TemplateBlock::new(
                template.content.as_ref(),
                ByteRange::new(template.loc.start as u32, template.loc.end as u32),
                TemplateSyntax::Vue,
            ));
        }

        if let Some(script) = descriptor.script.as_ref() {
            document = document.with_script(ScriptBlock::new(
                script.content.as_ref(),
                ByteRange::new(script.loc.start as u32, script.loc.end as u32),
                ScriptLanguage::from_lang_attr(script.lang.as_deref()),
                ScriptKind::Script,
            ));
        }

        if let Some(script_setup) = descriptor.script_setup.as_ref() {
            document = document.with_script(ScriptBlock::new(
                script_setup.content.as_ref(),
                ByteRange::new(script_setup.loc.start as u32, script_setup.loc.end as u32),
                ScriptLanguage::from_lang_attr(script_setup.lang.as_deref()),
                ScriptKind::ScriptSetup,
            ));
        }

        document
    }

    /// Attach a template fragment.
    pub fn with_template(mut self, template: TemplateBlock<'a>) -> Self {
        self.template = Some(template);
        self
    }

    /// Attach a script fragment.
    pub fn with_script(mut self, script: ScriptBlock<'a>) -> Self {
        self.scripts.push(script);
        self
    }

    /// Original filename.
    pub fn filename(&self) -> &str {
        self.filename.as_str()
    }

    /// Original source.
    pub const fn source(&self) -> &'a str {
        self.source
    }

    /// Container kind.
    pub const fn kind(&self) -> LintDocumentKind {
        self.kind
    }

    /// Template fragment.
    pub const fn template(&self) -> Option<TemplateBlock<'a>> {
        self.template
    }

    /// Script fragments.
    pub fn scripts(&self) -> &[ScriptBlock<'a>] {
        &self.scripts
    }
}
