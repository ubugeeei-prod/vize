//! Parse-only and transformed Relief providers for SFC templates.

use vize_armature::{parse_with_options, parse_with_options_and_template_syntax};
use vize_atelier_core::transform;
use vize_atlas::{
    InputId, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
    SourceInputId,
};
use vize_carton::Bump;
use vize_croquis::CroquisDocumentProduct;
use vize_relief::{
    Namespace, ParserOptions, ReliefArtifact, ReliefProduct, ReliefSnapshot, TransformOptions,
    TransformedReliefArtifact, TransformedReliefProduct, VueDialectInput,
};

use super::{
    SfcCompileSettingsInput, SfcDescriptor, SfcDescriptorProduct, SfcTemplateProduct, full_compile,
    is_sfc_source,
};

/// Parse one template block into source-faithful owned Relief syntax.
pub struct SfcReliefProvider;

impl Provider for SfcReliefProvider {
    type Product = ReliefProduct;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<VueDialectInput>()]
    }

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SfcCompileSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<SfcDescriptorProduct>(),
            ProductId::of::<SfcTemplateProduct>(),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<ReliefProduct as Product>::Value, ProviderError> {
        let artifact = context.get::<SfcDescriptorProduct>()?;
        let template = context.get::<SfcTemplateProduct>()?;
        let Some(descriptor) = artifact.descriptor() else {
            return Ok(None);
        };
        let Some(template) = template.as_ref() else {
            return Ok(None);
        };
        let allocator = Bump::new();
        let (root, parse_diagnostics) =
            if context.source_input::<SfcCompileSettingsInput>().is_some() {
                let request = full_compile::request_for(context);
                parse_with_options_and_template_syntax(
                    &allocator,
                    &template.text,
                    production_parser_options(&request.options, descriptor),
                    request.template_syntax,
                )
            } else {
                let dialect = context
                    .input::<VueDialectInput>()
                    .copied()
                    .unwrap_or_default();
                parse_with_options(
                    &allocator,
                    &template.text,
                    ParserOptions {
                        dialect,
                        ..Default::default()
                    },
                )
            };
        Ok(Some(ReliefArtifact::new(
            ReliefSnapshot::from_root(&root),
            parse_diagnostics.to_vec(),
        )))
    }
}

fn production_parser_options(
    options: &crate::SfcCompileOptions,
    descriptor: &SfcDescriptor<'_>,
) -> ParserOptions {
    let compiler = options.template.compiler_options.as_ref();
    let vapor_requested = options.vapor
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|script| script.attrs.contains_key("vapor"))
        || descriptor
            .script
            .as_ref()
            .is_some_and(|script| script.attrs.contains_key("vapor"));
    let is_vapor = !options.template.ssr && vapor_requested;
    let mut parser = ParserOptions {
        is_void_tag: vize_carton::is_void_tag,
        is_native_tag: Some(vize_carton::is_native_tag),
        custom_renderer: options.template.custom_renderer,
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        experimental_in_tag_comments: compiler
            .is_some_and(|options| options.experimental_in_tag_comments),
        ..ParserOptions::default()
    };
    if !is_vapor {
        parser.comments = compiler.is_some_and(|options| options.comments);
        parser.dialect = options.template.dialect;
    }
    parser
}

fn get_namespace(tag: &str, parent: Option<&str>) -> Namespace {
    if vize_carton::is_svg_tag(tag) {
        return Namespace::Svg;
    }
    if vize_carton::is_math_ml_tag(tag) {
        return Namespace::MathMl;
    }
    if let Some(parent) = parent {
        if vize_carton::is_svg_tag(parent) && tag != "foreignObject" {
            return Namespace::Svg;
        }
        if vize_carton::is_math_ml_tag(parent) && tag != "annotation-xml" && tag != "foreignObject"
        {
            return Namespace::MathMl;
        }
    }
    Namespace::Html
}

/// Apply Relief's compiler transforms without reparsing the template source.
pub struct SfcTransformedReliefProvider;

impl Provider for SfcTransformedReliefProvider {
    type Product = TransformedReliefProduct;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<VueDialectInput>()]
    }

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SfcCompileSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        let mut dependencies = vec![
            ProductId::of::<SfcDescriptorProduct>(),
            ProductId::of::<ReliefProduct>(),
        ];
        if context.source().text().contains("<script") {
            dependencies.push(ProductId::of::<CroquisDocumentProduct>());
        }
        dependencies
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<TransformedReliefProduct as Product>::Value, ProviderError> {
        let descriptor = context.get::<SfcDescriptorProduct>()?;
        let Some(descriptor) = descriptor.descriptor() else {
            return Ok(None);
        };
        let syntax = context.get::<ReliefProduct>()?;
        let Some(syntax) = syntax.as_ref() else {
            return Ok(None);
        };
        let dialect = context
            .input::<VueDialectInput>()
            .copied()
            .unwrap_or_default();
        let request = full_compile::request_for(context);
        let compiler = request.options.template.compiler_options.as_ref();
        let binding_metadata = if context.source().text().contains("<script") {
            let document = context.get::<CroquisDocumentProduct>()?;
            let analysis = document.analysis();
            Some(
                vize_carton::BindingMetadata {
                    bindings: analysis.bindings.bindings.clone(),
                    props_aliases: analysis.bindings.props_aliases.clone(),
                    is_script_setup: analysis.bindings.is_script_setup,
                }
                .into(),
            )
        } else {
            None
        };
        let source_is_ts = descriptor
            .script_setup
            .as_ref()
            .is_some_and(|script| crate::compile::is_ts_lang(script.lang.as_deref()))
            || descriptor
                .script
                .as_ref()
                .is_some_and(|script| crate::compile::is_ts_lang(script.lang.as_deref()));
        let is_ts = source_is_ts || request.options.template.is_ts || request.options.script.is_ts;
        let allocator = Bump::new();
        let mut root = syntax.snapshot().materialize(&allocator);
        let transformed = transform(
            &allocator,
            &mut root,
            TransformOptions {
                filename: request.options.parse.filename.clone(),
                prefix_identifiers: true,
                binding_metadata,
                is_ts,
                custom_renderer: request.options.template.custom_renderer,
                experimental_patterned_template: compiler
                    .is_some_and(|options| options.experimental_patterned_template),
                dialect,
                ..Default::default()
            },
            None,
        );
        Ok(Some(TransformedReliefArtifact::new(
            ReliefSnapshot::from_root(&root),
            syntax.parse_diagnostics().to_vec(),
            transformed.errors,
        )))
    }
}
