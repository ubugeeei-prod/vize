//! The production compile shapes the reach gate measures, each built the way
//! its shipping adapter builds it.

use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError,
    SfcParseOptions, SfcScriptOutputMode, StyleCompileOptions, TemplateCompileOptions,
    compile_sfc_for_adapter,
};

/// One production compile shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// `vize build` and the Rust `compile_sfc` API: the render function is
    /// inlined into `setup()` (`SfcScriptOutputMode::InlineTemplate`).
    DomInline,
    /// The Vite plugin's NAPI `compileSfc` default: module mode with a
    /// separate render function and the setup-state object
    /// (`SfcScriptOutputMode::SeparateTemplate`).
    DomModule,
    /// The Vite plugin's SSR build: the NAPI shape with `ssr: true`.
    Ssr,
    /// `vapor: true` through the Rust `compile_sfc` shape.
    Vapor,
}

impl Shape {
    pub const ALL: [Self; 4] = [Self::DomInline, Self::DomModule, Self::Ssr, Self::Vapor];

    /// The `[reach]` budget id.
    pub const fn id(self) -> &'static str {
        match self {
            Self::DomInline => "dom_inline",
            Self::DomModule => "dom_module",
            Self::Ssr => "ssr",
            Self::Vapor => "vapor",
        }
    }

    /// The Davinci stage a template reaches on this shape when its lane
    /// accepts it.
    pub const fn stage(self) -> &'static str {
        match self {
            Self::DomInline | Self::DomModule => "S2",
            Self::Ssr => "S4 plan",
            Self::Vapor => "native S3",
        }
    }

    /// The selection-counter namespace of this shape's backend: exactly one
    /// `<ns>accepted`, `<ns>legacy.<reason>` or `<ns>rejected` counter per
    /// template compile.
    pub const fn namespace(self) -> &'static str {
        match self {
            Self::DomInline | Self::DomModule => "davinci.s2_dom.",
            Self::Ssr => "davinci.s4_ssr.",
            Self::Vapor => "davinci.s3_vapor.",
        }
    }

    /// Whether this shape compiles through the DOM lane, whose legacy side
    /// the differential can force.
    pub const fn is_dom(self) -> bool {
        matches!(self, Self::DomInline | Self::DomModule)
    }
}

/// Match the source-level override in `compile_sfc_inner`: an adapter that
/// requests DOM still runs the Vapor compiler for these SFCs.
pub fn explicit_vapor_source(descriptor: &SfcDescriptor<'_>) -> bool {
    descriptor
        .script_setup
        .as_ref()
        .is_some_and(|script| script.attrs.contains_key("vapor"))
        || descriptor
            .script
            .as_ref()
            .is_some_and(|script| script.attrs.contains_key("vapor"))
}

/// Compile `descriptor` exactly as the shape's adapter does.
pub fn compile(
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
    shape: Shape,
) -> Result<SfcCompileResult, SfcError> {
    let has_scoped = descriptor.styles.iter().any(|style| style.scoped);
    let filename = vize_s0::String::from(filename);
    let options = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.clone()),
            scoped: has_scoped,
            ssr: shape == Shape::Ssr,
            compiler_options: Some(vize_atelier_dom::DomCompilerOptions::default()),
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename,
            scoped: has_scoped,
            ..Default::default()
        },
        vapor: shape == Shape::Vapor,
        scope_id: None,
    };
    let output = match shape {
        Shape::DomInline | Shape::Vapor => SfcScriptOutputMode::InlineTemplate,
        Shape::DomModule | Shape::Ssr => SfcScriptOutputMode::SeparateTemplate,
    };
    compile_sfc_for_adapter(
        descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        output,
    )
}
